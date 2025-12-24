<script setup>
import { ref, watch, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

const isDragging = ref(false);
const dragCounter = ref(0);


// --- 动作：添加文件 ---
const addFiles = async () => {
  if (store.isProcessing) return;
  
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'nef', 'dng', 'arw'] }]
    });
    
    if (selected) {
      // 这里的 selected 就是文件路径数组
      // 我们需要把它构造成对象格式 { name: 'xxx', path: 'xxx' }
      const files = selected.map(path => ({
        name: path.replace(/^.*[\\/]/, ''), // 提取文件名
        path: path
      }));
      
      const addedCount = store.addFiles(files);
      if (addedCount > 0) {
        store.setStatus(`已添加 ${addedCount} 个文件`, 'success');
      } else {
        store.setStatus('文件已存在列表中', 'normal');
      }
    }
  } catch (err) {
    console.error(err);
    store.setStatus('添加文件失败', 'error');
  }
};

// --- 动作：添加文件夹 ---
const addFolder = async () => {
  if (store.isProcessing) return;

  try {
    const folderPath = await open({
      directory: true,
      multiple: false,
    });

    if (folderPath) {
      store.setStatus(`正在扫描: ${folderPath}...`, 'loading');
      
      // 🟢 调用 Rust 后端扫描文件夹
      // 注意：Rust 返回的可能已经是 struct { name, path } 或者只是 path
      // 假设 Rust 返回的是对象数组 (根据你之前的逻辑)
      const files = await invoke('scan_folder', { folderPath });

      if (files && files.length > 0) {
        const addedCount = store.addFiles(files);
        store.setStatus(`成功添加 ${addedCount} 张照片`, 'success');
      } else {
        store.setStatus('该文件夹内没有发现支持的图片', 'error');
      }
    }
  } catch (err) {
    console.error(err);
    store.setStatus('读取文件夹失败', 'error');
  }
};

// --- 动作：EXIF 检查辅助 ---
// 这是一个优化体验的逻辑：当列表有新文件(exifStatus='wait')时，去检查EXIF
// Vue 的 watch 可以监听 store.fileQueue 的变化

watch(() => store.fileQueue, async (newQueue) => {
  newQueue.forEach(async (file, index) => {
    if (file.exifStatus === 'wait') {
      // 标记为 scanning 防止重复检查
      file.exifStatus = 'scanning'; 
      try {
        const isOk = await invoke('check_exif', { path: file.path });
        file.exifStatus = isOk ? 'ok' : 'no';
      } catch (e) {
        file.exifStatus = 'no';
      }
    }
  });
}, { deep: true }); // 深度监听数组变化

</script>

<template>
  <div class="batch-controls">
    <label>处理列表 / Processing Queue</label>
    
    <div class="batch-btn-group">
      <button 
        @click="addFiles" 
        class="secondary-btn" 
        :disabled="store.isProcessing"
      >
        + 添加文件 (Files)
      </button>
      <button 
        @click="addFolder" 
        class="secondary-btn" 
        :disabled="store.isProcessing"
      >
        + 添加文件夹 (Folder)
      </button>
    </div>

    <div 
      id="drop-zone" 
      class="drop-zone"
      :class="{ 
        'active': store.isDragging,
        'has-files': store.fileQueue.length > 0,
        'disabled': store.isProcessing 
      }"
      @dragover.prevent="handleDragEnter" 
      @drop.prevent
    >
      <div id="empty-tip" v-if="store.fileQueue.length === 0">
        <p>拖拽照片到此处</p>
        <p style="font-size: 0.8em; opacity: 0.7;">(Drag & Drop photos here)</p>
      </div>

      <ul v-else id="file-list" class="file-list" :class="{ 'disabled-interaction': store.isProcessing }">
        <li 
          v-for="(file, index) in store.fileQueue" 
          :key="file.path" 
          class="file-item"
        >
          <div class="file-info">
            <span class="file-name" :title="file.path">
              <span class="file-index">{{ index + 1 }}</span>
              {{ file.name }}
            </span>
            
            <span 
              class="tag-exif" 
              :class="file.exifStatus"
            >
              {{ file.exifStatus === 'ok' ? 'EXIF' : (file.exifStatus === 'no' ? 'NO EXIF' : 'SCANNING...') }}
            </span>
          </div>
          
          <button 
            class="remove-item-btn" 
            @click="store.removeFile(index)"
            :disabled="store.isProcessing"
          >
            ×
          </button>
        </li>
      </ul>
    </div>

    <div class="queue-stats">
      <span id="queue-count">{{ store.fileQueue.length }} 张照片</span>
      <button 
        id="clear-list-btn" 
        @click="store.clearQueue"
        :disabled="store.isProcessing || store.fileQueue.length === 0"
        style="background:none; border:none; color:#777; font-size:inherit; cursor: pointer;"
      >
        清空列表 (Clear)
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Vue 的 scoped style 只对当前组件生效
  但你的样式已经在全局 css 里定义了，所以这里留空即可
*/
</style>