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
  width: 100vw;
  height: 100vh;
  
  /* 关键：从 style.css 读取背景色 */
  /* 因为 html/body 是透明的，这里必须上色，否则窗口是透明的 */
  background-color: var(--bg-color); 
  /* background-color: #fff;  */
  color: var(--text-main);
  
  display: flex;
  flex-direction: column;
  padding: 0; 
  border-radius: var(--app-radius);
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.08);
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
  
  /* --- A. 基础材质 --- */
  background: var(--glass-bg, rgba(0, 0, 0, 0.2));
  
  /* 保持磨砂效果，但去除复杂的玻璃光影 */
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  
  /* --- B. 形状 --- */
  border-radius: 16px; 
  overflow: hidden;
  
  /* 🟢 修复：显式定义边框，统一风格 */
  border: 1px solid var(--viewport-border-color, rgba(255, 255, 255, 0.08));

  position: relative;
  z-index: 10; /* 确保层级 */
}

/* 面板通用样式 */
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

/* 🟢 4. 子面板 (必须透明化！) */
/* 以前这里是实色背景，现在必须去掉，否则会挡住 main-viewport 的玻璃效果 */

.col-1 { 
  background-color: transparent; 
  border-right: 1px solid rgba(255, 255, 255, 0.10); 
}

.col-2 { 
  background-color: transparent; 
  border-right: 1px solid rgba(255, 255, 255, 0.10); 
}

.col-3 { 
  background-color: transparent; 
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