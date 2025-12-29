// src/composables/useGlobalEvents.js
import { onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { store } from '../store.js';

export function useGlobalEvents() {
  // 保存监听器的卸载函数
  let listeners = [];

  // 清理函数
  const cleanup = () => {
    listeners.forEach(unlisten => unlisten());
    listeners = [];
  };

  onUnmounted(cleanup);

  onMounted(async () => {
    console.log("📡 注册全局事件监听器...");

    // 1. 进度监听
    const unlistenProgress = await listen('process-progress', (event) => {
      const { current, total, filepath, status } = event.payload;
      store.updateProgress(current, total);
      const filename = filepath.replace(/^.*[\\/]/, '');
      
      const msg = status === 'skipped' ? `[跳过] 无EXIF: ${filename}` : `正在处理: ${filename}`;
      store.setStatus(msg, 'loading');
    });
    listeners.push(unlistenProgress);

    // 2. 状态监听
    const unlistenStatus = await listen('process-status', (event) => {
      const status = event.payload;
      // 这里只处理通过状态，具体的按钮重置逻辑交给 useBatchProcess 处理，或者通过 store 通信
      store.isProcessing = false;
      
      if (status === 'finished') {
        store.setStatus("批处理完成！✨", "success");
        setTimeout(() => { if (!store.isProcessing) store.progress.total = 0; }, 1500);
      } else if (status === 'stopped') {
        store.setStatus("已终止批处理", "error");
        store.progress.total = 0;
      }
    });
    listeners.push(unlistenStatus);

    // 3. 拖拽逻辑 (进入/离开/放下)
    const unlistenEnter = await listen('tauri://drag-enter', () => {
      if (!store.isProcessing) store.isDragging = true;
    });
    listeners.push(unlistenEnter);

    const unlistenLeave = await listen('tauri://drag-leave', () => {
      store.isDragging = false;
    });
    listeners.push(unlistenLeave);

    const unlistenDrop = await listen('tauri://drag-drop', async (event) => {
      store.isDragging = false;
      if (store.isProcessing) return;

      const paths = event.payload.paths;
      if (paths?.length > 0) {
        try {
          const validFiles = await invoke('filter_files', { paths });
          if (validFiles.length > 0) {
            const formattedFiles = validFiles.map(path => ({
              name: path.replace(/^.*[\\/]/, ''),
              path: path
            }));
            const count = store.addFiles(formattedFiles);
            store.setStatus(count > 0 ? `已添加 ${count} 个文件` : "文件已存在", count > 0 ? 'success' : 'normal');
          } else {
            store.setStatus("未检测到支持的图片文件", "error");
          }
        } catch (e) {
          console.error("Drop Error:", e);
          store.setStatus("文件添加失败", "error");
        }
      }
    });
    listeners.push(unlistenDrop);
  });
}