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
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.preview-area { 
  flex: 1; 
  /* 🟢 保持 transparent，让父组件的点阵背景透过来 */
  background: transparent; 
  position: relative; 
  overflow: hidden; 
  display: flex; 
  justify-content: center; 
  align-items: center; 
  user-select: none; 
}

.viewport-container { width: 100%; height: 100%; display: flex; justify-content: center; align-items: center; position: relative; overflow: hidden; }

.image-wrapper {
  position: relative;
  width: max-content;
  height: max-content;
  display: flex;
  justify-content: center;
  align-items: center;
  transform-origin: center center;
  image-rendering: -webkit-optimize-contrast;
  image-rendering: high-quality;
  image-rendering: auto;
}

.main-img {
  display: block;
  width: auto;
  height: auto; 
  /* 阴影稍微收敛一点，更精致 */
  box-shadow: 0 20px 60px rgba(0,0,0,0.6); 
  pointer-events: none; 
}

/* =========================================
   🟢 毛玻璃标签 (Glassmorphism Badge) 
   ========================================= */
.status-badge { 
  position: absolute; 
  top: 24px; 
  right: 24px; 
  padding: 8px 16px; 
  
  /* 字体设置 */
  font-size: 0.85em; 
  font-weight: 600; 
  color: #fff; /* Default text color for dark mode badges */
  letter-spacing: 0.5px;
  
  /* 形状 */
  border-radius: 8px; 
  z-index: 10; 
  pointer-events: none; 
  
  /* 🟢 核心毛玻璃效果 */
  backdrop-filter: blur(12px); 
  -webkit-backdrop-filter: blur(12px); 
  
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-top-color: rgba(255, 255, 255, 0.25);
  border-left-color: rgba(255, 255, 255, 0.25);

  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.25);
  
  transition: all 0.3s ease;
}

/* ⚪ 状态 A：编辑预览 */
.status-badge.preset { 
  background: rgba(30, 30, 30, 0.65); 
}
:global([data-theme='light']) .status-badge.preset {
  background: rgba(255, 255, 255, 0.65);
  color: var(--text-main);
  border: 1px solid var(--border-color);
  box-shadow: 0 4px 15px rgba(0,0,0,0.05);
}

/* 🟢 状态 B：结果预览 */
.status-badge.result { 
  background: rgba(16, 185, 129, 0.55); 
  border-color: rgba(16, 185, 129, 0.3);
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.4); 
  text-shadow: 0 1px 2px rgba(0,0,0,0.2);
}
:global([data-theme='light']) .status-badge.result {
  color: #fff; /* Keep green badge text white for contrast */
  text-shadow: none;
}

.placeholder-preview { color: var(--text-sub); text-align: center; }
</style>