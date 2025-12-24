<script setup>
// 引入所有组件
import ControlPanel from './components/ControlPanel.vue';
import FileList from './components/FileList.vue';
import StatusBar from './components/StatusBar.vue';
import PreviewModal from './components/PreviewModal.vue';
import DebugTools from './components/DebugTools.vue';

// 暂时还没写逻辑，先放个空壳
</script>

<template>
  <h1>NIKON <span>Z</span> FRAMER</h1>

  <div class="control-group">
    <ControlPanel />
    
    <FileList />

    <button id="start-batch-btn">开始批处理 (Start Batch)</button>
  </div>
  
  <StatusBar />

  <PreviewModal />

  <DebugTools />
</template>

<style scoped>
/* 这里可以写针对 App 布局的特定样式，目前用全局样式的就够了 */
</style>