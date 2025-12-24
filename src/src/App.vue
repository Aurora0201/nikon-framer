<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { store } from './store.js';

// 引入所有组件
import ControlPanel from './components/ControlPanel.vue';
import FileList from './components/FileList.vue';
import StatusBar from './components/StatusBar.vue';
import PreviewModal from './components/PreviewModal.vue';
import DebugTools from './components/DebugTools.vue';

// --- 本地状态 ---
// 控制是否允许点击停止 (3秒防误触)
const canStop = ref(false);
let stopTimer = null; 

// --- 监听器句柄 ---
let unlistenDrop = null;
let unlistenProgress = null;
let unlistenStatus = null;
let unlistenEnter = null;
let unlistenLeave = null;

// --- 生命周期：卸载清理 ---
onUnmounted(() => {
  if (stopTimer) clearTimeout(stopTimer);
  if (unlistenDrop) unlistenDrop();
  if (unlistenProgress) unlistenProgress();
  if (unlistenStatus) unlistenStatus();
  if (unlistenEnter) unlistenEnter();
  if (unlistenLeave) unlistenLeave();
});

// --- 生命周期：挂载初始化 ---
onMounted(async () => {
  console.log("🚀 App 已挂载，开始注册监听器...");

  // 1. 监听进度更新 (process-progress)
  // 🟢 注意：去掉 const，赋值给外部变量
  unlistenProgress = await listen('process-progress', (event) => {
    const { current, total, filepath, status } = event.payload;
    
    store.updateProgress(current, total);
    const filename = filepath.replace(/^.*[\\/]/, '');

    if (status === 'skipped') {
      store.setStatus(`[跳过] 无EXIF: ${filename}`, 'loading');
    } else {
      store.setStatus(`正在处理: ${filename}`, 'loading');
    }
  });

  // 2. 监听任务状态 (process-status)
  unlistenStatus = await listen('process-status', (event) => {
    const status = event.payload; 
    
    // 任务结束/停止时的通用清理
    store.isProcessing = false; 
    canStop.value = false;
    if (stopTimer) clearTimeout(stopTimer);

    if (status === 'finished') {
      store.setStatus("批处理完成！✨", "success");
      setTimeout(() => {
        // 只有当没有开始新任务时才重置
        if (!store.isProcessing) store.progress.total = 0; 
      }, 1500);
    } else if (status === 'stopped') {
      store.setStatus("已终止批处理", "error");
      store.progress.total = 0;
    }
  });

  // 3. 全局拖拽系统 (视觉 + 数据)
  
  // 进入窗口
  unlistenEnter = await listen('tauri://drag-enter', () => {
    if (!store.isProcessing) {
      store.isDragging = true;
      // console.log('文件进入');
    }
  });

  // 离开窗口
  unlistenLeave = await listen('tauri://drag-leave', () => {
    store.isDragging = false;
    // console.log('文件离开');
  });

  // 放下文件
  unlistenDrop = await listen('tauri://drag-drop', async (event) => {
    store.isDragging = false; // 立即复位 UI

    if (store.isProcessing) return;

    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      try {
        const validFiles = await invoke('filter_files', { paths });
        
        if (validFiles.length > 0) {
          // 格式转换：String -> Object
          const formattedFiles = validFiles.map(path => ({
            name: path.replace(/^.*[\\/]/, ''),
            path: path
          }));

          const count = store.addFiles(formattedFiles);
          
          if (count > 0) {
            store.setStatus(`已添加 ${count} 个文件`, 'success');
          } else {
            store.setStatus("文件已存在列表中", 'normal');
          }
        } else {
          store.setStatus("未检测到支持的图片文件", "error");
        }
      } catch (e) {
        console.error("Drop Error:", e);
        store.setStatus("文件添加失败", "error");
      }
    }
  });
});

// --- 🟢 核心动作：处理按钮点击 ---
const handleBatchClick = async () => {
  // === 场景 A: 正在处理中 -> 处理“终止”逻辑 ===
  if (store.isProcessing) {
    if (canStop.value) {
      store.setStatus("正在终止任务...", "loading");
      try {
        await invoke('stop_batch_process');
      } catch (err) {
        console.error("终止失败:", err);
      }
    } else {
      console.log("⚠️ 3秒防误触保护期");
    }
    return;
  }

  // === 场景 B: 未处理 -> 处理“开始”逻辑 ===
  if (store.fileQueue.length === 0) {
    store.setStatus("列表为空，请先添加照片！", "error");
    return;
  }

  // 1. 准备参数 (Payload)
  const payload = {
    filePaths: store.fileQueue.map(f => f.path),
    style: store.settings.style,
    fontFilename: store.settings.font,
    fontWeight: store.settings.weight,
    shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0
  };

  console.log("📦 准备发送参数:", payload);

  // 2. 更新 UI 状态
  store.isProcessing = true;
  canStop.value = false;
  store.setStatus("准备开始批处理...", "loading");
  store.progress.percent = 0;

  // 3. 启动 3秒倒计时 (防误触)
  if (stopTimer) clearTimeout(stopTimer);
  stopTimer = setTimeout(() => {
    if (store.isProcessing) {
      canStop.value = true; // 允许点击停止
    }
  }, 3000);

  // 4. 调用 Rust
  try {
    await invoke('start_batch_process', payload);
  } catch (error) {
    console.error("启动异常:", error);
    store.isProcessing = false;
    store.setStatus("启动失败: " + error, "error");
  }
};

// --- 计算属性：按钮文字 ---
const buttonText = computed(() => {
  if (!store.isProcessing) return '开始批处理 (Start Batch)';
  if (!canStop.value) return '启动中... (Starting)';
  return '终止处理 (Stop)';
});

// --- 计算属性：按钮样式类 ---
const buttonClass = computed(() => {
  return {
    'processing-mode': store.isProcessing && !canStop.value,
    'can-stop': store.isProcessing && canStop.value,
  };
});
</script>

<template>
  <h1>NIKON <span>Z</span> FRAMER</h1>

  <div class="control-group">
    <ControlPanel />
    
    <FileList />

    <button 
      id="start-batch-btn"
      @click="handleBatchClick"
      :disabled="!store.isProcessing && store.fileQueue.length === 0"
      :class="buttonClass"
      :style="{ cursor: (store.isProcessing && !canStop) ? 'not-allowed' : 'pointer' }"
    >
      {{ buttonText }}
    </button>
  </div>
  
  <StatusBar />
  <PreviewModal />
  <DebugTools />
</template>

<style scoped>
/* 启动中 (灰色等待) */
button.processing-mode {
  background-color: #666;
  border-color: #555;
  color: #ccc;
  opacity: 0.8;
}

/* 允许停止 (红色警告 + 呼吸动画) */
button.can-stop {
  background-color: #3e1f1f;
  border-color: #ff4444;
  color: #ff4444;
  animation: pulse-red 2s infinite;
}

button.can-stop:hover {
  background-color: #ff4444;
  color: white;
}

@keyframes pulse-red {
  0% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0.4); }
  70% { box-shadow: 0 0 0 10px rgba(255, 68, 68, 0); }
  100% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0); }
}
</style>