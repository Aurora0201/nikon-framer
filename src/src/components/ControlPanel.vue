<script setup>
import { onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

// --- 计算属性 ---

// 控制阴影滑块的显示：只有 "GaussianBlur" 风格才需要阴影
const showShadowControl = computed(() => {
  return store.settings.style === 'GaussianBlur';
});

// --- 方法 ---

// 加载字体列表
const loadFonts = async () => {
  try {
    // 假设后端命令叫 'get_font_list'，返回字符串数组
    // 如果你还没有写这个后端命令，这里会报错，catch 会捕获它
    const fonts = await invoke('get_font_list');
    if (fonts && fonts.length > 0) {
      store.setFonts(fonts);
      // 如果当前选中的字体不在列表里，重置为第一个
      if (!fonts.includes(store.settings.font) && store.settings.font !== 'Default') {
        store.settings.font = fonts[0];
      }
    }
  } catch (e) {
    console.warn("无法加载字体列表 (可能是后端命令未实现):", e);
    // 放入一些假数据用于调试 UI
    store.setFonts(['Arial', 'Microsoft YaHei', 'Segoe UI', 'San Francisco']);
  }
};

// 刷新字体按钮点击
const refreshFonts = async () => {
  const btn = document.getElementById('refresh-fonts-btn');
  btn.classList.add('rotating'); // 加个旋转动画类（需CSS支持）
  await loadFonts();
  setTimeout(() => btn.classList.remove('rotating'), 500);
};

// --- 生命周期 ---
onMounted(() => {
  loadFonts();
});
</script>

<template>
  <div class="panel-section">
    <div class="control-item">
      <label for="style-select">边框样式 / Frame Style</label>
      <select id="style-select" v-model="store.settings.style">
        <option value="BottomWhite">简约白底 (Bottom White)</option>
        <option value="GaussianBlur">高斯模糊 (Atmosphere)</option>
      </select>
    </div>

    <div class="control-item">
      <label for="font-select">字体文件 / Font</label>
      <div class="font-row">
        <select id="font-select" v-model="store.settings.font">
          <option value="Default">默认 (Default)</option>
          <option v-for="font in store.fontList" :key="font" :value="font">
            {{ font }}
          </option>
        </select>
        <button 
          id="refresh-fonts-btn" 
          class="icon-btn" 
          title="刷新字体列表"
          @click="refreshFonts"
        >
          🔄
        </button>
      </div>
    </div>

    <div class="control-item">
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
      v-show="showShadowControl"
      class="control-item fade-in"
    >
      <div class="slider-header">
        <label for="shadow-input">阴影强度 / Shadow</label>
        <span class="value-display">{{ store.settings.shadowIntensity }}</span>
      </div>
      <input 
        type="range" 
        id="shadow-input" 
        min="0" 
        max="2" 
        step="0.1" 
        v-model.number="store.settings.shadowIntensity" 
        style="width: 100%; cursor: pointer;"
      >
    </div>
  </div>
</template>

<style scoped>
/* 补充一些局部样式优化 */
.control-item {
  margin-bottom: 15px;
}
.font-row {
  display: flex;
  gap: 8px;
}
.icon-btn {
  padding: 0 10px;
  cursor: pointer;
}
.slider-header {
  display: flex; 
  justify-content: space-between; 
  align-items: center; 
  margin-bottom: 5px;
}
.value-display {
  font-size: 0.9em; 
  color: var(--nikon-yellow, #ffe100);
}

/* 简单的旋转动画 */
.rotating {
  animation: spin 0.5s linear;
}
@keyframes spin {
  100% { transform: rotate(360deg); }
}
.fade-in {
  animation: fadeIn 0.3s ease-in;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-5px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>