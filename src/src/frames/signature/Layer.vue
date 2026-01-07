<script setup>
import { computed } from 'vue'; // 🟢 新增 ref, onMounted, nextTick
import { store } from '../../store/index.js';


const dynamicFontSize = computed(() => {
  const imgW = store.imageDimensions?.width;
  const scale = store.modeParams.fontScale || 0.05; 
  if (!imgW) return '150px';
  return `${imgW * scale}px`;
});

const dynamicBottom = computed(() => {
  const ratio = store.modeParams.bottomRatio || 0.06;
  return `${ratio * 100}%`;
});


</script>

<template>
  <div class="signature-layer-container">
    
    <div :style="debugLineStyle"></div>

    <div class="sig-wrapper" :style="{ bottom: dynamicBottom }" >
        <span 
          class="sig-text debug-outline" 
          :style="{ fontSize: dynamicFontSize }"
        >
            {{ store.modeParams.text ? store.modeParams.text : '©Masterpiece' }}
        </span>
    </div>

  </div>
</template>

<style scoped>
.signature-layer-container {
  position: absolute;
  top: 0; left: 0;
  width: 100%; height: 100%;
  pointer-events: none;
  z-index: 20;
}

.sig-wrapper {
  position: absolute;
  /* bottom 由 style 绑定控制 */
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.sig-text {
  /* 🟢 1. 消除 CSS 行高导致的偏移，让 CSS 盒子紧贴文字 */
  line-height: 1;
  /* 🟢 字体颜色 */
  color: rgba(255, 255, 255, 0.95);
  
  font-family: 'Inter Display';
  font-weight: 500;
  white-space: nowrap;
}
</style>