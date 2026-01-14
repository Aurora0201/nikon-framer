<script setup>
import { onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

// 引入组件
import ResourcePanel from './components/layouts/ResourcePanel.vue';
import PresetPanel from './components/layouts/PresetPanel.vue';
import WorkspacePanel from './components/layouts/WorkspacePanel.vue';
import StatusBar from './components/layouts/StatusBar.vue';
import WindowHeader from './components/layouts/WindowHeader.vue';

// 引入全局逻辑
import { useGlobalEvents } from './composables/useGlobalEvents';

const appWindow = getCurrentWindow();

onMounted(() => {
  // 阻止默认行为，让应用感觉像原生软件
  document.addEventListener('dragstart', (e) => e.preventDefault());
  document.addEventListener('contextmenu', (e) => e.preventDefault());
});

useGlobalEvents();
</script>

<template>
  <div class="app-layout">
    
    <WindowHeader />

    <div class="content-wrapper">
      
      <div class="main-viewport">
        <aside class="panel col-1">
          <ResourcePanel />
        </aside>

        <aside class="panel col-2">
          <PresetPanel />
        </aside>

        <section class="panel col-3">
          <WorkspacePanel />
        </section>
      </div>

      <footer class="bottom-bar">
        <StatusBar />
      </footer>
      
    </div>

  </div>
</template>

<style scoped>
/* 🟢 1. 窗口实体 (The Window Body) */
.app-layout {
  /* 充满容器 (容器 #app 已设置 padding: 2px) */
  width: 100%;
  height: 100%;
  margin: 0;
  
  /* 关键：从 style.css 读取背景色 */
  /* 因为 html/body 是透明的，这里必须上色，否则窗口是透明的 */
  background-color: var(--bg-color); 
  color: var(--text-main);
  
  display: flex;
  flex-direction: column;
  padding: 0; 
  
  /* 关键：从 style.css 读取圆角 (12px) */
  /* 这决定了你整个 APP 窗口的圆润程度 */
  border-radius: var(--app-radius);
  
  /* 关键：裁切溢出，确保窗口四个角是圆的，不会有直角内容漏出来 */
  overflow: hidden;
  
  /* 可选：加一个极细的边框，增强窗口在深色壁纸上的轮廓感 */
  /* 使用 box-shadow inset 替代 border，防止盒模型计算差异导致尺寸跳变 */
  box-shadow: inset 0 0 0 1px var(--window-border, rgba(255, 255, 255, 0.08));
}


[data-theme='light'] .app-layout {
  --window-border: rgba(0, 0, 0, 0.12); 
}

/* 🟢 2. 内容布局层 */
.content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 0 12px 12px 12px;
  gap: 12px;
}

/* 🟢 3. 主视口 (中间黑色的工作区) */
.main-viewport {
  flex: 1; 
  min-height: 0; 
  
  display: grid;
  grid-template-columns: 280px 220px minmax(0, 1fr);
  
  /* --- 变量定义 (默认深色) --- */
  --glass-bg: rgba(0, 0, 0, 0.2);
  --viewport-border-color: rgba(255, 255, 255, 0.08);

  /* --- A. 基础材质 --- */
  background: var(--glass-bg);
  
  /* 保持磨砂效果 */
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  
  /* --- B. 形状 --- */
  border-radius: 16px; 
  overflow: hidden;
  
  /* 🟢 修复：显式定义边框 */
  border: 1px solid var(--viewport-border-color);
  box-shadow: var(--panel-shadow);

  /* --- C. 动画过渡 (解决切换时的闪烁问题) --- */
  transition: background 0.3s ease, border-color 0.3s ease;

  position: relative;
  z-index: 10; 
}

/* Light Mode Overrides for Viewport */
[data-theme='light'] .main-viewport {
  --glass-bg: #FFFFFF; 
  /* 增加不透明度，防止在浅色背景下边框显得过浅 */
  --viewport-border-color: var(--border-color); 
}

/* 面板通用样式 */
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

/* 🟢 4. 子面板 (使用变量区分背景) */

.col-1 { 
  background-color: var(--bg-resource); 
  border-right: 1px solid var(--border-color); 
}

.col-2 { 
  background-color: var(--bg-preset); 
  border-right: 1px solid var(--border-color); 
}

.col-3 { 
  background-color: var(--bg-workspace); 
}

/* 🟢 5. 底部栏容器 (保持透明占位) */
.bottom-bar {
  height: 55px; 
  flex-shrink: 0; 
  background: transparent;
  border: none;
  padding: 0; 
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>