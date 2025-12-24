// src/store.js
import { reactive } from 'vue';

export const store = reactive({
  // --- 核心数据 ---
  fileQueue: [],      // 文件列表
  isProcessing: false, // 是否正在处理
  
  // --- 进度与状态 ---
  progress: {
    current: 0,
    total: 0,
    percent: 0
  },
  statusText: "准备就绪，请添加照片。",
  statusType: "normal", // normal, success, error, loading
  
  // --- 设置选项 (ControlPanel 用) ---
  settings: {
    style: 'BottomWhite',
    font: 'default',
    weight: 'Normal',
    shadowIntensity: 1.0
  },

  // --- 动作 (Actions) ---
  // 添加文件
  addFiles(newFiles) {
    // 简单的去重逻辑
    const existingPaths = new Set(this.fileQueue.map(f => f.path));
    const uniqueFiles = newFiles.filter(f => !existingPaths.has(f.path));
    
    // 为每个文件添加 UI 状态 (exifStatus)
    const formattedFiles = uniqueFiles.map(f => ({
      name: f.name,
      path: f.path,
      exifStatus: 'wait' // 默认为 wait，稍后会检查 EXIF
    }));
    
    this.fileQueue.push(...formattedFiles);
    return uniqueFiles.length; // 返回实际添加的数量
  },

  // 移除文件
  removeFile(index) {
    this.fileQueue.splice(index, 1);
  },

  // 清空列表
  clearQueue() {
    this.fileQueue = [];
    this.progress = { current: 0, total: 0, percent: 0 };
    this.statusText = "列表已清空";
    this.statusType = "normal";
  },

  // 更新状态栏
  setStatus(text, type = "normal") {
    this.statusText = text;
    this.statusType = type;
  },

  // 更新进度
  updateProgress(current, total) {
    this.progress.current = current;
    this.progress.total = total;
    this.progress.percent = total > 0 ? Math.round((current / total) * 100) : 0;
  }
});