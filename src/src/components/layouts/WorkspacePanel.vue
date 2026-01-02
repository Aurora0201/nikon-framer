<script setup>
import { watch, ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../../store.js';
import LoadingSpinner from '../common/LoadingSpinner.vue';

// -------------------------------------------------------------
// 🟢 [核心修复] 防止图片跳变的“冻结显示”逻辑
// -------------------------------------------------------------

// 1. 本地状态：增加 presetId 字段，记录当前显示的图属于哪个预设
const frozenDisplay = ref({ 
  url: '', 
  type: 'preset', 
  text: '', 
  presetId: '' // 🟢 新增：记录这张图是哪个预设的 ID
});

// 2. 本地状态：浏览器是否正在下载/解码图片
const imgLoading = ref(false);

// 3. 合并 Loading 状态
const isBusy = computed(() => {
  return store.isProcessing || imgLoading.value || store.isLoadingPresets;
});

// 🟢 4. 智能防抖 Watcher (修复版)
watch(
  () => ({ 
    source: store.previewSource, 
    processing: store.isProcessing,
    switching: store.isLoadingPresets,
    currentId: store.activePresetId // 🟢 监听当前的 ID
  }),
  ({ source, processing, switching, currentId }) => {
    // 拦截一：繁忙状态 (处理中/切换中) -> 冻结
    if (processing || switching) return;

    // 🛡️ 拦截二：防退化机制 (Anti-Downgrade)
    // 逻辑修正：
    // 只有当 [新旧 ID 相同] 时，才不允许从 Result 变回 Preset。
    // 如果 [新旧 ID 不同] (说明用户切了模式)，必须允许更新，否则会显示上一个模式的图。
    const isSamePreset = frozenDisplay.value.presetId === currentId;

    if (
      source.type === 'preset' && 
      frozenDisplay.value.type === 'result' && 
      store.activeFilePath &&
      isSamePreset // 🟢 关键：只有同一个模式下才防抖
    ) {
      // console.log('🛡️ 同模式下触发防退化：保持显示旧结果');
      return; 
    }

    // ✅ 通行：更新画面，并记录当前的 ID
    frozenDisplay.value = { ...source, presetId: currentId };
  },
  { deep: true, immediate: true }
);

// 5. 监听 URL 变化触发前端 Loading (保持不变)
watch(() => frozenDisplay.value.url, (newVal, oldVal) => {
  if (newVal && newVal !== oldVal) {
    imgLoading.value = true;
  }
});

// ... (以下所有代码保持不变：handleImgLoad, checkPreviewStatus, 缩放逻辑等) ...
const handleImgLoad = () => { imgLoading.value = false; };
const handleImgError = (e) => {
  imgLoading.value = false;
  e.target.style.backgroundColor = '#333';
  e.target.alt = "图片丢失";
};

const checkPreviewStatus = async () => {
  if (!store.activeFilePath || !store.activePresetId) return;
  
  // 记录下发起请求时的 ID，防止异步回来后 ID 已经变了
  const currentPath = store.activeFilePath;
  const currentStyle = store.activePresetId;

  try {
    const existingPath = await invoke('check_output_exists', {
      filePath: currentPath,
      style: currentStyle
    });
    
    if (existingPath) {
      // 🟢 使用带 Style 的明确方法
      store.markFileProcessedWithStyle(currentPath, currentStyle, existingPath);
    } else {
      store.clearProcessedStatusWithStyle(currentPath, currentStyle);
    }
  } catch (e) {
    console.error("检查文件存在性失败:", e);
  }
};

watch([() => store.activeFilePath, () => store.activePresetId], () => checkPreviewStatus(), { immediate: true });
watch(() => store.isProcessing, (newVal, oldVal) => { 
  if (oldVal === true && newVal === false) checkPreviewStatus(); 
});

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
watch(() => frozenDisplay.value.url, () => { resetView(); });
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
    <Transition name="fade">
      <LoadingSpinner v-if="isBusy" text="处理中..." mode="overlay" />
    </Transition>

    <div v-if="frozenDisplay.url" class="viewport-container">
      <div class="image-wrapper" :style="imageStyle">
        <img 
          :src="frozenDisplay.url" 
          class="main-img" 
          alt="Preview" 
          @load="handleImgLoad" 
          @error="handleImgError"
          draggable="false" 
        />
        </div>
      
      <div v-if="!isBusy" class="status-badge" :class="frozenDisplay.type">
        {{ frozenDisplay.text }}
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

/* 🟢 添加简单的淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

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