<script setup>
import { onMounted } from 'vue';
import ResourcePanel from './components/layouts/ResourcePanel.vue';
import PresetPanel from './components/layouts/PresetPanel.vue';
import WorkspacePanel from './components/layouts/WorkspacePanel.vue';
import StatusBar from './components/layouts/StatusBar.vue';
// 🟢 引入新组件
import WindowHeader from './components/layouts/WindowHeader.vue';
import { useGlobalEvents } from './composables/useGlobalEvents';

onMounted(() => {
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
.app-layout {
  width: 100vw;
  height: 100vh;
  background-color: #121212;
  color: #e0e0e0;
  display: flex;
  flex-direction: column;
  padding: 0; 
  overflow: hidden;
}

.content-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  /* 这里的 padding 保持界面的呼吸感 */
  padding: 0 12px 12px 12px;
  gap: 12px;
}

.main-viewport {
  flex: 1; 
  min-height: 0; 
  display: grid;
  grid-template-columns: 280px 220px minmax(0, 1fr);
  gap: 2px;
  background-color: #000;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
  padding-bottom: 0; 
}

.col-1 { background-color: #1a1a1a; border-right: 1px solid #2a2a2a; }
.col-2 { background-color: #141414; border-right: 1px solid #2a2a2a; }
.col-3 { background-color: #0b0b0b; position: relative; }

.bottom-bar {
  height: 55px; 
  flex-shrink: 0; 
  background-color: #1a1a1a;
  border: 1px solid #333;
  border-radius: 6px;
  display: flex;
  align-items: center;
  padding: 0 16px;
}
</style>