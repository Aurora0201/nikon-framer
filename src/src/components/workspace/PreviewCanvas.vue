<script setup>
import { ref, computed, watch, nextTick } from 'vue';
import { store } from '../../store/index.js';
import LoadingSpinner from '../common/LoadingSpinner.vue';

const props = defineProps({
  displayData: { type: Object, required: true },
  isBusy: { type: Boolean, default: false }
});

const emit = defineEmits(['img-load', 'img-error']);
const viewportRef = ref(null);
const wrapperRef = ref(null);

const activeLayerComponent = computed(() => {
  if (props.displayData.type === 'result') return null;
  
  const comp = store.currentModeConfig?.layerComponent;
  
  return comp;
});

// --- 缩放逻辑 ---
const transformState = ref({ scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0 });

const imageStyle = computed(() => ({
  transform: `translate(${transformState.value.pointX}px, ${transformState.value.pointY}px) scale(${transformState.value.scale})`,
  cursor: transformState.value.panning ? 'grabbing' : 'grab',
  transition: transformState.value.panning ? 'none' : 'transform 0.1s ease-out'
}));

// 🟢 [增加调试日志] 的 fitToScreen
const fitToScreen = async () => {
  await nextTick(); // 等待 DOM 更新
  
  const container = viewportRef.value;
  const wrapper = wrapperRef.value;
  
  if (!container || !wrapper) return;

  // 容器尺寸 (黑色区域)
  const cW = container.clientWidth;
  const cH = container.clientHeight;
  
  // 内容尺寸 (图片原始尺寸)
  const wW = wrapper.offsetWidth;
  const wH = wrapper.offsetHeight;

  if (wW === 0 || wH === 0) return;

  const scaleX = cW / wW;
  const scaleY = cH / wH;
  
  // 计算缩放 (留 10% 边距)
  let bestFit = Math.min(scaleX, scaleY, 1) * 0.9;
  bestFit = Math.max(0.01, bestFit); // 允许缩得更小，防止超大图无法显示
  
  transformState.value = {
    scale: bestFit,
    panning: false,
    pointX: 0,
    pointY: 0,
    startX: 0,
    startY: 0
  };
  
};

const onImgLoad = (e) => {
  // 🟢 1. 获取图片真实尺寸
  const img = e.target;
  const naturalWidth = img.naturalWidth || img.width;
  const naturalHeight = img.naturalHeight || img.height;

  // 确保调用了 store 更新
  store.updateImageDimensions(naturalWidth, naturalHeight);

  emit('img-load');
  fitToScreen(); // 此时 wrapper 宽度已恢复正常，缩放会生效
};

// ... (交互事件保持不变，handleWheel, startDrag, resetView 等直接复制旧代码) ...
const handleWheel = (e) => {
  e.preventDefault();
  const zoomIntensity = 0.1;
  const direction = e.deltaY > 0 ? -1 : 1;
  let newScale = transformState.value.scale + (direction * zoomIntensity);
  newScale = Math.min(Math.max(0.1, newScale), 5);
  transformState.value.scale = newScale;
};
const startDrag = (e) => {
  if (e.button !== 0) return;
  transformState.value.panning = true;
  transformState.value.startX = e.clientX - transformState.value.pointX;
  transformState.value.startY = e.clientY - transformState.value.pointY;
};
const onDrag = (e) => {
  if (!transformState.value.panning) return;
  e.preventDefault();
  transformState.value.pointX = e.clientX - transformState.value.startX;
  transformState.value.pointY = e.clientY - transformState.value.startY;
};
const stopDrag = () => { transformState.value.panning = false; };
const resetView = () => fitToScreen();
watch(() => props.displayData.url, () => { transformState.value.pointX = 0; transformState.value.pointY = 0; });
defineExpose({ resetView });
</script>

<template>
  <div class="preview-area"
    @wheel="handleWheel" @mousedown="startDrag" @mousemove="onDrag" @mouseup="stopDrag" @mouseleave="stopDrag" @dblclick="resetView">
    
    <Transition name="fade">
      <LoadingSpinner v-if="isBusy" text="处理中..." mode="overlay" />
    </Transition>

    <div v-if="displayData.url" class="viewport-container" ref="viewportRef">
      
      <div class="image-wrapper" :style="imageStyle" ref="wrapperRef">
        
        <component :is="activeLayerComponent" v-if="activeLayerComponent" />

        <img 
          :src="displayData.url" 
          class="main-img" 
          alt="Preview" 
          @load="onImgLoad" 
          @error="$emit('img-error', $event)"
          draggable="false" 
        />
      </div>
      
      <div v-if="!isBusy" class="status-badge" :class="displayData.type">
        {{ displayData.text }}
      </div>
    </div>

    <div v-else class="placeholder-preview">
      <div style="font-size: 3em; margin-bottom: 20px;">🖼️</div>
      <div>选择照片以预览</div>
    </div>
  </div>
</template>

<style scoped>
/* 背景等样式保持不变 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.preview-area { flex: 1; background: #1a1a1a; position: relative; overflow: hidden; display: flex; justify-content: center; align-items: center; user-select: none; }
.viewport-container { width: 100%; height: 100%; display: flex; justify-content: center; align-items: center; position: relative; overflow: hidden; }

/* 🟢 [关键修复 1] 彻底移除尺寸限制 */
/* 让 Wrapper 诚实地变成图片原本的大小（比如 6000x4000） */
/* 这样 JS 算出来的 Scale 才是准确的 (比如 0.15) */
.image-wrapper {
  position: relative;
  width: max-content; /* 强制撑开，不换行 */
  height: max-content;
  display: flex;
  justify-content: center;
  align-items: center;
  
  transform-origin: center center;
  /* will-change: transform; */

  /* 🟢 告诉浏览器使用高质量缩放 (主要针对 Chrome/Edge) */
  image-rendering: -webkit-optimize-contrast; /* 旧版 Chrome */
  image-rendering: high-quality; /* 现代浏览器标准 */
  
  /* 防止某些浏览器默认使用了 pixelated (像素化) */
  image-rendering: auto;
}

/* 🟢 [关键修复 2] 图片还原真身 */
.main-img {
  display: block;
  /* ❌ 删掉 max-width/height */
  /* 让图片以原始分辨率渲染，JS 负责把它缩放回屏幕内 */
  width: auto;
  height: auto; 
  
  box-shadow: 0 50px 100px rgba(0,0,0,0.5); /* 阴影大一点，因为图片本身很大 */
  pointer-events: none; 
  
}

.status-badge { position: absolute; top: 20px; right: 20px; padding: 6px 12px; border-radius: 4px; font-size: 0.8em; font-weight: bold; color: white; z-index: 10; pointer-events: none; }
.status-badge.preset { background: rgba(100, 100, 100, 0.8); }
.status-badge.result { background: rgba(16, 185, 129, 0.9); }
.placeholder-preview { color: #444; text-align: center; }
</style>