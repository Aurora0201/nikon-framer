<script setup>
import { watch, ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../../store.js';

// ... (逻辑部分保持完全不变) ...
const checkPreviewStatus = async () => {
  if (!store.activeFilePath || !store.activePresetId) return;
  try {
    const existingPath = await invoke('check_output_exists', {
      filePath: store.activeFilePath,
      style: store.activePresetId 
    });
    if (existingPath) {
      store.markFileProcessed(store.activeFilePath, existingPath);
    } else {
      store.clearProcessedStatus(store.activeFilePath);
    }
  } catch (e) {
    console.error("检查文件存在性失败:", e);
  }
};

watch([() => store.activeFilePath, () => store.activePresetId], () => checkPreviewStatus(), { immediate: true });
watch(() => store.isProcessing, (newVal, oldVal) => { if (oldVal === true && newVal === false) checkPreviewStatus(); });

// --- 缩放拖拽逻辑 (保持不变) ---
const transformState = ref({
  scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0
});

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

watch(() => store.previewSource.url, () => { resetView(); });

const handleImgError = (e) => {
  e.target.style.backgroundColor = '#333';
  e.target.alt = "图片丢失";
};
</script>

<template>
  <div class="workspace-header">
    <span class="tab active">👁️ 实时预览</span>
    <span class="tab">⚙️ 导出设置</span>
    <button class="reset-btn" @click="resetView" title="重置视图">↺</button>
  </div>

  <div 
    class="preview-area"
    @wheel="handleWheel"
    @mousedown="startDrag"
    @mousemove="onDrag"
    @mouseup="stopDrag"
    @mouseleave="stopDrag"
    @dblclick="resetView"
  >
    <div v-if="store.previewSource.url" class="viewport-container">
      <div class="image-wrapper" :style="imageStyle">
        <img 
          :src="store.previewSource.url" 
          class="main-img" 
          alt="Preview" 
          @error="handleImgError"
          draggable="false" 
        />
      </div>
      
      <div class="status-badge" :class="store.previewSource.type">
        {{ store.previewSource.text }}
      </div>
    </div>

    <div v-else class="placeholder-preview">
      <div style="font-size: 3em; margin-bottom: 20px;">🖼️</div>
      <div>选择照片以预览</div>
    </div>
  </div>

  <div class="controls-area">
    <div class="control-row" v-if="store.activePresetId">
        <label style="color: #666; font-size: 0.75em;">
            当前模式: {{ store.activePresetId }}
        </label>
    </div>
    
    <div class="control-row" v-else>
       <label style="color: #444; font-size: 0.75em;">暂无参数配置</label>
    </div>
  </div>
</template>

<style scoped>
/* ... (Header 样式保持不变) ... */
.workspace-header {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  background: #151515;
  border-bottom: 1px solid #333;
  gap: 10px;
}
.reset-btn { margin-left: auto; background: transparent; border: none; color: #888; cursor: pointer; font-size: 1.2em; }
.reset-btn:hover { color: #fff; }
.tab { padding: 4px 12px; font-size: 0.85em; color: #888; cursor: pointer; }
.tab.active { color: #fff; background: #333; border-radius: 4px; }

.preview-area {
  flex: 1; 
  background: #1a1a1a;
  position: relative;
  overflow: hidden;
  display: flex;
  justify-content: center;
  align-items: center;
  /* 背景纹理 */
  background-image: 
    linear-gradient(45deg, #222 25%, transparent 25%), 
    linear-gradient(-45deg, #222 25%, transparent 25%), 
    linear-gradient(45deg, transparent 75%, #222 75%), 
    linear-gradient(-45deg, transparent 75%, #222 75%);
  background-size: 20px 20px;
  background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
  user-select: none; 
}

.viewport-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
}

.image-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  transform-origin: center center;
  will-change: transform;
  /* 确保 wrapper 本身占满空间，方便计算中心 */
  width: 100%;
  height: 100%;
}

.main-img {
  /* 🟢 修改点 2: 调整图片尺寸 */
  /* 改为 80% (或 85%)，这样四周会有留白，不会撑满 */
  max-width: 80%;
  max-height: 80%;
  
  object-fit: contain;
  box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  pointer-events: none; 
}

.status-badge {
  position: absolute;
  top: 20px;
  right: 20px;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 0.8em;
  font-weight: bold;
  color: white;
  z-index: 10;
  pointer-events: none;
}
.status-badge.preset { background: rgba(100, 100, 100, 0.8); }
.status-badge.result { background: rgba(16, 185, 129, 0.9); }

.placeholder-preview { color: #444; text-align: center; }

.controls-area {
  height: 100px; /* 高度可以稍微调小一点，因为内容少了 */
  background: #111;
  border-top: 1px solid #333;
  padding: 20px;
}
.control-row { margin-bottom: 15px; }
label { display: block; color: #888; font-size: 0.85em; margin-bottom: 8px; }
</style>