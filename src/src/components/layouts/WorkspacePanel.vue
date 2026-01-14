<script setup>
import { ref } from 'vue';
import { store } from '../../store/index.js'; 

// 🟢 [Tauri V2 适配] 引入新的 Shell 插件路径
// 如果你还在用 V1，请改回 '@tauri-apps/api/shell'
import { open } from '@tauri-apps/plugin-shell';

import { usePreviewLogic } from '../../composables/usePreviewLogic';
import PreviewCanvas from '../workspace/PreviewCanvas.vue';
import WorkspaceFooter from '../workspace/WorkspaceFooter.vue';
import ExportSettings from '../workspace/ExportSettings.vue';

const { frozenDisplay, isBusy, handleImgLoad, handleImgError } = usePreviewLogic();
const canvasRef = ref(null);
const handleReset = () => canvasRef.value?.resetView();

const footerHeight = ref(240);
const isDragging = ref(false);
const currentTab = ref('preview');

// 🔗 打开 GitHub 链接的函数 (Tauri 安全方式)
const openGithub = async () => {
  try {
    // 替换为你真实的仓库地址
    await open('https://github.com/Aurora0201/nikon-framer');
  } catch (error) {
    console.error('Failed to open URL:', error);
  }
};

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
      <div class="tabs-container">
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
      </div>
      
      <div class="header-actions">
        <a 
          href="javascript:void(0)" 
          @click.prevent="openGithub"
          class="icon-btn github" 
          title="View on GitHub"
        >
          <svg viewBox="0 0 16 16" width="20" height="20" fill="currentColor" style="display: block;">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"></path>
          </svg>
        </a>

        <button class="icon-btn reset" @click="handleReset" title="重置视图">
          <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
            <path d="M3 3v5h5"></path>
          </svg>
        </button>
      </div>
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
  height: 100vh;
  overflow: hidden;
  position: relative;
  background: transparent;
}

/* 🟢 修改重点：点阵背景 + 暗角效果 */
.workspace-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;

  /* --- 变量定义 (从全局变量派生或保持同步) --- */
  --workspace-bg: var(--bg-color);
  --workspace-dot: var(--border-color);
  --workspace-shadow: rgba(0, 0, 0, 0.2);

  /* 基础背景色 */
  background-color: var(--workspace-bg);

  /* 1. 绘制点阵 (在设置页面可以减弱或消失) */
  background-image: radial-gradient(var(--workspace-dot) 1px, transparent 1px);
  
  /* 2. 控制点阵间距 */
  background-size: 20px 20px; 

  /* 3. 添加暗角 (Vignette) */
  box-shadow: inset 0 0 120px var(--workspace-shadow);
  
  /* 4. 主题切换过渡 */
  transition: background-color 0.3s ease, background-image 0.3s ease, box-shadow 0.3s ease;
}

/* Light Mode Overrides for Workspace */
:global([data-theme='light']) .workspace-body {
  --workspace-bg: var(--bg-workspace); 
  --workspace-dot: rgba(0, 0, 0, 0.05); 
  --workspace-shadow: transparent; 
}

.workspace-header {
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: transparent;
  border-bottom: 1px solid var(--border-color);
  position: relative;
  z-index: 100; /* 确保层级高于 resizer */
}

.tabs-container {
  display: flex;
  gap: 10px;
}

.tab { 
  padding: 4px 12px; 
  font-size: 0.85em; 
  color: var(--text-sub); 
  cursor: pointer; 
  transition: color 0.2s;
  user-select: none;
}
.tab:hover { color: var(--text-main); }
.tab.active { 
  color: var(--text-main); 
  background: var(--input-bg); 
  border-radius: 6px; 
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-sub);
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  line-height: 0;
}

.icon-btn:hover {
  background: var(--input-bg);
  color: var(--text-main);
}

/* GitHub 按钮特有样式 */
.icon-btn.github:hover {
  color: var(--text-main); 
}

/* Reset 按钮特有样式 */
.icon-btn.reset:hover {
  color: var(--bright-blue, #646cff);
  transform: rotate(-30deg);
}

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
  background: var(--border-color);
  transition: all 0.2s;
}

.resize-handle:hover .handle-bar {
  height: 3px;
  background: var(--bright-blue, #646cff);
  width: 40px;
  border-radius: 2px;
}

.workspace-footer-wrapper {
  flex-shrink: 0;
  background: transparent;
  overflow: hidden;
}
</style>