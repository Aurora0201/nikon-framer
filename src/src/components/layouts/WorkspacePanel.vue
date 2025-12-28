<script setup>
// 这里引入你之前写的 DebugTool 里的逻辑，或者直接把 DebugTool.vue 作为子组件引入
// 为了演示布局，这里先写结构
import { store } from '../../store.js';
</script>

<template>
  <div class="workspace-header">
    <span class="tab active">👁️ 实时预览</span>
    <span class="tab">⚙️ 导出设置</span>
  </div>

  <div class="preview-area">
    <div class="placeholder-preview">
      <div style="font-size: 3em; margin-bottom: 20px;">🖼️</div>
      <div>选择照片以预览</div>
    </div>
  </div>

  <div class="controls-area">
    <div class="control-row">
      <label>阴影强度 ({{ store.settings.shadowIntensity }}%)</label>
      <input 
        type="range" 
        v-model.number="store.settings.shadowIntensity" 
        min="0" max="100"
      >
    </div>
    </div>
</template>

<style scoped>
.workspace-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  background: #151515;
  border-bottom: 1px solid #333;
  gap: 10px;
}
.tab {
  padding: 4px 12px;
  font-size: 0.85em;
  color: #888;
  cursor: pointer;
}
.tab.active {
  color: #fff;
  background: #333;
  border-radius: 4px;
}

.preview-area {
  flex: 1; /* 自动撑满剩余空间 */
  background: #000;
  position: relative;
  overflow: hidden;
  display: flex;
  justify-content: center;
  align-items: center;
}
.placeholder-preview {
  color: #333; text-align: center;
}

.controls-area {
  height: 120px; /* 固定高度的参数面板 */
  background: #111;
  border-top: 1px solid #333;
  padding: 20px;
}
.control-row {
  margin-bottom: 10px;
}
label { display: block; color: #888; font-size: 0.8em; margin-bottom: 5px; }
input[type="range"] { width: 100%; }
</style>