import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

export function usePreviewLogic() {
  // --- 1. 状态定义 ---
  const frozenDisplay = ref({ 
    url: '', 
    type: 'preset', 
    text: '', 
    presetId: '' 
  });
  
  const imgLoading = ref(false);

  // --- 2. 计算属性 ---
  const isBusy = computed(() => {
    return store.isProcessing || imgLoading.value || store.isLoadingPresets;
  });

  // --- 3. 核心防抖 Watcher ---
  watch(
    () => ({ 
      source: store.previewSource, 
      processing: store.isProcessing,
      switching: store.isLoadingPresets,
      currentId: store.activePresetId 
    }),
    ({ source, processing, switching, currentId }) => {
      // 拦截一：繁忙状态
      if (processing || switching) return;

      // 拦截二：防退化机制 (同模式下不退回 Preset)
      const isSamePreset = frozenDisplay.value.presetId === currentId;
      if (
        source.type === 'preset' && 
        frozenDisplay.value.type === 'result' && 
        store.activeFilePath &&
        isSamePreset
      ) {
        return; 
      }

      // 通过：更新
      frozenDisplay.value = { ...source, presetId: currentId };
    },
    { deep: true, immediate: true }
  );

  // --- 4. 辅助 Watcher ---
  // 监听 URL 变化触发 Loading
  watch(() => frozenDisplay.value.url, (newVal, oldVal) => {
    if (newVal && newVal !== oldVal) {
      imgLoading.value = true;
    }
  });

  // --- 5. Rust 通信逻辑 ---
  const checkPreviewStatus = async () => {
    if (!store.activeFilePath || !store.activePresetId) return;
    
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

  // 监听文件或模式变化，触发 Rust 检查
  watch([() => store.activeFilePath, () => store.activePresetId], () => checkPreviewStatus(), { immediate: true });
  
  // 监听处理结束，触发 Rust 检查
  watch(() => store.isProcessing, (newVal, oldVal) => { 
    if (oldVal === true && newVal === false) checkPreviewStatus(); 
  });

  // --- 6. 暴露给组件的方法 ---
  const handleImgLoad = () => { imgLoading.value = false; };
  const handleImgError = (e) => {
    imgLoading.value = false;
    // 这里简单处理，实际 UI 逻辑可以交给组件
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