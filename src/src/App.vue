<script setup>
import { onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event'; // 🟢 Tauri 事件监听
// 🔴 之前漏掉了这一行，请补上！
import { invoke } from '@tauri-apps/api/core';
// 引入所有组件
import ControlPanel from './components/ControlPanel.vue';
import FileList from './components/FileList.vue';
import StatusBar from './components/StatusBar.vue';
import PreviewModal from './components/PreviewModal.vue';
import DebugTools from './components/DebugTools.vue';


// 引入全局状态
import { store } from './store.js';

// 🟢 [修复 1] 先注册卸载钩子 (不要放在 await 后面)
let unlistenDrop = null;
let unlistenProgress = null;
let unlistenStatus = null;
let unlistenEnter = null; // 新增
let unlistenLeave = null; // 新增

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop();
  if (unlistenProgress) unlistenProgress();
  if (unlistenStatus) unlistenStatus();
  if (unlistenEnter) unlistenEnter();
  if (unlistenLeave) unlistenLeave();
});



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


  // 3. 监听文件拖拽 (tauri://drag-drop)
  // ==========================================
  // 🟢 全局拖拽系统 (视觉 + 数据)
  // ==========================================
  
  // 1. 进入窗口：开启高亮
  unlistenEnter = await listen('tauri://drag-enter', () => {
    if (!store.isProcessing) {
      store.isDragging = true; // 修改 Store、
      console.log('文件进入')
    }
  });

  // 2. 监听拖拽离开
  unlistenLeave = await listen('tauri://drag-leave', () => {
    store.isDragging = false;
    console.log('文件离开')
  });

  // 3. 放下文件：处理数据 + 关闭高亮
  unlistenDrop = await listen('tauri://drag-drop', async (event) => {
    // 🟢 1. 无论成功与否，立即关闭高亮状态 (UI复位)
    store.isDragging = false; 

    // 🟢 2. 如果正在批处理中，直接忽略，防止数据混乱
    if (store.isProcessing) return;

    const paths = event.payload.paths;

    if (paths && paths.length > 0) {
      try {
        // 🟢 3. 调用 Rust 后端过滤文件 (只保留支持的图片格式)
        // 此时 validFiles 是一个字符串数组: ["C:\path\a.jpg", ...]
        const validFiles = await invoke('filter_files', { paths });
        
        if (validFiles.length > 0) {
          // 🟢 4. 关键步骤：格式转换
          // Store 需要对象格式 { name, path }，而 Rust 返回的是字符串路径
          const formattedFiles = validFiles.map(path => ({
            // 使用正则提取文件名 (兼容 Windows 反斜杠 \ 和 Mac/Linux 斜杠 /)
            name: path.replace(/^.*[\\/]/, ''),
            path: path
          }));

          // 🟢 5. 存入全局 Store
          const count = store.addFiles(formattedFiles);
          
          // 🟢 6. 给用户反馈
          if (count > 0) {
            store.setStatus(`已添加 ${count} 个文件`, 'success');
          } else {
            store.setStatus("文件已存在列表中", 'normal');
          }
        } else {
          store.setStatus("未检测到支持的图片文件", "error");
        }
      } catch (e) {
        console.error("Drop Error:", e);
        store.setStatus("文件添加失败", "error");
      }
    }
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