<script setup>
import { store } from './store.js';

// 组件
import ControlPanel from './components/ControlPanel.vue';
import FileList from './components/FileList.vue';
import StatusBar from './components/StatusBar.vue';
import PreviewModal from './components/PreviewModal.vue';
import DebugTools from './components/DebugTools.vue';

// 🟢 引入组合式函数 (Hooks)
import { useGlobalEvents } from './composables/useGlobalEvents.js';
import { useBatchProcess } from './composables/useBatchProcess.js';

// 1. 激活全局事件监听 (一行代码搞定所有拖拽、进度监听)
useGlobalEvents();

// 2. 获取按钮逻辑 (将复杂的 UI 逻辑解耦)
const { 
  handleBatchClick, 
  buttonText, 
  buttonClass, 
  buttonCursor 
} = useBatchProcess();

</script>

<template>
  <h1>NIKON <span>Z</span> FRAMER</h1>

  <div class="control-group">
    <ControlPanel />
    
    <FileList />

    <button 
      id="start-batch-btn"
      @click="handleBatchClick"
      :disabled="!store.isProcessing && store.fileQueue.length === 0"
      :class="buttonClass"
      :style="{ cursor: buttonCursor }"
    >
      {{ buttonText }}
    </button>
  </div>
  
  <StatusBar />
  <PreviewModal />
  <DebugTools />
</template>

<style scoped>
/* 按钮样式依然保留在这里，或者移到全局 styles.css */
button.processing-mode {
  background-color: #666;
  border-color: #555;
  color: #ccc;
  opacity: 0.8;
}

button.can-stop {
  background-color: #3e1f1f;
  border-color: #ff4444;
  color: #ff4444;
  animation: pulse-red 2s infinite;
}

button.can-stop:hover {
  background-color: #ff4444;
  color: white;
}

@keyframes pulse-red {
  0% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0.4); }
  70% { box-shadow: 0 0 0 10px rgba(255, 68, 68, 0); }
  100% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0); }
}
</style>