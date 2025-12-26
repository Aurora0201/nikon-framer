<script setup>
import { computed } from 'vue';
import { store } from '../store.js';

// --- 计算属性 ---

// 🟢 核心逻辑：判断当前模式是否支持自定义字体
// 目前所有模式（BottomWhite, GaussianBlur, Master）都由后端“硬编码”指定最佳字体。
// 因此，这里返回 false，界面上会隐藏字体选择器。
// 未来如果开发了 'Custom' 模式，只需将其加入数组即可。
const supportsCustomFont = computed(() => {
  const customModes = ['Custom']; // 预留给未来的扩展
  return customModes.includes(store.settings.style);
});

// 控制阴影滑块的显示：只有 "GaussianBlur" 风格才需要
const showShadowControl = computed(() => {
  return store.settings.style === 'GaussianBlur';
});


</script>

<template>
  <div class="panel-section">
    <div class="control-item">
      <label for="style-select">边框样式 / Frame Style</label>
      <select id="style-select" v-model="store.settings.style">
        <option value="BottomWhite">简约白底 (Gallery)</option>
        <option value="GaussianBlur">高斯模糊 (Atmosphere)</option>
        <option value="Master">大师模式 (Master Series)</option>
      </select>
    </div>

    <div class="control-item" v-if="supportsCustomFont">
      <label for="font-select">字体文件 / Font</label>
      <div class="font-row">
        <select id="font-select" v-model="store.settings.font">
          <option value="Default">默认 (Default)</option>
          <option v-for="font in store.fontList" :key="font" :value="font">
            {{ font }}
          </option>
        </select>
      </div>
    </div>

    <div class="control-item" v-if="supportsCustomFont">
      <label for="font-weight-select">字体粗细 / Font Weight</label>
      <select id="font-weight-select" v-model="store.settings.weight">
        <option value="Normal">正常 (Normal)</option>
        <option value="Medium">中粗 (Medium)</option>
        <option value="Bold">加粗 (Bold)</option>
        <option value="ExtraBold">特粗 (Extra Bold)</option>
      </select>
    </div>
    
    <div 
      id="shadow-control-group" 
      v-if="showShadowControl"
      class="control-item fade-in"
    >
      <div class="slider-header">
        <label for="shadow-input">阴影强度 / Shadow</label>
        <span class="value-display">{{ store.settings.shadowIntensity }}%</span>
      </div>
      <input 
        type="range" 
        id="shadow-input" 
        min="0" 
        max="100" 
        step="5" 
        v-model.number="store.settings.shadowIntensity" 
        style="width: 100%; cursor: pointer;"
      >
    </div>
  </div>
</template>

<style scoped>
.control-item {
  margin-bottom: 20px;
}

label {
  display: block;
  margin-bottom: 8px;
  font-size: 0.9em;
  color: #ccc;
  font-weight: 500;
}

/* 🟢 核心修复：下拉框样式 */
select {
  width: 100%;
  padding: 10px 12px;
  padding-right: 30px; /* 右侧留出空间给箭头 */
  border-radius: 6px;
  border: 1px solid #444;
  
  /* 1. 去掉默认样式 */
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  
  /* 2. 定义背景颜色 和 箭头图标 (Nikon黄) */
  background-color: #333;
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23FFE100' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  
  /* 3. 禁止平铺，定位到右侧居中 */
  background-repeat: no-repeat;
  background-position: right 10px center;
  background-size: 16px;
  
  color: white;
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
  cursor: pointer;
}

/* 鼠标悬停和聚焦时的效果 */
select:hover {
  border-color: #666;
  background-color: #3a3a3a;
}

select:focus {
  border-color: var(--nikon-yellow, #ffe100);
  background-color: #2a2a2a;
}

/* Range 滑块样式 */
input[type="range"] {
  width: 100%;
  accent-color: var(--nikon-yellow, #ffe100);
  cursor: pointer;
  margin-top: 5px;
}

.font-row {
  display: flex;
  gap: 8px;
}

.slider-header {
  display: flex; 
  justify-content: space-between; 
  align-items: center; 
  margin-bottom: 8px;
}

.value-display {
  font-size: 0.85em; 
  color: var(--nikon-yellow, #ffe100);
  background: rgba(255, 225, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.fade-in {
  animation: fadeIn 0.3s ease-in-out;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-5px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>