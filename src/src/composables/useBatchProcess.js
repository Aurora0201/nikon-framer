// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store/index.js';

// 🟢 辅助函数：构建上下文
function buildBatchContext() {
  let targetStyleId = store.activePresetId;

  // 1. 容错：如果未选中，尝试获取当前列表第一个
  if (!targetStyleId) {
    const currentPresets = store.currentPresets;
    if (currentPresets && currentPresets.length > 0) {
      targetStyleId = currentPresets[0].id;
    }
  }

  // 2. 兜底：如果还是没有，使用默认值
  if (!targetStyleId) {
    console.warn("⚠️ [Batch] 未找到有效的 Style ID，使用默认兜底值");
    return { style: 'BottomWhite' }; 
  }

  console.log(`🔧 [Batch] 锁定后端 Style ID: ${targetStyleId}`);

  // 根据后端协议，直接发送 style 字段即可
  return { 
    style: targetStyleId 
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
    // =================================================
    // 🛑 场景 A: 停止任务
    // =================================================
    if (store.isProcessing) {
      if (canStop.value) {
        store.setStatus("正在终止任务...", "loading");
        try {
          await invoke('stop_batch_process');
        } catch (err) {
          console.error("终止失败:", err);
          store.setStatus("终止失败", "error");
        }
      }
      return;
    }

    // =================================================
    // ▶️ 场景 B: 启动任务
    // =================================================
    if (store.fileQueue.length === 0) {
      store.setStatus("列表为空，请先添加照片！", "error");
      return;
    }

    // 1. 准备数据
    const allPaths = store.fileQueue.map(f => f.path);
    const contextPayload = buildBatchContext();

    // 2. 🟢 智能过滤：调用 Rust 检查哪些文件还没生成过
    store.setStatus("正在检查重复文件...", "loading");
    let filesToProcess = [];
    let skippedCount = 0;

    try {
      // 调用我们在 main.rs 新增的 filter_unprocessed_files 命令
      filesToProcess = await invoke('filter_unprocessed_files', { 
        paths: allPaths, 
        // 传递字符串 ID (如 "BottomWhite")，Rust 端会自动拼接后缀检查
        style: contextPayload.style 
      });
      
      skippedCount = allPaths.length - filesToProcess.length;
    } catch (e) {
      console.error("过滤检查失败，将全部处理:", e);
      // 降级处理：如果检查失败，就全部重新跑一遍，保证功能可用
      filesToProcess = allPaths;
    }

    // 3. 检查过滤结果
    // Case 1: 所有文件都已存在
    if (filesToProcess.length === 0) {
      store.setStatus(`所有文件均已生成过 (${skippedCount} 张)，无需处理！`, "success");
      // 可以在这里稍微闪烁一下进度条表示完成，或者直接退出
      store.updateProgress(skippedCount, skippedCount);
      return; 
    }
    
    // Case 2: 有部分或全部需要处理
    if (skippedCount > 0) {
      console.log(`[Batch] 自动跳过 ${skippedCount} 张已存在文件`);
    }

    // 4. 更新 UI 状态
    store.isProcessing = true;
    canStop.value = false;
    store.setStatus(
      skippedCount > 0 
        ? `开始处理 (已跳过 ${skippedCount} 张重复)...` 
        : "准备开始批处理...", 
      "loading"
    );
    
    // 5. 重置进度 (Total 设为实际需要处理的数量)
    store.progress.percent = 0;
    store.progress.current = 0;
    store.progress.total = filesToProcess.length;

    // 6. 启动“停止按钮”计时器 (3秒后允许终止)
    if (stopTimer) clearTimeout(stopTimer);
    stopTimer = setTimeout(() => {
      // 只有还在处理中才显示停止按钮
      if (store.isProcessing) canStop.value = true;
    }, 3000);

    // 7. 正式调用后端批处理
    try {
      await invoke('start_batch_process_v2', {
        filePaths: filesToProcess, // 👈 关键：只传过滤后的列表
        context: contextPayload
      });
    } catch (error) {
      console.error("启动异常:", error);
      store.isProcessing = false;
      store.setStatus("启动失败: " + error, "error");
    }
  };

  // --- UI 计算属性 ---
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