<script setup>
import { watch } from 'vue'; 
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../../store/index.js'; 
// 🟢 1. 直接引入静态配置数组
import { CATEGORY_OPTIONS } from '../../frames/registry.js'; 

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
const clearAll = () => { if(confirm('确定清空列表?')) store.clearQueue(); };
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
      
      <div class="file-list" :class="{ 'drag-active': store.isDragging }">
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
            <span class="file-index">{{ index + 1 }}</span>
            <div class="name-col">
              <span class="file-name" :title="file.name">{{ file.name }}</span>
              <span class="exif-badge" :class="file.exifStatus">
                {{ file.exifStatus === 'ok' ? 'EXIF' : (file.exifStatus === 'scanning' ? '...' : 'NO EXIF') }}
              </span>
            </div>
          </div>
          <button @click="(e) => removeFile(e, index)" class="del-btn">×</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 面板头部 */
.panel-header {
  height: 40px; display: flex; align-items: center; justify-content: space-between;
  padding: 0 12px; background: #1a1a1a; border-bottom: 1px solid #333;
  font-weight: 600; font-size: 0.9em; color: #ccc; flex-shrink: 0;
}
.header-actions { display: flex; gap: 8px; }
.icon-btn-mini {
  background: #333; border: 1px solid #444; color: #fff; width: 26px; height: 26px;
  border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 14px;
}
.icon-btn-mini:hover { background: #444; border-color: #666; }

/* 主体区域 */
.panel-body {
  flex: 1; padding: 12px; overflow-y: auto; display: flex; flex-direction: column; gap: 20px;
}
.section { display: flex; flex-direction: column; }
.section-title {
  display: block; font-size: 0.75em; color: #666; margin-bottom: 6px; 
  text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px;
}

/* 🟢 [修复 2] 下拉选框美化 */
.mode-select {
  width: 100%;
  background-color: #222;
  color: #fff;
  border: 1px solid #444;
  padding: 8px 10px;
  border-radius: 4px;
  outline: none;
  font-size: 0.9em;
  cursor: pointer;
  
  /* 关键：去除默认外观，使用 SVG 自定义箭头 */
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  
  /* SVG 箭头图标 (白色) */
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-position: right 10px center;
  background-size: 16px;
  padding-right: 35px; /* 给箭头留出空间 */
  
  transition: border-color 0.2s;
}
.mode-select:focus { border-color: #666; }
.mode-select:hover { border-color: #555; }

/* 列表区域 */
.file-list-section { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.list-header-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
.clear-btn { background: none; border: none; color: #555; font-size: 0.75em; cursor: pointer; padding: 0; }
.clear-btn:hover { color: #d44; text-decoration: underline; }

.file-list {
  flex: 1; overflow-y: auto; display: flex; flex-direction: column;
  border: 1px solid #222; border-radius: 4px;
}

.file-item {
  padding: 10px 10px; /* 稍微增加一点点击区域 */
  border-bottom: 1px solid #2a2a2a;
  display: flex; align-items: center; justify-content: space-between;
  transition: background 0.2s;
}
.file-item:hover { background: #252525; }
.file-item.active {
  background: #2c2c2c;
  border-left: 3px solid var(--nikon-yellow);
  padding-left: 7px;
}

.item-left { display: flex; align-items: center; overflow: hidden; gap: 10px; flex: 1; }

.file-index {
  font-size: 0.75em; color: #555; width: 20px; height: 20px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 50%; background: #222; flex-shrink: 0;
}
.file-item.active .file-index { color: var(--nikon-yellow); background: rgba(255,225,0,0.1); }

.name-col { display: flex; flex-direction: column; overflow: hidden; gap: 3px; flex: 1; }

/* 🟢 [修复 3] 文件名加大 */
.file-name {
  font-size: 0.95em; /* 从 0.85em 增大 */
  font-weight: 500;
  color: #ddd; /* 稍微调亮一点 */
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.file-item.active .file-name { color: #fff; font-weight: 600; }

.exif-badge {
  font-size: 9px; padding: 1px 4px; border-radius: 2px;
  background: #333; color: #666; width: fit-content; font-weight: bold;
}
.exif-badge.ok { background: rgba(102, 187, 106, 0.15); color: #66bb6a; }
.exif-badge.no { background: rgba(183, 28, 28, 0.2); color: #ef5350; }
.exif-badge.scanning { color: var(--nikon-yellow); }

.empty-tip {
  flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  text-align: center; color: #444; font-size: 0.85em; min-height: 150px; user-select: none;
}
.del-btn {
  background: none; border: none; color: #444; cursor: pointer;
  font-size: 1.4em; line-height: 1; padding: 0 5px; margin-left: 5px;
}
.del-btn:hover { color: #d44; }
</style>