// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

// 🟢 辅助函数：构建上下文 (重构版)
// 原则：后端接管审美，前端不再发送字体配置，只发送模式特有的必要参数。
function buildBatchContext() {
  const currentStyle = store.settings.style;

  // 1. 极简白底 (BottomWhite)
  // 后端定义: StyleOptions::BottomWhite (Unit Variant)
  if (currentStyle === 'BottomWhite') {
    return { 
      style: 'BottomWhite' 
    };
  }

  // 2. 高斯模糊 (GaussianBlur)
  // 后端定义: StyleOptions::GaussianBlur { shadow_intensity: f32 }
  if (currentStyle === 'GaussianBlur') {
    return {
      style: 'GaussianBlur',
      // 确保转为浮点数，符合 Rust f32 类型
      shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0
    };
  }

  // 3. 大师模式 (Master)
  // 后端定义: StyleOptions::Master (Unit Variant)
  // 字体由后端 MasterProcessor 内部加载，前端无需关心
  if (currentStyle === 'Master') {
    return { 
      style: 'Master' 
    };
  }

  // 🚀 未来预留：自定义模式 (Custom)
  // 只有在这个模式下，我们才恢复发送 fontConfig
  /*
  if (currentStyle === 'Custom') {
    return {
      style: 'Custom',
      font: {
        filename: store.settings.font,
        weight: store.settings.weight
      }
    };
  }
  */

  // 默认兜底
  console.warn("未知的样式，回退到默认参数");
  return { 
    style: 'BottomWhite' 
  };
}

export function useBatchProcess() {
  const canStop = ref(false);
  let stopTimer = null;

  // 监听全局处理状态，如果变为 false (任务结束/出错)，重置 canStop
  watch(() => store.isProcessing, (newVal) => {
    if (!newVal) {
      canStop.value = false;
      if (stopTimer) clearTimeout(stopTimer);
    }
  });

  onUnmounted(() => {
    if (stopTimer) clearTimeout(stopTimer);
  });

  const handleBatchClick = async () => {
    // === 场景 A: 停止 ===
    if (store.isProcessing) {
      if (canStop.value) {
        store.setStatus("正在终止任务...", "loading");
        try {
          await invoke('stop_batch_process');
        } catch (err) {
          console.error("终止失败:", err);
        }
      }
      return;
    }

    // === 场景 B: 启动 ===
    if (store.fileQueue.length === 0) {
      store.setStatus("列表为空，请先添加照片！", "error");
      return;
    }

    // 1. 获取文件路径
    const filePaths = store.fileQueue.map(f => f.path);

    // 🟢 2. 动态构建 Context (使用瘦身后的辅助函数)
    // 这里生成的对象结构，必须严格匹配 Rust 后端的 Enum 定义
    const contextPayload = buildBatchContext();
    console.log("📦 [V2] 发送 Payload:", contextPayload);

    // 更新状态
    store.isProcessing = true;
    canStop.value = false;
    store.setStatus("准备开始批处理...", "loading");
    store.progress.percent = 0;

    // 启动计时器 (3秒后允许终止)
    if (stopTimer) clearTimeout(stopTimer);
    stopTimer = setTimeout(() => {
      if (store.isProcessing) canStop.value = true;
    }, 3000);

    // 调用后端
    try {
      await invoke('start_batch_process_v2', {
        filePaths: filePaths,
        context: contextPayload
      });
    } catch (error) {
      console.error("启动异常:", error);
      store.isProcessing = false;
      store.setStatus("启动失败: " + error, "error");
    }
  };

  // UI 计算属性
  const buttonText = computed(() => {
    if (!store.isProcessing) return '开始批处理 (Start Batch)';
    if (!canStop.value) return '启动中... (Starting)';
    return '终止处理 (Stop)';
  });

  const buttonClass = computed(() => ({
    'processing-mode': store.isProcessing && !canStop.value,
    'can-stop': store.isProcessing && canStop.value,
  }));

  const buttonCursor = computed(() => 
    (store.isProcessing && !canStop.value) ? 'not-allowed' : 'pointer'
  );

  return {
    handleBatchClick,
    buttonText,
    buttonClass,
    buttonCursor,
    canStop
  };
}