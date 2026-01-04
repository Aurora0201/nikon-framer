<script setup>
import { ref, computed, watch } from 'vue';
import LoadingSpinner from '../common/LoadingSpinner.vue';

const props = defineProps({
  displayData: { type: Object, required: true }, // frozenDisplay
  isBusy: { type: Boolean, default: false }
});

const emit = defineEmits(['img-load', 'img-error']);

// --- 缩放与拖拽逻辑 (纯 UI 交互) ---
const transformState = ref({ scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0 });

const imageStyle = computed(() => ({
  transform: `translate(${transformState.value.pointX}px, ${transformState.value.pointY}px) scale(${transformState.value.scale})`,
  cursor: transformState.value.panning ? 'grabbing' : 'grab',
  transition: transformState.value.panning ? 'none' : 'transform 0.1s ease-out'
}));

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

const resetView = () => {
  transformState.value = { scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0 };
};

// 监听 URL 变化自动重置视图
watch(() => props.displayData.url, () => { resetView(); });

// 暴露 resetView 方法给父组件 (通过 template ref 调用)
defineExpose({ resetView });
</script>

<template>
  <div 
    class="preview-area"
    @wheel="handleWheel"
    @mousedown="startDrag"
    @mousemove="onDrag"
    @mouseup="stopDrag"
    @mouseleave="stopDrag"
    @dblclick="resetView"
  >
    <Transition name="fade">
      <LoadingSpinner v-if="isBusy" text="处理中..." mode="overlay" />
    </Transition>

    <div v-if="displayData.url" class="viewport-container">
      <div class="image-wrapper" :style="imageStyle">
        <img 
          :src="displayData.url" 
          class="main-img" 
          alt="Preview" 
          @load="$emit('img-load')" 
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
/* 原有的 preview-area, viewport-container, image-wrapper, main-img, status-badge 样式 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.preview-area {
  flex: 1; 
  background: #1a1a1a;
  position: relative;
  overflow: hidden;
  display: flex;
  justify-content: center;
  align-items: center;
  background-image: 
    linear-gradient(45deg, #222 25%, transparent 25%), 
    linear-gradient(-45deg, #222 25%, transparent 25%), 
    linear-gradient(45deg, transparent 75%, #222 75%), 
    linear-gradient(-45deg, transparent 75%, #222 75%);
  background-size: 20px 20px;
  background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
  user-select: none; 
}
.viewport-container { width: 100%; height: 100%; display: flex; justify-content: center; align-items: center; position: relative; }
.image-wrapper { display: flex; justify-content: center; align-items: center; width: 100%; height: 100%; will-change: transform; transform-origin: center center; }
.main-img { max-width: 80%; max-height: 80%; object-fit: contain; box-shadow: 0 10px 30px rgba(0,0,0,0.5); pointer-events: none; }
.status-badge { position: absolute; top: 20px; right: 20px; padding: 6px 12px; border-radius: 4px; font-size: 0.8em; font-weight: bold; color: white; z-index: 10; pointer-events: none; }
.status-badge.preset { background: rgba(100, 100, 100, 0.8); }
.status-badge.result { background: rgba(16, 185, 129, 0.9); }
.placeholder-preview { color: #444; text-align: center; }
</style>