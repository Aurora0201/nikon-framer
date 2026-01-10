// src/utils/thumbnailManager.js
import { invoke } from '@tauri-apps/api/core';

class ThumbnailLoader {
  constructor() {
    this.queue = [];         // 等待队列
    this.processing = 0;     // 当前正在进行的请求数
    this.maxConcurrency = 4; // 🟢 最大并发数 (建议 4-6，太大会卡 IPC)
  }

  // 添加任务
  add(filePath, onSuccess, onError) {
    // 如果队列里已经有这个文件的任务，先移除旧的 (避免重复)
    this.remove(filePath);

    // 🟢 LIFO 策略：push 到数组末尾，取出时用 pop()
    // 这样保证最后进入视口的图片最先被加载
    this.queue.push({ filePath, onSuccess, onError });
    
    this.processNext();
  }

  // 移除任务 (当图片移出视口时调用)
  remove(filePath) {
    // 过滤掉未开始的任务
    this.queue = this.queue.filter(task => task.filePath !== filePath);
  }

  // 调度器
  async processNext() {
    // 如果正在处理的数量已满，或者队列空了，就停止
    if (this.processing >= this.maxConcurrency || this.queue.length === 0) return;

    this.processing++;

    // 🟢 取出最新的任务 (Last In First Out)
    // 对于快速滚动场景，这能极大提升体感速度
    const task = this.queue.pop(); 

    if (!task) {
      this.processing--;
      return;
    }

    try {
      // 调用 Rust
      const base64Str = await invoke('generate_thumbnail', { filePath: task.filePath });
      task.onSuccess(base64Str);
    } catch (err) {
      if (task.onError) task.onError(err);
    } finally {
      this.processing--;
      // 一个任务结束，递归尝试执行下一个
      this.processNext();
    }
  }
}

// 导出单例
export const thumbnailLoader = new ThumbnailLoader();