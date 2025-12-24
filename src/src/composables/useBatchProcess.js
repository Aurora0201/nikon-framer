// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

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

    // 准备参数
    const payload = {
      filePaths: store.fileQueue.map(f => f.path),
      style: store.settings.style,
      fontFilename: store.settings.font,
      fontWeight: store.settings.weight,
      shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0
    };

    // 更新状态
    store.isProcessing = true;
    canStop.value = false;
    store.setStatus("准备开始批处理...", "loading");
    store.progress.percent = 0;

    // 启动计时器
    if (stopTimer) clearTimeout(stopTimer);
    stopTimer = setTimeout(() => {
      if (store.isProcessing) canStop.value = true;
    }, 3000);

    // 调用后端
    try {
      await invoke('start_batch_process', payload);
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
    canStop // 导出这个状态以防万一需要
  };
}