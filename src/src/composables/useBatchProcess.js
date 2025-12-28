// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

// 🟢 辅助函数：构建上下文 (极简版)
// 逻辑：直接获取当前选中的预设 ID，作为 style 发送给后端
// 因为你已经确保了 PRESET_CONFIGS 里的 id 与后端 Enum Variant 一一对应
function buildBatchContext() {
  let targetStyleId = store.activePresetId;

  // 🛡️ 容错处理：如果用户刚打开软件，还没点击任何预设卡片
  // 我们需要自动获取当前模式下的第一个预设 ID 作为默认值
  if (!targetStyleId) {
    const currentPresets = store.currentPresets;
    if (currentPresets && currentPresets.length > 0) {
      targetStyleId = currentPresets[0].id;
    }
  }

  // 🛡️ 最终兜底：如果连列表都是空的（极少见），使用你的默认白底 ID
  if (!targetStyleId) {
    console.warn("⚠️ [Batch] 未找到有效的 Style ID，使用默认兜底值");
    return { style: 'BottomWhite' }; 
  }

  console.log(`🔧 [Batch] 锁定后端 Style ID: ${targetStyleId}`);

  // 🟢 核心逻辑：
  // 根据目前的协议，我们只发送 style ID。
  // 虽然 Store 里有 shadowIntensity 等参数，但既然我们要遵守“后端接管审美”，
  // 这里暂时不发送这些参数，除非你的后端接口明确要求接收它们。
  
  // 如果是 GaussianBlur，且后端接口定义为 { style: 'GaussianBlur', shadowIntensity: f32 }
  // 你需要解开下面的注释并做判断。
  // 但根据你的指示“后端通过唯一的参数 style 来确定”，我们保持最简：
  
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

    // 🟢 2. 动态构建 Context
    const contextPayload = buildBatchContext();
    
    console.log("📦 [Batch] 最终发送 Payload:", JSON.stringify(contextPayload, null, 2));

    // 更新状态
    store.isProcessing = true;
    canStop.value = false;
    store.setStatus("准备开始批处理...", "loading");
    
    // 重置进度
    store.progress.percent = 0;
    store.progress.current = 0;
    store.progress.total = filePaths.length;

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