<script setup>
import { watch } from 'vue'; 
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../../store/index.js'; 
// 🟢 1. 直接引入静态配置数组
import { CATEGORY_OPTIONS } from '../../frames/registry.js'; 
import LazyThumbnail from '../common/LazyThumbnail.vue';

// --- 辅助逻辑 & 按钮动作 (保持原样，没有任何修改) ---
const handlePathList = (paths) => {
  const validPaths = paths.filter(p => /\.(jpg|jpeg|png|webp|tif|tiff|nef|dng|arw)$/i.test(p));
  if (validPaths.length > 0) {
    const files = validPaths.map(pathStr => ({ name: pathStr.replace(/^.*[\\/]/, ''), path: pathStr }));
    store.addFiles(files);
  }
};

const addFiles = async () => {
  if (store.isProcessing) return;
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'webp', 'nef', 'dng', 'arw'] }] });
    if (selected) {
      const paths = selected.map(item => typeof item === 'string' ? item : item.path);
      handlePathList(paths);
    }
  } catch (err) { console.error(err); }
};

const addFolder = async () => {
  if (store.isProcessing) return;
  try {
    const folderPath = await open({ directory: true, multiple: false });
    if (folderPath) {
      const rawPaths = await invoke('scan_folder', { folderPath });
      if (rawPaths && rawPaths.length > 0) handlePathList(rawPaths);
    }
  } catch (err) { console.error(err); }
};

// --- EXIF 监听 & 列表操作 (保持原样) ---
watch(() => store.fileQueue, (newQueue) => {
  newQueue.forEach(async (file) => {
    if (file.exifStatus === 'wait') {
      file.exifStatus = 'scanning'; 
      try {
        const isOk = await invoke('check_file_exif', { path: file.path });
        file.exifStatus = isOk ? 'ok' : 'no';
      } catch (e) { file.exifStatus = 'no'; }
    }
  });
}, { deep: true, immediate: true });

const selectFile = (path) => store.setActiveFile(path);
const removeFile = (e, index) => { e.stopPropagation(); store.removeFile(index); };
const clearAll = () => store.clearQueue(); 
</script>

<template>
  <div class="panel-header">
    <span>📂 资源 (Resources)</span>
    <div class="header-actions">
      <button class="icon-btn-mini" @click="addFiles" title="添加文件">📄</button>
      <button class="icon-btn-mini" @click="addFolder" title="添加文件夹">📂</button>
    </div>
  </div>
  
  <div class="panel-body">
    
    <div class="section">
      <label class="section-title">样式分类 / Category</label>
      <select 
        :value="store.settings.style" 
        @change="(e) => store.setCategory(e.target.value)"
        class="mode-select"
      >
        <option 
          v-for="opt in CATEGORY_OPTIONS" 
          :key="opt.value" 
          :value="opt.value"
        >
          {{ opt.label }}
        </option>
      </select>
    </div>

    <div class="section file-list-section">
    <div class="list-header-row">
      <label class="section-title">队列 ({{ store.fileQueue.length }})</label>
      <button v-if="store.fileQueue.length > 0" @click="clearAll" class="clear-btn">清空</button>
    </div>
    
    <div class="list-viewport">
      
      <div class="file-list">
        <div v-if="store.fileQueue.length === 0" class="empty-tip">
          <div style="font-size: 2em; margin-bottom: 10px;">📥</div>
          <div>拖入照片<br>或使用上方按钮</div>
        </div>

        <div 
          v-else
          v-for="(file, index) in store.fileQueue" 
          :key="file.path"
          class="file-item"
          :class="{ active: store.activeFilePath === file.path }"
          @click="selectFile(file.path)"
        >
          <div class="item-left">
              <LazyThumbnail :path="file.path" class="list-thumb" />
              <div class="name-col">
                <div class="name-row">
                  <span class="file-index">{{ index + 1 }}</span>
                  <span class="file-name" :title="file.name">{{ file.name }}</span>
                </div>
                <span class="exif-badge" :class="file.exifStatus">
                  {{ file.exifStatus === 'ok' ? 'EXIF DATA' : (file.exifStatus === 'scanning' ? 'SCANNING...' : 'NO EXIF') }}
                </span>
              </div>
            </div>
            <button @click="(e) => removeFile(e, index)" class="del-btn">×</button>
        </div>
      </div>

      <div v-if="store.isDragging" class="drag-overlay">
        <div class="overlay-content">
          <span style="font-size: 2em">📂</span>
          <span>释放添加图片</span>
        </div>
      </div>

    </div>
  </div>
  </div>
</template>

<style scoped>
/* =========================================
   1. 面板头部 (Header)
   ========================================= */
.panel-header {
  height: 40px; 
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 12px; 
  background: transparent; 
  border-bottom: 1px solid var(--border-color);
  font-weight: 600; font-size: 0.9em; 
  color: var(--text-main); 
  flex-shrink: 0;
}

.header-actions { display: flex; gap: 8px; }

.icon-btn-mini {
  background: var(--input-bg); 
  border: 1px solid var(--border-color); 
  color: var(--text-sub); 
  width: 26px; height: 26px;
  border-radius: 6px; cursor: pointer; 
  display: flex; align-items: center; justify-content: center; font-size: 14px;
  transition: all 0.2s;
}
.icon-btn-mini:hover { 
  background: var(--bg-color); /* Slightly different on hover */
  border-color: var(--text-sub);
  color: var(--text-main);
}

