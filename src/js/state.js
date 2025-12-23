import { el } from './elements.js';

// --- 预览图状态 ---
export const previewState = {
  scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0,
};

// --- 🟢 [新增] 文件队列管理 ---
export const fileQueue = {
  files: [], // 存储结构: { path: string, name: string, exifStatus: 'wait'|'ok'|'no' }
  
  /**
   * 添加文件到队列
   * @param {Array|FileList} items - 文件路径数组 或 FileList对象
   * @returns {boolean} 是否有新文件被添加
   */
  add(items) {
    const newPaths = [];
    
    // 1. 归一化输入：无论是 FileList 还是 路径数组，都提取出 path 字符串
    // 注意：在 Tauri 环境下，HTML5 的 File 对象通常包含 path 属性
    Array.from(items).forEach(item => {
        // 如果是对象且有 path 属性(拖拽/Select)，用 path；如果是纯字符串(Rust返回)，直接用
        const path = item.path ? item.path : item; 
        if (typeof path === 'string') newPaths.push(path);
    });

    let addedCount = 0;
    // 使用 Set 防止重复添加
    const existingPaths = new Set(this.files.map(f => f.path));

    newPaths.forEach(path => {
      if (!existingPaths.has(path)) {
        // 简单的提取文件名逻辑 (兼容 Windows \ 和 Unix /)
        const name = path.replace(/^.*[\\/]/, '');
        
        this.files.push({
          path: path,
          name: name,
          exifStatus: 'wait' // 默认状态：等待检查
        });
        addedCount++;
      }
    });
    return addedCount > 0;
  },

  /**
   * 移除指定索引的文件
   */
  remove(index) {
    this.files.splice(index, 1);
  },

  /**
   * 清空所有文件
   */
  clear() {
    this.files = [];
  }
};

// --- 视图重置逻辑 ---
export function resetViewState() {
  previewState.scale = 1;
  previewState.panning = false;
  previewState.pointX = 0;
  previewState.pointY = 0;
  updateTransform();
  el.modal.style.display = "none";
  el.previewImg.src = "";
}

export function updateTransform() {
  el.previewImg.style.transform = `translate(${previewState.pointX}px, ${previewState.pointY}px) scale(${previewState.scale})`;
}