// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

// 🟢 新增辅助函数：根据不同模式，组装不同的参数对象
function buildBatchContext() {
  const currentStyle = store.settings.style; // 例如 "BottomWhite" 或 "GaussianBlur"

  // 1. 公共参数：字体配置
  const fontConfig = {
    filename: store.settings.font,
    weight: store.settings.weight
  };

  // 2. 根据样式名称，构建不同的对象结构 (对应 Rust 的 Enum)
  switch (currentStyle) {
    case 'BottomWhite':
      return {
        style: 'BottomWhite', // 对应 Rust Enum 的变体名
        font: fontConfig
        // 白底模式不需要其他参数
      };

    case 'GaussianBlur':
      return {
        style: 'GaussianBlur',
        font: fontConfig,
        // 只有模糊模式才传这个参数
        shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0
      };

    // 未来扩展：
    // case 'FilmParams':
    //   return { style: 'FilmParams', iso: 400, showDate: true };

    default:
      console.warn("未知的样式，回退到默认参数");
      return {
        style: 'BottomWhite',
        font: fontConfig
      };
  }
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

    // 🟢 1. 获取文件路径
    const filePaths = store.fileQueue.map(f => f.path);

    // 🟢 2. 动态构建 Context (使用上面的辅助函数)
    // 这里生成的对象结构，完全匹配 Rust 的 Enum 定义
    const contextPayload = buildBatchContext();
    console.log("📦 [V2] 准备发送 Context:", contextPayload);

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
    canStop // 导出这个状态以防万一需要
  };
}