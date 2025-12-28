<script setup>
import { onMounted } from 'vue';
// 确保路径正确，对应你新建的 layout 文件夹
import ResourcePanel from './components/layouts/ResourcePanel.vue';
import PresetPanel from './components/layouts/PresetPanel.vue';
import WorkspacePanel from './components/layouts/WorkspacePanel.vue';
import StatusBar from './components/layouts/StatusBar.vue';
import { useGlobalEvents } from './composables/useGlobalEvents';

onMounted(() => {
  document.addEventListener('dragstart', (e) => e.preventDefault());
  document.addEventListener('contextmenu', (e) => e.preventDefault());
});

useGlobalEvents();
</script>

<template>
  <div class="app-layout">
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
</template>

<style scoped>
/* 🟢 1. 全局布局容器 */
.app-layout {
  width: 100vw;
  height: 100vh;
  background-color: #121212;
  color: #e0e0e0;
  
  /* 关键布局：纵向排列 */
  display: flex;
  flex-direction: column;
  
  /* 间距控制 */
  padding: 12px; /* 窗口四周留白 */
  gap: 12px;     /* 上下两部分的间距 */
  
  box-sizing: border-box;
  overflow: hidden;
}

/* 🟢 2. 主体视口 (Grid 布局) */
.main-viewport {
  /* 自动占据剩余高度 */
  flex: 1; 
  /* ⚠️ 关键：防止 flex 子元素溢出导致无法滚动 */
  min-height: 0; 
  
  display: grid;
  grid-template-columns: 240px 220px minmax(0, 1fr);
  gap: 2px;
  
  /* 容器样式 */
  background-color: #000;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
}

/* --- 面板通用样式 --- */
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
  /* 既然状态栏不悬浮了，这里不需要额外的 padding-bottom */
  padding-bottom: 0; 
}

.col-1 { background-color: #1a1a1a; border-right: 1px solid #2a2a2a; }
.col-2 { background-color: #141414; border-right: 1px solid #2a2a2a; }
.col-3 { background-color: #0b0b0b; position: relative; }

/* 🟢 3. 沉底状态栏 (Docked Footer) */
.bottom-bar {
  /* 固定高度 */
  height: 50px; 
  flex-shrink: 0; /* 禁止被压缩 */
  
  /* 视觉样式：与上面的主面板保持一致的质感 */
  background-color: #1a1a1a;
  border: 1px solid #333;
  border-radius: 6px;
  
  display: flex;
  align-items: center;
  padding: 0 16px;
  
  /* 不再需要 absolute, backdrop-filter 或 z-index */
}
</style>