<script setup>
import { ref, watch, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { store } from '../store.js';

// --- 状态定义 ---
const previewSrc = ref(null);
const isLoading = ref(false);
const errorMsg = ref(null);
const currentFilePath = ref(null);

// 视图变换状态
const view = reactive({
  scale: 1,
  x: 0,
  y: 0,
  isDragging: false,
  startX: 0,
  startY: 0
});

// --- 核心逻辑 ---

function buildDebugContext() {
  const s = store.settings.style;
  if (s === 'BottomWhite') return { style: 'BottomWhite' };
  if (s === 'Master') return { style: 'Master' };
  if (s === 'GaussianBlur') {
    return { 
      style: 'GaussianBlur', 
      shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0 
    };
  }
  return { style: 'BottomWhite' };
}

const resetView = () => {
  view.scale = 1;
  view.x = 0;
  view.y = 0;
};

const generatePreview = async (path) => {
  if (!path) return;
  isLoading.value = true;
  errorMsg.value = null;
  currentFilePath.value = path;
  
  // 切换图片时复位视图，体验更好
  resetView();

  try {
    const context = buildDebugContext();
    const base64Image = await invoke('debug_process_image', {
      path: path,
      context: context
    });
    previewSrc.value = base64Image;
  } catch (err) {
    console.error("Debug Error:", err);
    errorMsg.value = typeof err === 'string' ? err : JSON.stringify(err);
  } finally {
    isLoading.value = false;
  }
};

const handleFileSelect = async () => {
  try {
    const file = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['jpg', 'png', 'jpeg', 'webp'] }]
    });
    if (file) {
      const path = file.path || file; 
      generatePreview(path);
    }
  } catch (e) {
    console.error("选择文件失败:", e);
  }
};

watch(() => store.settings, () => {
  if (currentFilePath.value && !isLoading.value) {
    generatePreview(currentFilePath.value);
  }
}, { deep: true });

// --- 交互事件处理 ---

const handleWheel = (e) => {
  if (!previewSrc.value) return;
  e.preventDefault();
  const zoomIntensity = 0.1;
  const delta = e.deltaY > 0 ? -zoomIntensity : zoomIntensity;
  const newScale = view.scale + delta;
  view.scale = Math.min(Math.max(0.1, newScale), 10);
};

const startDrag = (e) => {
  if (!previewSrc.value) return;
  e.preventDefault();
  view.isDragging = true;
  view.startX = e.clientX - view.x;
  view.startY = e.clientY - view.y;
};

const onDrag = (e) => {
  if (!view.isDragging) return;
  e.preventDefault();
  view.x = e.clientX - view.startX;
  view.y = e.clientY - view.startY;
};

const stopDrag = () => {
  view.isDragging = false;
};

// --- 测试工具 ---
const runShadowTest = async () => {
  resetView();
  isLoading.value = true;
  try {
    const res = await invoke('debug_shadow_grid');
    previewSrc.value = res;
    currentFilePath.value = null;
  } catch (e) { alert(e); } finally { isLoading.value = false; }
};

const runWeightTest = async () => {
  resetView();
  isLoading.value = true;
  try {
    const res = await invoke('debug_weight_grid');
    previewSrc.value = res;
    currentFilePath.value = null;
  } catch (e) { alert(e); } finally { isLoading.value = false; }
};

const toggleDragState = () => {
  store.isDragging = !store.isDragging;
};
</script>

