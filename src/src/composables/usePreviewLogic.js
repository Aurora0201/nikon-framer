import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store/index.js'; // 路径更新

export function usePreviewLogic() {
  const frozenDisplay = ref({ 
    url: '', 
    type: 'preset', 
    text: '', 
    presetId: '' 
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
      // 监听 loading
      loading: imgLoading.value 
    }),
    ({ source, processing, switching, currentId, loading }) => {
      // 拦截器：如果正在加载数据 (loading=true)，则保持冻结
      if (processing || switching || loading) return;

      const isSamePreset = frozenDisplay.value.presetId === currentId;
      if (
        source.type === 'preset' && 
        frozenDisplay.value.type === 'result' && 
        store.activeFilePath &&
        isSamePreset
      ) {
        return; 
      }

      // 更新画面
      frozenDisplay.value = { ...source, presetId: currentId };
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

    try {
      const existingPath = await invoke('check_output_exists', {
        filePath: currentPath,
        style: currentStyle
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

  watch([() => store.activeFilePath, () => store.activePresetId], () => checkPreviewStatus(), { immediate: true });
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