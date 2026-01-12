<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

const minimizeWindow = () => appWindow.minimize();
const toggleMaximize = async () => {
  await appWindow.toggleMaximize();
  isMaximized.value = await appWindow.isMaximized();
};
const closeWindow = () => appWindow.close();

let unlistenResize = null;
onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized();
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });
});

onUnmounted(() => {
  if (unlistenResize) unlistenResize();
});
</script>

<template>
  <header class="window-title-bar" data-tauri-drag-region>
    
    <div class="window-controls">
      
      <button class="mac-btn close" @click="closeWindow" title="Close">
        <svg viewBox="0 0 10 10" width="6" height="6">
          <path d="M1,1 L9,9 M9,1 L1,9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>

      <button class="mac-btn minimize" @click="minimizeWindow" title="Minimize">
        <svg viewBox="0 0 10 10" width="6" height="6">
          <path d="M1,5 L9,5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>

      <button class="mac-btn maximize" @click="toggleMaximize" title="Maximize">
        <svg viewBox="0 0 10 10" width="6" height="6">
          <path d="M1,5 L9,5 M5,1 L5,9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>
      
    </div>

    <div class="title-area">
      <span class="app-icon">📸</span>
      <span class="app-name">Nikon Framer</span>
    </div>

    <div class="spacer"></div>

  </header>
</template>

<style scoped>
.window-title-bar {
  height: 38px; /* macOS 标题栏通常稍微高一点点 */
  background-color: #121212; /* 或者 #181818 */
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  flex-shrink: 0;
  padding: 0 16px; /* 两侧留白 */
  z-index: 9999;
  position: relative;
}

/* --- 左侧红绿灯区域 --- */
.window-controls {
  display: flex;
  align-items: center;
  gap: 8px; /* 按钮间距 */
  width: 70px; /* 固定宽度，方便布局 */
  height: 100%;
  -webkit-app-region: no-drag;
}

.mac-btn {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  padding: 0;
  outline: none;
  transition: transform 0.1s, filter 0.1s;
  
  /* 默认文字/图标颜色为黑色半透明 */
  color: rgba(0, 0, 0, 0.6);
}

/* 按钮颜色定义 */
.mac-btn.close { background-color: #ff5f56; border: 0.5px solid #e0443e; }
.mac-btn.minimize { background-color: #ffbd2e; border: 0.5px solid #dea123; }
.mac-btn.maximize { background-color: #27c93f; border: 0.5px solid #1aab29; }

/* 悬停变亮一点 */
.mac-btn:hover { filter: brightness(1.1); }
.mac-btn:active { transform: scale(0.95); filter: brightness(0.9); }

/* --- 核心交互：Hover 时显示符号 --- */
/* 默认隐藏图标 */
.mac-btn svg { opacity: 0; transition: opacity 0.1s; }

/* 当鼠标移入整个 controls 区域时，显示所有按钮的图标 (这是 macOS 的经典行为) */
.window-controls:hover .mac-btn svg { opacity: 1; }

/* --- 中间标题区域 --- */
.title-area {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.85em;
  font-weight: 500;
  color: #ccc;
  
  /* 绝对居中 */
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  
  /* 穿透点击，保证拖拽 */
  pointer-events: none;
  opacity: 0.8;
}

.spacer {
  width: 70px; /* 与左侧 controls 等宽，保持平衡 */
  pointer-events: none;
}
</style>