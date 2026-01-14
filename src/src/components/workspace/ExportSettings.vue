<script setup>
import { computed } from 'vue';
import { store } from '../../store/index.js';
import { open } from '@tauri-apps/plugin-dialog';

// 🟢 辅助函数：从完整路径提取文件夹路径 (纯字符串处理，不依赖 Tauri API 以提升性能)
const getParentDirectory = (filePath) => {
  if (!filePath) return '未选择图片';
  // 兼容 Windows (\) 和 Unix (/) 分隔符
  const separator = filePath.includes('\\') ? '\\' : '/';
  return filePath.substring(0, filePath.lastIndexOf(separator));
};

// 🟢 计算属性：动态显示最终导出路径
const finalExportPath = computed(() => {
  if (store.exportSettings.pathMode === 'custom') {
    return store.exportSettings.customPath || '⚠️ 尚未选择文件夹';
  } else {
    // 原图模式：尝试获取当前选中图片的父目录
    return store.activeFilePath 
      ? getParentDirectory(store.activeFilePath) 
      : '⚠️ 请先在左侧选择一张图片以预览路径';
  }
});

const selectFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择导出文件夹'
    });
    if (selected) {
      store.setExportPath(selected);
    }
  } catch (err) {
    console.error(err);
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="scroll-content">
      
      <div class="setting-group">
        <div class="group-header">
          <span class="icon">📂</span> 输出目录 / Output Path
        </div>
        
        <div class="radio-group">
          <label class="radio-label">
            <input type="radio" v-model="store.exportSettings.pathMode" value="original">
            <span>原图同级目录 (默认)</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="store.exportSettings.pathMode" value="custom">
            <span>自定义目录</span>
          </label>
        </div>

        <div v-if="store.exportSettings.pathMode === 'custom'" class="path-action-row">
           <button @click="selectFolder" class="browse-btn">选择文件夹...</button>
        </div>

        <div class="path-preview-card" :title="finalExportPath">
          <div class="label">保存位置:</div>
          <div class="path-text">{{ finalExportPath }}</div>
        </div>
      </div>

      <div class="divider"></div>

      <div class="setting-group">
        <div class="group-header">
          <span class="icon">🖼️</span> 格式与质量 / Format & Quality
        </div>

        <div class="format-options">
          <label class="radio-card" :class="{ active: store.exportSettings.format === 'jpg' }">
            <input type="radio" v-model="store.exportSettings.format" value="jpg" hidden>
            <span class="fmt-name">JPG</span>
            <span class="fmt-desc">通用 / 推荐</span>
          </label>
          
          <label class="radio-card" :class="{ active: store.exportSettings.format === 'png' }">
            <input type="radio" v-model="store.exportSettings.format" value="png" hidden>
            <span class="fmt-name">PNG</span>
            <span class="fmt-desc">无损 / 大体积</span>
          </label>

          </div>

        <div class="quality-box" v-if="store.exportSettings.format === 'jpg'">
          <div class="slider-header">
            <span>压缩质量</span>
            <span class="val-text">{{ store.exportSettings.quality }}%</span>
          </div>
          <input 
            type="range" 
            v-model.number="store.exportSettings.quality" 
            min="50" max="100" step="1"
            class="slider"
          />
          <div class="slider-hint">
            <span>50% (更小)</span>
            <span>100% (最佳)</span>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.settings-container {
  width: 100%; 
  height: 100%;
  padding: 20px 30px;
  color: var(--text-sub);
  
  /* 🟢 使用中间层背景，确保视觉平滑 */
  background: var(--bg-preset);
  backdrop-filter: blur(20px);
  
  overflow-y: auto;
  scrollbar-gutter: stable; 
}

:global([data-theme='light']) .settings-container {
  background: rgba(255, 255, 255, 0.7);
}

/* 限制内容宽度居中 */
.scroll-content { max-width: 500px; margin: 0 auto; }

.setting-group { margin-bottom: 25px; }

.group-header {
  font-size: 1.1em; font-weight: bold; color: var(--text-main);
  margin-bottom: 15px; display: flex; align-items: center; gap: 8px;
}
.icon { font-size: 1.2em; }

/* 单选框组 */
.radio-group { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
.radio-label {
  display: flex; align-items: center; gap: 10px; cursor: pointer;
  padding: 8px 12px; border-radius: 4px; transition: background 0.2s;
  border: 1px solid transparent;
  color: var(--text-main);
}
.radio-label:hover { background: var(--input-bg); }
.radio-label:has(input:checked) { background: var(--input-bg); border-color: var(--border-color); }
.radio-label input { accent-color: var(--nikon-yellow); transform: scale(1.1); }

/* 🟢 修改 2: 路径预览卡片样式调整 */
.path-preview-card {
  margin-top: 10px;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--text-sub);
  padding: 10px 12px;
  border-radius: 4px;
  
  /* ❌ 删掉了 font-family: monospace; 让它继承全局字体 */
  font-size: 0.85em;
  color: var(--text-sub);
  
  /* 保持换行逻辑，防止路径太长撑破容器 */
  word-break: break-all; 
  line-height: 1.5;
}

.radio-group:has(input[value="custom"]:checked) ~ .path-preview-card {
  border-left-color: var(--nikon-yellow);
  color: var(--text-main);
}

.path-preview-card .label { font-size: 0.85em; margin-bottom: 4px; opacity: 0.7; font-weight: 600; }
.path-action-row { margin-left: 28px; margin-bottom: 10px; }
.browse-btn {
  background: var(--input-bg); color: var(--text-main); border: 1px solid var(--border-color);
  padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 0.9em;
  transition: all 0.2s;
}
.browse-btn:hover { background: var(--border-color); border-color: var(--text-sub); }

.divider { height: 1px; background: var(--border-color); margin: 30px 0; }

/* 格式卡片 */
.format-options { display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 20px; }
.radio-card {
  background: var(--card-bg); border: 1px solid var(--border-color);
  padding: 15px; border-radius: 6px; cursor: pointer; text-align: center;
  transition: all 0.2s;
}
.radio-card:hover { border-color: var(--text-sub); background: var(--input-bg); }
.radio-card.active {
  border-color: var(--nikon-yellow); background: var(--nikon-yellow-dim, rgba(255, 225, 0, 0.05)); color: var(--text-main);
}
.fmt-name { display: block; font-weight: bold; font-size: 1.2em; margin-bottom: 4px; color: var(--text-main); }
.fmt-desc { display: block; font-size: 0.8em; color: var(--text-sub); }

/* 滑块 */
.quality-box { background: var(--card-bg); padding: 15px; border-radius: 6px; border: 1px solid var(--border-color); }
.slider-header { display: flex; justify-content: space-between; margin-bottom: 10px; font-size: 0.9em; color: var(--text-main); }
.val-text { color: var(--nikon-yellow); font-weight: bold; }
.slider { width: 100%; accent-color: var(--nikon-yellow); cursor: pointer; }
.slider-hint { display: flex; justify-content: space-between; font-size: 0.75em; color: var(--text-sub); margin-top: 5px; }
</style>