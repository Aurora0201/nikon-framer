<script setup>
import { computed, watch } from 'vue';
import { store } from '../../store/index.js';


const dynamicFontSize = computed(() => {
  const imgW = store.imageDimensions?.width;
  // 🟢 从 modeParams 读取
  const scale = store.modeParams.fontScale || 0.05; 
  if (!imgW) return '150px';
  return `${imgW * scale}px`;
});

const dynamicBottom = computed(() => {
  // 🟢 从 modeParams 读取
  const ratio = store.modeParams.bottomRatio || 0.06;
  return `${ratio * 100}%`;
});

</script>

<template>
  <div class="signature-layer-container">
    
    <div class="sig-wrapper" :style="{ bottom: dynamicBottom }" >
        <span class="sig-text" :style="{ fontSize: dynamicFontSize }">
            {{ store.modeParams.text ? store.modeParams.text : '请输入文字' }}
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
  /* bottom: 6%; 距离底部 6% */
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  /* 🧹 已移除调试边框 */


}

.sig-text {
  
  /* 🟢 字体颜色：稍微带一点透明度的白，更有质感 */
  color: rgba(255, 255, 255, 0.95);
  
  font-family: 'Inter Display', system-ui, sans-serif;
  font-weight: 500;
  letter-spacing: 0.05em;
  white-space: nowrap;
  
  /* 🟢 阴影：增加立体感，防止在浅色背景上看不清 */
  text-shadow: 0 4px 12px rgba(0,0,0,0.4);
}
</style>