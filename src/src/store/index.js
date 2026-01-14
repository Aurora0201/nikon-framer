import { reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { frameRegistry } from '../frames/registry';

export const store = reactive({
  // --- 1. 文件队列与核心状态 ---
  fileQueue: [],
  activeFilePath: null,
  activePresetId: 'WhiteClassic', // 当前选中的 ID (通常与 style 保持一致)
  
  // --- 2. 处理流程状态 ---
  rawBlobUrl: null,        // 本地原图 Blob URL
  isLoadingPresets: false, // 切换模式时的 Loading 状态
  isProcessing: false,     // 是否正在执行批量处理
  isDragging: false,       // 🟢 [补全] 是否正在拖拽文件到窗口
  
  // 缓存已生成的结果图 (Key: "path|style", Value: "blob_url" or "file_path")
  processedFiles: new Map(), 

  // --- 3. 进度与状态提示 ---
  progress: { 
    current: 0, 
    total: 0, 
    percent: 0 
  },
  statusText: "准备就绪",
  statusType: "normal",    // 🟢 [补全] 'normal' | 'success' | 'error'

  // --- 4. 开放参数槽 (Custom Params) ---
  // 这里存放所有模式可能用到的自定义参数
  // 🟢 [重构] 只有这一个对象，用来存放"当前模式"的参数
  // 不再写死 signatureText, fontScale...
  modeParams: {},

  // --- 5. 全局设置 ---
  settings: {
    style: 'ClassicWhite', // 当前选中的大类 Key (对应 Registry Key)
  },

  // 🟢 新增：存储底图的真实物理尺寸
  imageDimensions: { width: 0, height: 0 },

  // 🟢 [新增] 导出全局设置
  exportSettings: {
    pathMode: 'original', // 'original' | 'custom'
    customPath: '',       // 自定义输出目录
    format: 'jpg',        // 'jpg' | 'png' | 'webp'
    quality: 90,          // 1-100 (仅 JPG/WebP)
    resize: 'none',       // 'none' | 'short-2048' | 'short-4096' (预留)
  },

  // 🟢 [新增] 颜色模式
  theme: localStorage.getItem('app-theme') || 'dark', // 'dark' | 'light'

  // 🟢 [新增] 切换颜色模式
  toggleTheme() {
    this.theme = this.theme === 'dark' ? 'light' : 'dark';
    localStorage.setItem('app-theme', this.theme);
  },

  // =========================================
  // Getters (计算属性)
  // =========================================

  // 🟢 [BUG 核心修复] 
  // 必须使用 this.activePresetId (具体ID) 去查配置
  // 绝对不能用 this.settings.style (那是分类名，查不到的)
  get currentModeConfig() {
    return frameRegistry.get(this.activePresetId);
  },

  // 🟢 获取当前分类下的所有预设 (用于中间面板)
  get currentPresets() {
    // 这里才应该用 settings.style (分类名)
    return frameRegistry.getByCategory(this.settings.style).map(item => ({
      id: item.id,
      name: item.label,
      desc: item.desc || item.label,
      img: item.getPresetUrl ? item.getPresetUrl() : '' 
    }));
  },

  // 3. 预览源 (逻辑微调)
  get previewSource() {
    const config = this.currentModeConfig;
    
    // 获取当前具体 ID 对应的图
    const presetPreview = {
      type: 'preset',
      url: config.getPresetUrl ? config.getPresetUrl() : '', 
      text: '效果预览'
    };

    if (!this.activeFilePath) return presetPreview;

    if (config.features?.useRawPreview) {
      if (this.rawBlobUrl) {
        return { type: 'raw', url: this.rawBlobUrl, text: '原图预览' };
      } else {
        return presetPreview;
      }
    }

    // 缓存 Key 也要包含具体 ID
    const cacheKey = `${this.activeFilePath}|${this.activePresetId}`;
    const resultData = this.processedFiles.get(cacheKey);

    return resultData 
      ? { type: 'result', url: resultData, text: '已生成' }
      : presetPreview;
  },

  // 获取下拉菜单选项 (从注册表读)
  get modeOptions() { 
    return frameRegistry.getOptions(); 
  },



  // =========================================
  // Actions (方法)
  // =========================================

  // 🟢 [新增] 设置自定义导出目录 (配合 open dialog)
  setExportPath(path) {
    this.exportSettings.customPath = path;
    this.exportSettings.pathMode = 'custom';
  },

  // 🟢 [核心 Action] 切换模式时，加载该模式的默认参数
  loadModeParams(presetId) {
    const config = frameRegistry.get(presetId);
    
    // 如果该模式定义了 defaultParams，就深拷贝一份过来
    if (config && config.defaultParams) {
      // 使用 JSON 序列化进行深拷贝，防止引用污染
      this.modeParams = JSON.parse(JSON.stringify(config.defaultParams));
    } else {
      this.modeParams = {}; // 该模式没有特殊参数
    }
    
    console.log(`[Store] 已加载 ${presetId} 参数:`, this.modeParams);
  },
  // 🟢 新增 Action：更新尺寸
  updateImageDimensions(w, h) {
    this.imageDimensions.width = w;
    this.imageDimensions.height = h;
  },
  // 🟢 切换大类 (Category) -> 比如从白底切到透明
  // 这通常由 Sidebar 或者顶部 Tab 触发
  async setCategory(newCategory) {
    this.isLoadingPresets = true;
    await new Promise(r => setTimeout(r, 300));

    this.settings.style = newCategory;
    
    // 切换大类后，自动选中该类下的第一个预设
    const presets = this.currentPresets;
    if (presets.length > 0) {
        this.activePresetId = presets[0].id;
    }

    this.isLoadingPresets = false;
  },

  // 🟢 切换具体预设 (Preset) -> 比如在面板里点击了"宝丽来"
  applyPreset(presetId) {
    if (this.activePresetId !== presetId) {
      this.activePresetId = presetId;
    }
  },

  // 🟢 加载本地原图 Blob (用于签名模式等)
  async loadPreviewBlob(filePath) {
    if (!filePath) return;
    this.cleanupBlob(); // 先清理旧的

    try {
      const bytes = await invoke('read_local_image_blob', { filePath });
      const byteArray = new Uint8Array(bytes);
      const blob = new Blob([byteArray], { type: 'image/jpeg' });
      this.rawBlobUrl = URL.createObjectURL(blob);
    } catch (e) {
      console.error("❌ Blob Load Error:", e);
      this.rawBlobUrl = null;
    }
  },

  // 🟢 清理 Blob 内存
  cleanupBlob() {
    if (this.rawBlobUrl) {
      URL.revokeObjectURL(this.rawBlobUrl);
      this.rawBlobUrl = null;
    }
  },

  // 🟢 缓存管理：标记文件处理完成
  markFileProcessedWithStyle(path, style, outPath) {
    const key = `${path}|${style}`;
    this.processedFiles.set(key, outPath);
  },
  
  // 🟢 缓存管理：清除特定文件的缓存状态
  clearProcessedStatusWithStyle(path, style) {
    const key = `${path}|${style}`;
    this.processedFiles.delete(key);
  },

  // 🟢 队列管理：添加文件
  addFiles(newFiles) {
    // 过滤重复文件
    const existingPaths = new Set(this.fileQueue.map(f => f.path));
    const uniqueFiles = newFiles.filter(f => !existingPaths.has(f.path));
    
    const formattedFiles = uniqueFiles.map(f => ({
      name: f.name,
      path: f.path,
      exifStatus: 'wait'
    }));
    
    this.fileQueue.push(...formattedFiles);
    
    // 如果当前没有选中文件，默认选中第一个
    if (!this.activeFilePath && this.fileQueue.length > 0) {
      this.activeFilePath = this.fileQueue[0].path;
    }
    return formattedFiles.length
  },

  // 🟢 队列管理：移除文件
  removeFile(index) {
    const fileToRemove = this.fileQueue[index];
    const isRemovingActive = fileToRemove && fileToRemove.path === this.activeFilePath;
    
    if (fileToRemove) {
      // 清理该文件相关的所有缓存，防止内存泄漏
      for (const [key] of this.processedFiles) {
        if (key.startsWith(`${fileToRemove.path}|`)) {
          this.processedFiles.delete(key);
        }
      }
    }

    this.fileQueue.splice(index, 1);

    // 如果删除了当前选中的文件，自动选中下一个
    if (isRemovingActive) {
      this.activeFilePath = this.fileQueue.length > 0 ? this.fileQueue[0].path : null;
    }
  },

  // 🟢 选中文件
  setActiveFile(path) {
    this.activeFilePath = path;
  },

  // 🟢 清空列表
  clearQueue() {
    this.cleanupBlob(); // 务必清理 Blob
    this.fileQueue = [];
    this.processedFiles.clear();
    this.activeFilePath = null;
    this.progress = { current: 0, total: 0, percent: 0 };
    this.statusText = "列表已清空";
    this.statusType = "normal";
  },

  // 🟢 状态提示更新
  setStatus(text, type = "normal") {
    this.statusText = text;
    this.statusType = type;
  },

  // 🟢 进度更新
  updateProgress(current, total) {
    this.progress.current = current;
    this.progress.total = total;
    this.progress.percent = total > 0 ? Math.round((current / total) * 100) : 0;
  }
});

// 🟢 [自动监听] 监听 activePresetId 变化，自动重置参数
// 放在 reactive 定义之外
watch(
  () => store.activePresetId, 
  (newId) => {
    store.loadModeParams(newId);
  },
  { immediate: true } // 初始化时立即执行一次
);

// 🟢 [自动监听] 监听 theme 变化，应用到 html 标签
watch(
  () => store.theme,
  (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme);
  },
  { immediate: true }
);