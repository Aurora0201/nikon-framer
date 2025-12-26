<script setup>
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { store } from '../store.js';

// --- 状态定义 ---
const previewSrc = ref(null);
const isLoading = ref(false);
const errorMsg = ref(null);
const currentFilePath = ref(null); // 记住当前调试的文件路径，以便重复刷新

// --- 核心逻辑 1: 构建后端需要的上下文 ---
// ⚠️ 必须严格匹配后端 Rust Enum (StyleOptions) 的结构
function buildDebugContext() {
  const s = store.settings.style;
  
  // 1. 极简白底
  if (s === 'BottomWhite') return { style: 'BottomWhite' };
  
  // 2. 大师模式
  if (s === 'Master') return { style: 'Master' };
  
  // 3. 高斯模糊 (带参数)
  if (s === 'GaussianBlur') {
    return { 
      style: 'GaussianBlur', 
      shadowIntensity: parseFloat(store.settings.shadowIntensity) || 0.0 
    };
  }
  
  return { style: 'BottomWhite' };
}

// --- 核心逻辑 2: 生成预览 ---
const generatePreview = async (path) => {
  if (!path) return;
  
  isLoading.value = true;
  errorMsg.value = null;
  currentFilePath.value = path; // 记录路径

  try {
    const context = buildDebugContext();
    console.log("🧪 [Debug] 请求预览:", path, context);

    // 调用后端 debug 命令
    const base64Image = await invoke('debug_process_image', {
      path: path,
      context: context
    });

    previewSrc.value = base64Image;
  } catch (err) {
    console.error("❌ Debug Error:", err);
    errorMsg.value = typeof err === 'string' ? err : JSON.stringify(err);
  } finally {
    isLoading.value = false;
  }
};

// --- 交互逻辑 ---

// 1. 选择文件
const handleFileSelect = async () => {
  try {
    const file = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['jpg', 'png', 'jpeg', 'webp'] }]
    });
    
    if (file) {
      // open 插件返回的是对象 { path: "...", name: "..." } 或直接路径字符串，取决于版本
      // Tauri V2 plugin-dialog 通常返回 struct，我们需要 path 字段
      const path = file.path || file; 
      generatePreview(path);
    }
  } catch (e) {
    console.error("选择文件失败:", e);
  }
};

// 2. 监听设置变化 -> 自动刷新预览
// 只要 store.settings 变了（比如拖动了阴影滑块），且当前有图片，就重绘
watch(() => store.settings, () => {
  if (currentFilePath.value && !isLoading.value) {
    console.log("🔄 设置变化，自动刷新预览...");
    generatePreview(currentFilePath.value);
  }
}, { deep: true });

// --- 辅助测试工具 (对应原来的按钮) ---

const runShadowTest = async () => {
  isLoading.value = true;
  try {
    const res = await invoke('debug_shadow_grid'); // 调用后端生成阴影网格
    previewSrc.value = res;
    currentFilePath.value = null; // 清除当前文件路径，因为这是生成的
  } catch (e) {
    alert("阴影测试失败: " + e);
  } finally {
    isLoading.value = false;
  }
};

const runWeightTest = async () => {
  isLoading.value = true;
  try {
    const res = await invoke('debug_weight_grid'); // 调用后端生成字重网格
    previewSrc.value = res;
    currentFilePath.value = null;
  } catch (e) {
    alert("字重测试失败: " + e);
  } finally {
    isLoading.value = false;
  }
};

// 手动切换拖拽状态 (原有逻辑)
const toggleDragState = () => {
  store.isDragging = !store.isDragging;
  console.log("🔧 [Debug] 手动切换状态:", store.isDragging);
};
</script>

<template>
  <div class="debug-container">
    <div class="debug-header">
      <label>🛠️ 实时调试台 / Instant Preview</label>
      <button @click="handleFileSelect" :disabled="isLoading" class="primary-btn">
        {{ isLoading ? '处理中...' : '📸 选择照片 (Pick Image)' }}
      </button>
    </div>

    <div class="preview-viewport">
      <div v-if="!previewSrc && !isLoading" class="placeholder" @click="handleFileSelect">
        <div style="font-size: 2em; margin-bottom: 10px;">🖼️</div>
        <div>点击选择一张图片<br>进行实时调参预览</div>
      </div>

      <div v-if="isLoading" class="loading-overlay">
        <div class="spinner"></div>
      </div>

      <img v-if="previewSrc" :src="previewSrc" class="preview-img" />

      <div v-if="errorMsg" class="error-msg">
        {{ errorMsg }}
      </div>
    </div>

    <div class="info-bar" v-if="currentFilePath">
      正在调试: {{ currentFilePath }}
    </div>

    <div class="tools-bar">
      <label>生成器测试 (Generators):</label>
      <div class="btn-group">
        <button @click="runShadowTest" class="secondary-btn">
          🐞 阴影网格
        </button>
        <button @click="runWeightTest" class="secondary-btn">
          🐞 字体粗细
        </button>
        <button 
          @click="toggleDragState" 
          class="secondary-btn danger-btn"
        >
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
  margin-bottom: 15px;
}

.debug-header label {
  font-weight: bold;
  color: var(--nikon-yellow, #ffe100);
}

.primary-btn {
  background: #444;
  color: white;
  border: 1px solid #555;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}
.primary-btn:hover { background: #555; border-color: #777; }

/* 预览视口 */
.preview-viewport {
  width: 100%;
  min-height: 250px;
  background: #000;
  border: 1px dashed #444;
  border-radius: 6px;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  overflow: hidden;
}

.placeholder {
  color: #555;
  text-align: center;
  cursor: pointer;
  padding: 20px;
}
.placeholder:hover { color: #777; }

.preview-img {
  max-width: 100%;
  max-height: 400px; /* 限制高度，防止把页面撑太长 */
  object-fit: contain;
  box-shadow: 0 5px 20px rgba(0,0,0,0.5);
}

/* Loading */
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

/* 错误信息 */
.error-msg {
  color: #ff4d4d;
  padding: 20px;
  background: rgba(255, 0, 0, 0.1);
  border-radius: 4px;
}

.info-bar {
  margin-top: 8px;
  font-size: 0.75em;
  color: #666;
  font-family: monospace;
  word-break: break-all;
}

/* 底部工具栏 */
.tools-bar {
  margin-top: 20px;
  padding-top: 15px;
  border-top: 1px dashed #333;
}
.tools-bar label {
  display: block;
  font-size: 0.8em;
  color: #666;
  margin-bottom: 8px;
}
.btn-group {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.secondary-btn {
  background: transparent;
  border: 1px dashed #555;
  color: #aaa;
  padding: 5px 10px;
  font-size: 0.8em;
  cursor: pointer;
  border-radius: 4px;
}
.secondary-btn:hover { border-color: #888; color: #fff; }

.danger-btn {
  border-color: #833;
  color: #d66;
}
.danger-btn:hover { border-color: #f55; color: #f55; }
</style>