<template>
  <div class="debug-container">
    <div class="debug-header">
      <label>🛠️ 实时调试台 / Instant Preview</label>
      <div class="header-controls">
        <span class="hint" v-if="previewSrc">🖱️ 滚轮缩放 · 拖拽移动 · 双击复位</span>
        <button @click="handleFileSelect" :disabled="isLoading" class="primary-btn">
          {{ isLoading ? '处理中...' : '📸 选择照片' }}
        </button>
      </div>
    </div>

    <div 
      class="preview-viewport"
      @wheel="handleWheel"
      @mousedown="startDrag"
      @mousemove="onDrag"
      @mouseup="stopDrag"
      @mouseleave="stopDrag"
      @dblclick="resetView"
      :class="{ 'grabbing': view.isDragging, 'grab': previewSrc && !view.isDragging }"
    >
      <div v-if="!previewSrc && !isLoading" class="placeholder" @click="handleFileSelect">
        <div style="font-size: 2em; margin-bottom: 10px;">🖼️</div>
        <div>点击选择一张图片<br>进行实时调参预览</div>
      </div>

      <div v-if="isLoading" class="loading-overlay">
        <div class="spinner"></div>
      </div>

      <img 
        v-if="previewSrc" 
        :src="previewSrc" 
        class="preview-img" 
        :style="{ 
          transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})` 
        }"
        @dragstart.prevent
      />

      <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
      
      <div v-if="previewSrc" class="scale-indicator">
        {{ Math.round(view.scale * 100) }}%
      </div>
    </div>

    <div class="tools-bar">
      <div class="btn-group">
        <button @click="runShadowTest" class="secondary-btn">🐞 阴影网格</button>
        <button @click="runWeightTest" class="secondary-btn">🐞 字体粗细</button>
        <button @click="toggleDragState" class="secondary-btn danger-btn">
          ⚡ 强制高亮 ({{ store.isDragging ? 'ON' : 'OFF' }})
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.debug-container {
  margin-top: 30px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 15px;
  color: #eee;
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.hint {
  font-size: 0.75em;
  color: #666;
  margin-right: 15px;
  user-select: none;
}

/* 🟢 关键样式修改：视口 */
.preview-viewport {
  width: 100%;
  height: 460px;        /* 1. 固定高度 */
  background: #0b0b0b;  /* 深色背景 */
  border: 1px solid #333;
  border-radius: 4px;
  position: relative;   /* 2. 相对定位，作为绝对定位子元素的参考 */
  overflow: hidden;     /* 3. 隐藏溢出内容 */
  
  /* Flex 用于居中 placeholder 和 loading，图片不依赖 flex */
  display: flex;
  justify-content: center;
  align-items: center;
}

/* 鼠标手势 */
.grab { cursor: grab; }
.grabbing { cursor: grabbing; }

/* 🟢 关键样式修改：图片 */
.preview-img {
  position: absolute;   /* 4. 绝对定位：脱离文档流，不占空间 */
  top: 0; left: 0;
  width: 100%;          /* 5. 填满容器宽度 */
  height: 100%;         /* 6. 填满容器高度 */
  object-fit: contain;  /* 7. 保持比例显示，不拉伸 */
  
  transition: transform 0.05s linear; /* 极短的过渡让拖拽更跟手 */
  user-select: none;    /* 禁止选中 */
  will-change: transform; /* 性能优化 */
}

.scale-indicator {
  position: absolute;
  bottom: 10px;
  right: 10px;
  background: rgba(0,0,0,0.6);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.7em;
  color: #ccc;
  pointer-events: none;
  z-index: 5;
}

.placeholder {
  color: #444;
  text-align: center;
  cursor: pointer;
  padding: 20px;
  user-select: none;
  z-index: 1; /* 确保在底层 */
}
.placeholder:hover { color: #666; }

.primary-btn {
  background: #444;
  color: white;
  border: 1px solid #555;
  padding: 5px 12px;
  border-radius: 4px;
  cursor: pointer;
}
.primary-btn:hover { background: #555; }
.primary-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.loading-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10;
}
.spinner {
  width: 30px; height: 30px;
  border: 3px solid var(--nikon-yellow, #ffe100);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.error-msg {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  color: #ff4d4d;
  padding: 10px;
  background: rgba(50, 0, 0, 0.9);
  text-align: center;
  font-size: 0.8em;
  z-index: 20;
}

.tools-bar {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed #333;
}
.btn-group { display: flex; gap: 10px; flex-wrap: wrap; }
.secondary-btn {
  background: transparent;
  border: 1px dashed #555;
  color: #888;
  padding: 4px 10px;
  font-size: 0.75em;
  cursor: pointer;
  border-radius: 4px;
}
.secondary-btn:hover { border-color: #888; color: #ccc; }
.danger-btn { border-color: #833; color: #c55; }
.danger-btn:hover { border-color: #f55; color: #f55; }
</style>