import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store/index.js'; // 路径更新
import { buildExportPayload, buildStylePayload } from '../utils/payloadHelper.js';
import { frameRegistry } from '../frames/registry.js';

export function usePreviewLogic() {
  const frozenDisplay = ref({ 
    url: '', 
    type: 'preset', 
    text: '', 
    presetId: '',
    filePath: '' // 🟢 新增：记录这张图属于哪个文件
  });
  
  const imgLoading = ref(false);

  // --- Blob 管理 ---
  watch(
    [() => store.activeFilePath, () => store.settings.style], 
    async ([newPath, newStyle], [oldPath, oldStyle]) => {
      
      // 🟢 使用 Store Getter 获取配置 (Registry 已集成在 Store 中)
      const newConfig = store.currentModeConfig; 
      
      // 注意：这里无法直接获取 oldConfig，因为 style 已经变了，Store 里的 getter 只能获取当前的
      // 但我们可以通过简单的逻辑推断，或者不清理也没大碍(Store.setMode 里如果想清理可以在那里做)
      // 为了保持逻辑，我们可以仅判断“当前是否需要Blob”
      
      if (newConfig?.features?.useRawPreview && newPath) {
        if (newPath !== oldPath || newStyle !== oldStyle) {
          imgLoading.value = true;
          await store.loadPreviewBlob(newPath);
          imgLoading.value = false;
        }
      } else {
        // 如果当前模式不需要 Blob，但之前的 Blob 还在，就清理
        store.cleanupBlob();
      }
    },
    { immediate: true }
  );

  const isBusy = computed(() => {
    return store.isProcessing || imgLoading.value || store.isLoadingPresets;
  });

  // --- 核心 UI 更新逻辑 ---
  watch(
    () => ({ 
      source: store.previewSource, 
      processing: store.isProcessing,
      switching: store.isLoadingPresets,
      currentId: store.activePresetId,
      loading: imgLoading.value,
      // 🟢 新增：把当前文件路径也放入监听对象的解构中，方便对比
      currentPath: store.activeFilePath 
    }),
    ({ source, processing, switching, currentId, loading, currentPath }) => {
      
      if (processing || switching || loading) return;

      const isSamePreset = frozenDisplay.value.presetId === currentId;
      // 🟢 关键判断：当前显示的文件路径，是否等于现在选中的文件路径
      const isSameFile = frozenDisplay.value.filePath === currentPath;

      // // 拦截器逻辑修正：
      // if (
      //   source.type === 'preset' && 
      //   frozenDisplay.value.type === 'result' && 
      //   isSamePreset &&
      //   isSameFile // 🟢 只有是“同一张照片”且“同一个样式”时，才进行防止闪烁的拦截
      // ) {
      //   // 如果是切到了另一张照片 (isSameFile 为 false)，这里就不会拦截，
      //   // 会直接往下走，从而正确切换到 preset 视图。
      //   return; 
      // }

      // 更新画面
      frozenDisplay.value = { 
        ...source, 
        presetId: currentId,
        filePath: currentPath // 🟢 更新时，务必记下当前是哪张图
      };
    },
    { deep: true, immediate: true }
  );

  // --- 辅助 Watcher: DOM 渲染阶段 ---
  // 当 frozenDisplay 确实更新后，我们再次进入 loading 状态，等待 DOM 渲染
  watch(() => frozenDisplay.value.url, (newVal, oldVal) => {
    if (newVal && newVal !== oldVal) {
      imgLoading.value = true;
    }
  });

  // (其余代码保持不变...)
  const checkPreviewStatus = async () => {
    if (!store.activeFilePath || !store.activePresetId) return;
    
    // 🟢 判断逻辑：如果当前模式“使用原图预览”，则不需要检查后端缓存
    if (store.currentModeConfig.features.useRawPreview) return;

    const currentPath = store.activeFilePath;
    const currentStyle = store.activePresetId;

      // 1. 复用逻辑构建参数
    const stylePayload = buildStylePayload(
      store.activePresetId, 
      store.modeParams, 
      frameRegistry
    );
  
    const exportPayload = buildExportPayload(store.exportSettings);

    try {
      const existingPath = await invoke('check_output_exists', {
        filePath: currentPath,
        styleOptions: stylePayload,  // Rust: style_options
        exportConfig: exportPayload  // Rust: export_config
      });
      if (existingPath) {
        store.markFileProcessedWithStyle(currentPath, currentStyle, existingPath);
      } else {
        store.clearProcessedStatusWithStyle(currentPath, currentStyle);
      }
    } catch (e) {
      console.error("检查文件存在性失败:", e);
    }
  };

  // 🟢 [修复] 添加 store.exportSettings 到监听列表
  // 任何影响输出路径/文件名的因素变化，都必须重新检查
  watch(
    [
      () => store.activeFilePath, 
      () => store.activePresetId,
      () => store.exportSettings // ✅ 新增：监听导出设置
    ], 
    () => checkPreviewStatus(), 
    { 
      immediate: true, 
      deep: true // ✅ 新增：因为 exportSettings 是对象，需要深度监听属性变化 (如 format, customPath)
    }
  );
  watch(() => store.isProcessing, (newVal, oldVal) => { 
    if (oldVal === true && newVal === false) checkPreviewStatus(); 
  });

  const handleImgLoad = () => { imgLoading.value = false; };
  const handleImgError = (e) => {
    imgLoading.value = false;
    if(e.target) {
        e.target.style.backgroundColor = '#333';
        e.target.alt = "图片丢失";
    }
  };

  return {
    frozenDisplay,
    isBusy,
    handleImgLoad,
    handleImgError
  };
}