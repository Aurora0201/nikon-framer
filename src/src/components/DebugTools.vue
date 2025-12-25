<script setup>
import { store } from '../store.js'; // 🟢 引入 Store

const toggleDragState = () => {
  store.isDragging = !store.isDragging;
  console.log("🔧 [Debug] 手动切换状态，当前 store.isDragging =", store.isDragging);
};
</script>


<template>
  <div class="debug-tools" style="margin-top: 30px; border-top: 1px dashed #ccc; padding-top: 10px;">
      <label style="color: #666; font-size: 0.8em; margin-bottom: 8px; display:block;">开发者工具 / Debug Tools</label>
      <div style="display: flex; gap: 10px;">
        <button id="debug-shadow-btn" class="secondary-btn" style="font-size: 0.8em; border-style: dashed;">
          🐞 阴影测试
        </button>
        <button id="debug-weight-btn" class="secondary-btn" style="font-size: 0.8em; border-style: dashed;">
          🐞 粗细测试
        </button>

        <button 
          @click="toggleDragState" 
          class="secondary-btn" 
          style="font-size: 0.8em; border-style: dashed; border-color: red; color: red;"
        >
          🐞 强制切换拖拽高亮 (Toggle Active)
        </button>
      </div>
  </div>
</template>