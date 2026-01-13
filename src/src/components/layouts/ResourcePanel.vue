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
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  font-weight: 600; font-size: 0.9em; 
  color: rgba(255, 255, 255, 0.85); 
  flex-shrink: 0;
}

.header-actions { display: flex; gap: 8px; }

.icon-btn-mini {
  background: rgba(255, 255, 255, 0.05); 
  border: 1px solid rgba(255, 255, 255, 0.1); 
  color: rgba(255, 255, 255, 0.7); 
  width: 26px; height: 26px;
  border-radius: 6px; cursor: pointer; 
  display: flex; align-items: center; justify-content: center; font-size: 14px;
  transition: all 0.2s;
}
.icon-btn-mini:hover { 
  background: rgba(255, 255, 255, 0.15); 
  border-color: rgba(255, 255, 255, 0.3);
  color: #fff;
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
  color: rgba(255, 255, 255, 0.5); 
  margin-bottom: 8px; 
  text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px;
}

/* =========================================
   3. 下拉选框 (Select) - 调亮，不再死黑
   ========================================= */
.mode-select {
  width: 100%;
  
  /* 🟢 修改：不再用 0.6 的黑，改用 lighter 的深空灰，更融合 */
  background-color: rgba(30, 30, 35, 0.4); 
  color: rgba(255, 255, 255, 0.95);
  
  border: 1px solid rgba(255, 255, 255, 0.1);
  
  padding: 8px 10px; border-radius: 6px; outline: none; font-size: 0.9em; cursor: pointer;
  appearance: none; -webkit-appearance: none;
  
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat; background-position: right 10px center; background-size: 16px; padding-right: 35px;
  
  transition: all 0.2s;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.2); 
}
.mode-select:focus { 
  border-color: rgba(255, 255, 255, 0.3); 
  background-color: rgba(30, 30, 35, 0.6); 
}
.mode-select:hover { border-color: rgba(255, 255, 255, 0.2); background-color: rgba(255, 255, 255, 0.1); }
.mode-select option { background-color: #252528; color: #eee; }

/* =========================================
   4. 列表视口 (Viewport) - 调亮底色
   ========================================= */
.file-list-section { flex: 1; min-height: 0; display: flex; flex-direction: column; }

.list-header-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
.clear-btn { background: none; border: none; color: rgba(255, 255, 255, 0.4); font-size: 0.75em; cursor: pointer; padding: 0; }
.clear-btn:hover { color: #d44; text-decoration: underline; }

.list-viewport {
  flex: 1; position: relative; overflow: hidden; display: flex;
  
  /* 🟢 修改：从 0.5 降到 0.25，去除“黑洞感” */
  background: rgba(0, 0, 0, 0.25); 
  
  border: 1px solid rgba(255, 255, 255, 0.08);
  /* 仅保留微弱的内阴影 */
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.2);
  
  border-radius: 6px;
}

.file-list { flex: 1; overflow-y: auto; width: 100%; display: flex; flex-direction: column; }

/* =========================================
   5. 列表项 (File Item) - 核心修改
   ========================================= */
.file-item {
  padding: 8px 10px; height: 64px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  display: flex; align-items: center; justify-content: space-between;
  transition: background 0.2s;
  position: relative; /* 为伪元素定位 */
}

/* 悬停：微弱的白光 */
.file-item:hover { background: rgba(255, 255, 255, 0.03); }

/* 🟢 选中状态：流光渐变 (The Golden Glow) */
.file-item.active {
  /* 不再是实心色块，而是从左侧黄色发出的渐变光 */
  background: linear-gradient(90deg, rgba(255, 215, 0, 0.15) 0%, rgba(255, 255, 255, 0.05) 100%);
  
  /* 左侧指示条保持 */
  border-left: 3px solid var(--nikon-yellow);
  padding-left: 7px;
  
  /* 上下加一条极细的高光线，增加精致感 */
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

.item-left { display: flex; align-items: center; overflow: hidden; gap: 12px; flex: 1; }

.list-thumb { 
  margin-right: 0; flex-shrink: 0; 
  border-radius: 2px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.4);
  opacity: 0.9;
}

/* 序号标签 - 玻璃化 */
.file-index {
  font-family: inherit; font-size: 0.7em; font-weight: 700;
  
  /* 🟢 修改：未选中时是半透明玻璃 */
  color: rgba(255, 255, 255, 0.5); 
  background: rgba(255, 255, 255, 0.1); 
  
  width: 18px; height: 18px; border-radius: 4px;
  display: flex; align-items: center; justify-content: center; flex-shrink: 0; line-height: 1;
}

/* 选中时：实心黄，文字变黑 */
.file-item.active .file-index { 
  background: var(--nikon-yellow); 
  color: #111; 
  box-shadow: 0 0 8px rgba(255, 215, 0, 0.4); /* 序号发光 */
}

.name-col { display: flex; flex-direction: column; gap: 4px; overflow: hidden; justify-content: center; }
.name-row { display: flex; align-items: center; gap: 8px; width: 100%; }

.file-name {
  font-size: 0.9em; font-weight: 500;
  color: rgba(255, 255, 255, 0.75); 
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; line-height: 1.2;
}
/* 选中文字高亮 */
.file-item.active .file-name { color: #fff; text-shadow: 0 0 5px rgba(0,0,0,0.5); font-weight: 600; }

/* EXIF 徽章 - 保持鲜艳 */
.exif-badge {
  font-size: 9px; padding: 1px 5px; border-radius: 3px;
  background: rgba(255, 255, 255, 0.1); 
  color: rgba(255, 255, 255, 0.6); 
  width: fit-content; font-weight: 600; letter-spacing: 0.3px;
}
.exif-badge.ok { 
  background: rgba(102, 187, 106, 0.2); 
  color: #28a52e; 
}
.exif-badge.no { 
  background: rgba(229, 115, 115, 0.2); 
  color: #e64f4f; 
}
.exif-badge.scanning { color: var(--nikon-yellow); background: rgba(255, 215, 0, 0.1); }

.del-btn {
  background: none; border: none; color: rgba(255, 255, 255, 0.3); 
  cursor: pointer; font-size: 1.4em; line-height: 1; padding: 0 5px; margin-left: 5px;
  transition: color 0.2s;
}
.del-btn:hover { color: #ff5252; }

.empty-tip {
  flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
  text-align: center; color: rgba(255, 255, 255, 0.25); 
  font-size: 0.85em; min-height: 150px; user-select: none;
}

.drag-overlay {
  position: absolute; top: 0; left: 0; right: 0; bottom: 0; z-index: 99;
  background-color: rgba(20, 20, 20, 0.85); 
  border: 2px dashed var(--nikon-yellow); 
  backdrop-filter: blur(4px); 
  display: flex; align-items: center; justify-content: center; pointer-events: none;
}
.overlay-content { color: var(--nikon-yellow); font-weight: bold; font-size: 1.1em; display: flex; flex-direction: column; align-items: center; gap: 10px; text-transform: uppercase; letter-spacing: 1px; }
</style>