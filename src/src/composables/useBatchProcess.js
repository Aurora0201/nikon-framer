// src/composables/useBatchProcess.js
import { ref, computed, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store/index.js';
// 1. 引入注册表，用于查询配置
import { frameRegistry } from '../frames/registry.js';
import { buildExportPayload, buildStylePayload } from '../utils/payloadHelper.js';

// =============================================================================
// 🟢 辅助函数：构建上下文 (OCP 通用版)
// =============================================================================
function buildBatchContext() {
  let targetStyleId = store.activePresetId;

  // 1. 容错逻辑：如果未选中，尝试兜底
  if (!targetStyleId) {
    const currentPresets = store.currentPresets;
    if (currentPresets && currentPresets.length > 0) {
      targetStyleId = currentPresets[0].id;
    } else {
      console.warn("⚠️ [Batch] 未找到有效的 Style ID");
      // 这里的异常处理视你的业务而定
      targetStyleId = 'BottomWhite'; 
    }
  }

  // 2. 🟢 使用 Helper 构建样式部分
  const stylePayload = buildStylePayload(
    targetStyleId, 
    store.modeParams, 
    frameRegistry
  );

  // 3. 🟢 使用 Helper 构建导出部分
  const exportPayload = buildExportPayload(store.exportSettings);

  // 4. 最终组装
  // Rust BatchContext: 
  //   - options: #[serde(flatten)] -> 展开 stylePayload
  //   - export:  #[serde(rename="exportSettings")] -> 放入 exportPayload
  return {
    ...stylePayload,       
    exportSettings: exportPayload
  };
}

// =============================================================================
// 🟢 主要 Composable 逻辑
// =============================================================================
export function useBatchProcess() {
  const canStop = ref(false);
  let stopTimer = null;

  // 监听全局处理状态，如果任务结束，重置停止按钮状态
  watch(() => store.isProcessing, (newVal) => {
    if (!newVal) {
      canStop.value = false;
      if (stopTimer) clearTimeout(stopTimer);
    }
  });

  onUnmounted(() => {
    if (stopTimer) clearTimeout(stopTimer);
  });

  // =================================================
  // 🟢 核心通用执行器 (Internal Executor)
  // 无论是批处理还是单张处理，最终都调用这个函数
  // =================================================
  const executeProcess = async (targetPaths, modeName = "处理") => {
    if (targetPaths.length === 0) {
      store.setStatus("文件列表为空！", "error");
      return;
    }

    // 1. 准备上下文
    const contextPayload = buildBatchContext();

    // 2. 智能过滤：调用 Rust 检查重复文件
    store.setStatus(`正在检查${modeName}文件...`, "loading");
    let filesToProcess = [];
    let skippedCount = 0;

    try {
      // 传递完整的 context 对象供 Rust 判断 (例如 is_editable 模式不过滤)
      filesToProcess = await invoke('filter_unprocessed_files', { 
        paths: targetPaths, 
        context: contextPayload 
      });
      
      skippedCount = targetPaths.length - filesToProcess.length;
    } catch (e) {
      console.error("过滤检查失败，降级为全部处理:", e);
      filesToProcess = targetPaths;
    }

    // 3. 检查过滤结果
    if (filesToProcess.length === 0) {
      store.setStatus(`文件已存在，无需${modeName}！`, "success");
      // 稍微更新一下进度条给个视觉反馈
      store.updateProgress(targetPaths.length, targetPaths.length);
      return; 
    }

    if (skippedCount > 0) {
      console.log(`[Batch] 自动跳过 ${skippedCount} 张已存在文件`);
    }

    // 4. 更新 UI 为“处理中”状态
    store.isProcessing = true;
    canStop.value = false; // 先禁用停止，过3秒开启
    store.setStatus(
      skippedCount > 0 
        ? `开始${modeName} (已跳过 ${skippedCount} 张)...` 
        : `准备开始${modeName}...`, 
      "loading"
    );
    
    // 5. 重置进度
    store.progress.percent = 0;
    store.progress.current = 0;
    store.progress.total = filesToProcess.length;

    // 6. 启动“停止按钮”计时器 (3秒后允许终止)
    if (stopTimer) clearTimeout(stopTimer);
    stopTimer = setTimeout(() => {
      if (store.isProcessing) canStop.value = true;
    }, 3000);

    // 7. 正式调用 Rust V3 管道接口
    try {
      await invoke('start_batch_process_v3', {
        filePaths: filesToProcess, 
        context: contextPayload
      });
    } catch (error) {
      console.error("启动异常:", error);
      store.isProcessing = false;
      store.setStatus("启动失败: " + error, "error");
    }
  };

  // =================================================
  // 🟢 A. 批量处理按钮点击事件
  // =================================================
  const handleBatchClick = async () => {
    // 场景: 如果正在运行，此按钮充当“停止”功能
    if (store.isProcessing) {
      if (canStop.value) {
        store.setStatus("正在终止任务...", "loading");
        try {
          await invoke('stop_batch_process');
        } catch (err) {
          store.setStatus("终止失败", "error");
        }
      }
      return;
    }

    // 场景: 启动批量任务
    if (store.fileQueue.length === 0) {
      store.setStatus("列表为空，请先添加照片！", "error");
      return;
    }

    const allPaths = store.fileQueue.map(f => f.path);
    await executeProcess(allPaths, "批处理");
  };

  // =================================================
  // 🟢 B. 单张处理按钮点击事件 (新增)
  // =================================================
  const handleSingleClick = async () => {
    // 忙碌状态下通过禁用属性控制，这里做双重保险
    if (store.isProcessing) return;

    if (!store.activeFilePath) {
      store.setStatus("请先选择一张照片！", "error");
      return;
    }

    // 构造只包含单张文件的数组，复用批处理管道
    const singlePath = [store.activeFilePath];
    await executeProcess(singlePath, "当前图片");
  };

  // =================================================
  // UI 计算属性
  // =================================================
  const buttonText = computed(() => {
    if (!store.isProcessing) return '生成全部';
    if (!canStop.value) return '启动中... ';
    return '终止处理';
  });

  const buttonClass = computed(() => ({
    'processing-mode': store.isProcessing && !canStop.value,
    'can-stop': store.isProcessing && canStop.value,
  }));

  const buttonCursor = computed(() => 
    (store.isProcessing && !canStop.value) ? 'not-allowed' : 'pointer'
  );

  // 单张按钮禁用状态：处理中 或 没有选中文件
  const isSingleDisabled = computed(() => {
    return store.isProcessing || !store.activeFilePath;
  });

  return {
    handleBatchClick,
    handleSingleClick, // 导出
    isSingleDisabled,  // 导出
    buttonText,
    buttonClass,
    buttonCursor,
    canStop
  };
}