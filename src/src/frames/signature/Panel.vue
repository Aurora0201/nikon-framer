<script setup>
import { store } from '../../store/index.js';
</script>

<template>
  <div class="signature-panel">
    
    <div class="input-group">
      <label>✍️ 签名内容 / Content</label>
      <input 
        type="text" 
        v-model="store.modeParams.text" 
        placeholder="在此输入签名..." 
        class="sig-input"
      />
    </div>

    <div class="input-group">
      <label>
        <span>大小 / Size</span>
        <span class="value-tag" v-if="store.modeParams.fontScale">
          {{ (store.modeParams.fontScale * 100).toFixed(1) }}%
        </span>
      </label>
      <input 
        type="range" 
        min="0.01" 
        max="0.20" 
        step="0.005"
        v-model.number="store.modeParams.fontScale" 
        class="range-input"
      />
    </div>

    <div class="input-group">
      <label>
        <span>位置 / Position</span>
        <span class="value-tag" v-if="store.modeParams.bottomRatio">
          {{ (store.modeParams.bottomRatio * 100).toFixed(1) }}%
        </span>
      </label>
      <input 
        type="range" 
        min="0.01" 
        max="0.10" 
        step="0.005"
        v-model.number="store.modeParams.bottomRatio" 
        class="range-input"
      />
    </div>

    <p class="hint">设置将自动应用到所有已选图片</p>

  </div>
</template>

<style scoped>
/* 容器布局 */
.signature-panel {
  display: flex;
  flex-direction: column;
  gap: 16px; /* 控件之间的垂直间距 */
  padding: 5px 0;
}

/* 每个输入组 */
.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 标签样式 */
label {
  color: #aaa;
  font-size: 0.9em;
  font-weight: bold;
  display: flex;
  justify-content: space-between; /* 让文字靠左，数值靠右 */
  align-items: center;
}

/* 数值高亮 */
.value-tag {
  color: #4caf50;
  font-family: monospace;
  font-size: 0.9em;
  background: rgba(76, 175, 80, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

/* 文本输入框美化 */
.sig-input {
  background: #222;
  border: 1px solid #444;
  color: #fff;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 1em;
  outline: none;
  transition: all 0.2s ease;
}

.sig-input:focus {
  border-color: #646cff; /* 聚焦时变亮紫/蓝色 */
  background: #2a2a2a;
  box-shadow: 0 0 0 2px rgba(100, 108, 255, 0.2);
}

/* 滑块条美化 (兼容 Webkit) */
.range-input {
  width: 100%;
  height: 6px;
  background: #333;
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

/* 滑块圆钮样式 */
.range-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  background: #bbb;
  border-radius: 50%;
  transition: transform 0.1s, background-color 0.2s;
}

.range-input::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  background: #fff;
}

/* 底部提示文字 */
.hint {
  color: #555;
  font-size: 0.75em;
  margin-top: 10px;
  text-align: center;
  border-top: 1px dashed #333;
  padding-top: 10px;
}
</style>