/* =========================================
   2. 主体区域 (Body)
   ========================================= */
.panel-body {
  flex: 1; padding: 12px; overflow: hidden; 
  display: flex; flex-direction: column; gap: 20px;
}

.section { display: flex; flex-direction: column; }
.section-title {
  display: block; font-size: 0.75em; 
  color: var(--text-sub); 
  margin-bottom: 8px; 
  text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px;
}

/* =========================================
   3. 下拉选框 (Select) - 调亮，不再死黑
   ========================================= */
.mode-select {
  width: 100%;
  
  background-color: var(--input-bg); 
  color: var(--text-main);
  
  border: 1px solid var(--border-color);
  
  padding: 8px 10px; border-radius: 6px; outline: none; font-size: 0.9em; cursor: pointer;
  appearance: none; -webkit-appearance: none;
  
  /* Use a generic svg icon or encoded one that works on dark/light */
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat; background-position: right 10px center; background-size: 16px; padding-right: 35px;
  
  transition: all 0.2s;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.05); 
}
.mode-select:focus { 
  border-color: var(--border-focus); 
}
.mode-select:hover { border-color: var(--text-sub); }
.mode-select option { background-color: var(--input-bg); color: var(--text-main); }

/* =========================================
   4. 列表视口 (Viewport) - 调亮底色
   ========================================= */
.file-list-section { flex: 1; min-height: 0; display: flex; flex-direction: column; }

.list-header-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
.clear-btn { background: none; border: none; color: var(--text-sub); font-size: 0.75em; cursor: pointer; padding: 0; }
.clear-btn:hover { color: var(--status-no-text); text-decoration: underline; }

.list-viewport {
  flex: 1; position: relative; overflow: hidden; display: flex;
  
  background: var(--inner-bg); 
  
  border: 1px solid var(--border-color);
  /* 仅保留微弱的内阴影 */
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.05);
  
  border-radius: 6px;
}

.file-list { flex: 1; overflow-y: auto; width: 100%; display: flex; flex-direction: column; }

/* =========================================
   5. 列表项 (File Item) - 核心修改
   ========================================= */
.file-item {
  padding: 8px 10px; height: 64px;
  border-bottom: 1px solid var(--border-color);
  display: flex; align-items: center; justify-content: space-between;
  transition: background 0.2s;
  position: relative; /* 为伪元素定位 */
}

/* 悬停 */
.file-item:hover { background: var(--nikon-yellow-dim); }

/* 🟢 选中状态 */
.file-item.active {
  background: linear-gradient(90deg, var(--nikon-yellow-dim) 0%, transparent 100%);
  border-left: 3px solid var(--nikon-yellow);
  padding-left: 7px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.item-left { display: flex; align-items: center; overflow: hidden; gap: 12px; flex: 1; }

.list-thumb { 
  margin-right: 0; flex-shrink: 0; 
  border-radius: 2px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  opacity: 0.9;
}

/* 序号标签 */
.file-index {
  font-family: inherit; font-size: 0.7em; font-weight: 700;
  
  color: var(--text-sub); 
  background: rgba(125, 125, 125, 0.1); 
  
  width: 18px; height: 18px; border-radius: 4px;
  display: flex; align-items: center; justify-content: center; flex-shrink: 0; line-height: 1;
}

/* 选中时：实心黄，文字变黑 */
.file-item.active .file-index { 
  background: var(--nikon-yellow); 
  color: #121212; 
  box-shadow: 0 0 8px rgba(255, 215, 0, 0.4); 
}

.name-col { display: flex; flex-direction: column; gap: 4px; overflow: hidden; justify-content: center; }
.name-row { display: flex; align-items: center; gap: 8px; width: 100%; }

.file-name {
  font-size: 0.9em; font-weight: 500;
  color: var(--text-main); 
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; line-height: 1.2;
}
/* 选中文字高亮 */
.file-item.active .file-name { color: var(--text-main); font-weight: 600; text-shadow: none; }

/* EXIF 徽章 */
.exif-badge {
  font-size: 9px; padding: 1px 5px; border-radius: 3px;
  background: rgba(125, 125, 125, 0.1); 
  color: var(--text-sub); 
  width: fit-content; font-weight: 600; letter-spacing: 0.3px;
}
.exif-badge.ok { 
  background: var(--status-ok-bg); 
  color: var(--status-ok-text); 
}
.exif-badge.no { 
  background: var(--status-no-bg); 
  color: var(--status-no-text); 
}
.exif-badge.scanning { color: var(--nikon-yellow); background: rgba(255, 215, 0, 0.1); }

.del-btn {
  background: none; border: none; color: var(--text-sub); 
  cursor: pointer; font-size: 1.4em; line-height: 1; padding: 0 5px; margin-left: 5px;
  transition: color 0.2s;
}
.del-btn:hover { color: var(--status-no-text); }

.empty-tip {
  flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  text-align: center; color: var(--text-sub); 
  font-size: 0.85em; min-height: 150px; user-select: none;
}
</style>