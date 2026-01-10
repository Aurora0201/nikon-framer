<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
// 🟢 路径修正：确保指向 store/index.js
import { store } from '../../store/index.js'; 

import { usePreviewLogic } from '../../composables/usePreviewLogic';
import PreviewCanvas from '../workspace/PreviewCanvas.vue';
import WorkspaceFooter from '../workspace/WorkspaceFooter.vue';
// 🟢 1. 引入新组件
import ExportSettings from '../workspace/ExportSettings.vue';

const { frozenDisplay, isBusy, handleImgLoad, handleImgError } = usePreviewLogic();
const canvasRef = ref(null);
const handleReset = () => canvasRef.value?.resetView();

const footerHeight = ref(240);
const isDragging = ref(false);
// 🟢 2. 状态控制 ('preview' | 'settings')
const currentTab = ref('preview');

const startResize = () => {
  isDragging.value = true;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.userSelect = 'none';
};

const onResize = (e) => {
  if (!isDragging.value) return;
  let newHeight = window.innerHeight - e.clientY;
  const minHeight = 100;
  const maxHeight = window.innerHeight * 0.6;
  if (newHeight < minHeight) newHeight = minHeight;
  if (newHeight > maxHeight) newHeight = maxHeight;
  footerHeight.value = newHeight;
};

const stopResize = () => {
  isDragging.value = false;
  document.removeEventListener('mousemove', onResize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.userSelect = '';
};
</script>

<template>
  <div class="workspace-panel-container">
    
    <div class="workspace-header">
      <span 
        class="tab" 
        :class="{ active: currentTab === 'preview' }"
        @click="currentTab = 'preview'"
      >
        👁️ 实时预览
      </span>
      <span 
        class="tab" 
        :class="{ active: currentTab === 'settings' }"
        @click="currentTab = 'settings'"
      >
        ⚙️ 导出设置
      </span>
      
      <button class="reset-btn" @click="handleReset" title="重置视图">↺</button>
    </div>

    <div class="workspace-body">
      <KeepAlive>
        <PreviewCanvas 
          v-if="currentTab === 'preview'"
          ref="canvasRef"
          :display-data="frozenDisplay" 
          :is-busy="isBusy"
          @img-load="handleImgLoad"
          @img-error="handleImgError"
        />
        <ExportSettings v-else />
      </KeepAlive>
    </div>

    <div 
      v-if="currentTab === 'preview'"
      class="resize-handle" 
      @mousedown="startResize">
      <div class="handle-bar"></div>
    </div>

    <div 
      v-if="currentTab === 'preview'"
      class="workspace-footer-wrapper" 
      :style="{ height: footerHeight + 'px' }">
      <WorkspaceFooter :active-preset-id="store.activePresetId" />
    </div>

  </div>
</template>

<style scoped>
.workspace-panel-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh; /* 强制撑满 */
  overflow: hidden;
  position: relative;
  background: #1a1a1a;
}

.workspace-body {
  /* 🟢 [核心修复] 加上 display: flex，打通父子高度传递 */
  display: flex;
  flex-direction: column;
  
  flex: 1;
  min-height: 0;
  position: relative;
  background: #151515;
  overflow: hidden;
}

.workspace-header {
  height: 40px;
  flex-shrink: 0;
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

.resize-handle {
  height: 10px;
  margin-top: -5px;
  cursor: ns-resize;
  z-index: 50;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
  position: relative;
}

.resize-handle .handle-bar {
  width: 100%;
  height: 1px;
  background: #333;
  transition: all 0.2s;
}

.resize-handle:hover .handle-bar {
  height: 3px;
  background: #646cff;
  width: 40px;
  border-radius: 2px;
}

.workspace-footer-wrapper {
  flex-shrink: 0;
  background: #1a1a1a;
  overflow: hidden;
}
</style>