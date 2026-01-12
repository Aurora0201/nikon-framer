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
  border: 1px solid rgba(255, 255, 255, 0.08);
  /* box-shadow: 0 0 30px rgba(0, 0, 0, 0.5);  */
}

/* 🟢 2. 内容布局层 */
.content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  
  /* 这里的 padding 控制内容距离窗口边缘的间距 */
  /* 上0 (因为有标题栏)，左右下 12px */
  padding: 0 12px 12px 12px;
  gap: 12px;
}

/* 🟢 3. 主视口 (中间黑色的工作区) */
.main-viewport {
  flex: 1; 
  min-height: 0; 
  
  display: grid;
  grid-template-columns: 280px 220px minmax(0, 1fr);
  /* gap: 1px; 微调间距 */
  
  background-color: #000;
  border: 1px solid var(--border-color);
  
  /* 这个圆角是内部面板的圆角，可以稍微小一点，或者也用 var(--app-radius) */
  border-radius: var(--app-radius); 
  overflow: hidden;
}

/* 面板通用样式 */
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

.col-1 { background-color: var(--panel-bg); border-right: 1px solid var(--border-color); }
.col-2 { background-color: #141414; border-right: 1px solid var(--border-color); }
.col-3 { background-color: #0b0b0b; position: relative; }

/* 🟢 4. 底部栏容器 (The Invisible Container) */
.bottom-bar {
  /* 只负责布局占位 */
  height: 55px; 
  flex-shrink: 0; 
  
  /* ⚠️ 样式全部移除，变成透明容器 */
  background: transparent;
  border: none;
  padding: 0; 
  
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>