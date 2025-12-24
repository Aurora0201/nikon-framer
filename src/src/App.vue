<script setup>
import { onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event'; // 🟢 Tauri 事件监听
// 引入所有组件
import ControlPanel from './components/ControlPanel.vue';
import FileList from './components/FileList.vue';
import StatusBar from './components/StatusBar.vue';
import PreviewModal from './components/PreviewModal.vue';
import DebugTools from './components/DebugTools.vue';


// 引入全局状态
import { store } from './store.js';

onMounted(async () => {
  console.log("🚀 App 已挂载，开始注册监听器...");

  // 1. 监听进度更新 (process-progress)
  const unlistenProgress = await listen('process-progress', (event) => {
    const { current, total, filepath, status } = event.payload;
    
    // 更新进度条数据
    store.updateProgress(current, total);

    // 提取文件名 (兼容 Windows/Mac 路径)
    const filename = filepath.replace(/^.*[\\/]/, '');

    // 更新状态文字
    if (status === 'skipped') {
      store.setStatus(`[跳过] 无EXIF: ${filename}`, 'loading');
    } else {
      store.setStatus(`正在处理: ${filename}`, 'loading');
    }
  });

  // 2. 监听任务状态 (process-status)
  const unlistenStatus = await listen('process-status', (event) => {
    const status = event.payload; // 'finished' | 'stopped'
    
    store.isProcessing = false; // 关掉处理状态

    if (status === 'finished') {
      store.setStatus("批处理完成！", "success");
      // 1.5秒后重置进度条 (视觉优化)
      setTimeout(() => {
        store.progress.total = 0; 
      }, 1500);
    } else if (status === 'stopped') {
      store.setStatus("已终止批处理", "error");
      store.progress.total = 0;
    }
  });

  // 保存卸载函数，防止内存泄漏 (虽然 App.vue 一般不会卸载)
  onUnmounted(() => {
    unlistenProgress();
    unlistenStatus();
  });
});

</script>

<template>
  <h1>NIKON <span>Z</span> FRAMER</h1>

  <div class="control-group">
    <ControlPanel />
    
    <FileList />

    <button id="start-batch-btn">开始批处理 (Start Batch)</button>
  </div>
  
  <StatusBar />

  <PreviewModal />

  <DebugTools />
</template>

<style scoped>
/* 这里可以写针对 App 布局的特定样式，目前用全局样式的就够了 */
</style>