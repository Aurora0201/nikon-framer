<script setup>
import { ref } from 'vue';
import { store } from '../../store.js';
// 引入拆分后的模块
import { usePreviewLogic } from '../../composables/usePreviewLogic';
import PreviewCanvas from '../workspace/PreviewCanvas.vue';
import WorkspaceFooter from '../workspace/WorkspaceFooter.vue';

// 1. 获取业务逻辑
const { 
  frozenDisplay, 
  isBusy, 
  handleImgLoad, 
  handleImgError 
} = usePreviewLogic();

// 2. 引用子组件实例 (用于调用 resetView)
const canvasRef = ref(null);

const handleReset = () => {
  canvasRef.value?.resetView();
};
</script>

<template>
  <div class="workspace-header">
    <span class="tab active">👁️ 实时预览</span>
    <span class="tab">⚙️ 导出设置</span>
    <button class="reset-btn" @click="handleReset" title="重置视图">↺</button>
  </div>

  <PreviewCanvas 
    ref="canvasRef"
    :display-data="frozenDisplay" 
    :is-busy="isBusy"
    @img-load="handleImgLoad"
    @img-error="handleImgError"
  />

  <WorkspaceFooter :active-preset-id="store.activePresetId" />
</template>

<style scoped>
.workspace-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  background: #151515;
  border-bottom: 1px solid #333;
  gap: 10px;
}
.reset-btn { margin-left: auto; background: transparent; border: none; color: #888; cursor: pointer; font-size: 1.2em; }
.reset-btn:hover { color: #fff; }
.tab { padding: 4px 12px; font-size: 0.85em; color: #888; cursor: pointer; }
.tab.active { color: #fff; background: #333; border-radius: 4px; }
</style>