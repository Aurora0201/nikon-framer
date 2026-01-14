<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { store } from '../../store'; // 引入 store

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

// 获取当前主题图标
const themeIcon = computed(() => store.theme === 'dark' ? '🌙' : '☀️');
const toggleTheme = () => store.toggleTheme();

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

    <!-- 右侧功能区 (替代原来的 spacer) -->
    <div class="right-controls">
      <button class="theme-btn" @click="toggleTheme" :title="store.theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'">
        <!-- Sun Icon -->
        <svg v-if="store.theme === 'light'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5"></circle>
          <line x1="12" y1="1" x2="12" y2="3"></line>
          <line x1="12" y1="21" x2="12" y2="23"></line>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
          <line x1="1" y1="12" x2="3" y2="12"></line>
          <line x1="21" y1="12" x2="23" y2="12"></line>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
        </svg>
        <!-- Moon Icon -->
        <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
        </svg>
      </button>
    </div>

  </header>
</template>

<style scoped>
.window-title-bar {
  height: 38px; /* macOS 标题栏通常稍微高一点点 */
  background-color: transparent; /* 或者 #181818 */
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  flex-shrink: 0;
  padding: 0 16px; /* 两侧留白 */
  z-index: 9999;
  position: relative;
  width: 100%;
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

.right-controls {
  width: 70px; /* 与左侧 controls 等宽，保持平衡 */
  display: flex;
  justify-content: flex-end;
  align-items: center;
  -webkit-app-region: no-drag; /* 允许点击 */
}

.theme-btn {
  background: transparent;
  border: none;
  color: var(--text-sub); /* 跟随主题色 */
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.theme-btn:hover {
  background: var(--input-bg);
  color: var(--text-main);
}

.spacer {
  display: none; /* Hide spacer if it exists to avoid duplication issues if I failed to replace it properly before */
}
</style>