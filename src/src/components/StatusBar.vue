<script setup>
import { computed } from 'vue';
import { store } from '../store.js'; // 🟢 引入全局状态

// 计算状态颜色
const statusColor = computed(() => {
  switch (store.statusType) {
    case 'error': return '#ff4444';
    case 'success': return '#4caf50';
    case 'loading': return '#FF9800';
    default: return '#ccc'; // 深色模式下的默认文字颜色
  }
});
</script>

<template>
  <div id="status-container" class="status-panel">
    
    <div class="spinner-wrapper">
      <div 
        id="loading-spinner" 
        class="spinner" 
        v-show="store.isProcessing || store.statusType === 'loading'"
      ></div>
    </div>

    <div class="status-content">
      
      <div 
        id="status" 
        class="status-text" 
        :style="{ color: statusColor }"
      >
        {{ store.statusText }}
      </div>

      <div 
        id="progress-container" 
        class="progress-area" 
        v-if="store.progress.total > 0"
      >
        <div class="progress-track">
          <div 
            id="progress-fill" 
            class="progress-fill"
            :style="{ width: store.progress.percent + '%' }"
          ></div>
        </div>
        <div id="progress-text" class="progress-info">
          {{ store.progress.current }} / {{ store.progress.total }} ({{ store.progress.percent }}%)
        </div>
      </div>
      
    </div>
  </div>
</template>

<style scoped>
/* 这里不需要写样式，因为它会继承 assets/css/styles.css */
/* 如果你需要微调，可以在这里写 */
</style>