import { reactive, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 🟢 1. 配置定义
const PRESET_CONFIGS = {
  ClassicWhite: [
    { id: 'WhiteClassic', name: 'Standard White', desc: '标准白底 / 简约风格', img: 'white_standard.jpg' },
    { id: 'WhitePolaroid', name: 'Polaroid White', desc: '宝丽来风格', img: 'polaroid_white.jpg' },
    { id: 'WhiteMaster', name: 'Master White', desc: '大师风格', img: 'polaroid_white.jpg' },
    { id: 'WhiteModern', name: 'Modern White', desc: '现代风格', img: 'polaroid_white.jpg' },
  ],
  Transparent: [
    { id: 'TransparentMaster', name: 'Glass Blur', desc: '大师风格 / 模糊', img: 'transparent_standard.jpg' },
    { id: 'TransparentClassic', name: 'Classic Blur', desc: '经典效果 / 模糊', img: 'transparent_classic.jpg' },
  ],
  Signature: [
    { id: 'SignatureMode', name: 'Signature', desc: '个性签名 / 手写体', img: 'white_standard.jpg' } 
  ]
};

const MODE_OPTIONS = [
  { value: 'ClassicWhite', label: '经典白底 (ClassicWhite)' },
  { value: 'Transparent', label: '透明相框 (Transparent)' },
  { value: 'Signature', label: '个性签名 (Signature)' },
];


// 🟢 2. 资源预加载 (Vite Glob Import)
// 注意：这里的路径 ./assets 必须相对于 store.js 的位置
const presetAssets = import.meta.glob('./assets/presets/*.{png,jpg,jpeg,svg}', { 
  eager: true, 
  import: 'default' 
});

const getPresetUrl = (filename) => {
  const key = `./assets/presets/${filename}`;
  return presetAssets[key] || '';
};

// 🟢 3. Store 定义
export const store = reactive({
  // --- 核心状态 ---
  fileQueue: [],
  activeFilePath: null,
  activePresetId: 'WhiteClassic',
  
  // 存储由 Rust 传来的二进制图片生成的 Blob URL
  rawBlobUrl: null, 

  isLoadingPresets: false,
  processedFiles: new Map(),

  isProcessing: false,
  isDragging: false,
  progress: { current: 0, total: 0, percent: 0 },
  statusText: "准备就绪",
  statusType: "normal",
  
  // 通用参数槽
  customParams: {
    signatureText: '', 
  },

  settings: {
    style: 'ClassicWhite', 
    shadowIntensity: 40,
    paddingRatio: 10,
  },

  // --- Getters ---

  // 🟢 [新增] 辅助方法：根据传入的 style 名称获取配置
  // 解决了 usePreviewLogic 无法访问 MODE_METADATA 的问题
  getModeConfig(style) {
    return MODE_METADATA[style] || { features: {}, controls: [], layers: [] };
  },
  
  get modeOptions() { return MODE_OPTIONS; },

  get currentPresets() { return PRESET_CONFIGS[this.settings.style] || []; },

  get previewSource() {
    const allPresets = Object.values(PRESET_CONFIGS).flat();
    const currentConfig = allPresets.find(p => p.id === this.activePresetId);
    
    const presetPreview = {
      type: 'preset',
      url: currentConfig ? getPresetUrl(currentConfig.img) : null,
      text: '效果预览'
    };

    if (!this.activeFilePath) return presetPreview;

    // 🟢 [重构] 不再检查 style === 'Signature'
    // 而是检查 "是否具备使用 RawPreview 的能力"
    if (this.currentModeConfig.features.useRawPreview) {
      if (this.rawBlobUrl) {
        return { type: 'raw', url: this.rawBlobUrl, text: '原图预览' };
      } else {
        return presetPreview;
      }
    }

    // 缓存结果逻辑
    const cacheKey = `${this.activeFilePath}|${this.activePresetId}`;
    const resultData = this.processedFiles.get(cacheKey);

    if (resultData) {
      return { type: 'result', url: resultData, text: '已生成' };
    } else {
      return presetPreview;
    }
  },

  // 🟢 获取当前模式的元数据 (核心 Getter)
  get currentModeConfig() {
    // 默认为空配置，防止报错
    // 🔍 调试日志：看看究竟拿到了什么
    const config = MODE_METADATA[this.settings.style];
    console.log(`[Store] Mode: ${this.settings.style}, Config:`, config);

    return MODE_METADATA[this.settings.style] || { features: {}, controls: [], layers: [] };
  },
  // --- Actions ---

  async setMode(newMode) {
    this.isLoadingPresets = true;
    // 模拟微小延迟
    await new Promise(resolve => setTimeout(resolve, 100));

    this.settings.style = newMode;
    
    const presets = this.currentPresets;
    if (presets.length > 0) {
      this.applyPreset(presets[0]);
    } else {
      this.activePresetId = null;
    }

    this.isLoadingPresets = false;
  },

  applyPreset(preset) {
    if (this.activePresetId !== preset.id) {
        this.activePresetId = preset.id;
    }
  },

  // 加载本地图片的 Blob (核心新功能)
  async loadPreviewBlob(filePath) {
    if (!filePath) return;

    this.cleanupBlob();

    try {
      // 调用 Rust 命令
      const bytes = await invoke('read_local_image_blob', { filePath });
      const byteArray = new Uint8Array(bytes);
      const blob = new Blob([byteArray], { type: 'image/jpeg' });
      this.rawBlobUrl = URL.createObjectURL(blob);
    } catch (e) {
      console.error("❌ 图片 Blob 加载失败:", e);
      this.rawBlobUrl = null; 
    }
  },

  // 清理内存
  cleanupBlob() {
    if (this.rawBlobUrl) {
      URL.revokeObjectURL(this.rawBlobUrl); 
      this.rawBlobUrl = null;
    }
  },

  markFileProcessedWithStyle(originalPath, style, outputPath) {
    const key = `${originalPath}|${style}`;
    this.processedFiles.set(key, outputPath);
  },

  clearProcessedStatusWithStyle(originalPath, style) {
    const key = `${originalPath}|${style}`;
    if (this.processedFiles.has(key)) {
      this.processedFiles.delete(key);
    }
  },

  addFiles(newFiles) {
    const existingPaths = new Set(this.fileQueue.map(f => f.path));
    const uniqueFiles = newFiles.filter(f => !existingPaths.has(f.path));
    
    const formattedFiles = uniqueFiles.map(f => ({
      name: f.name,
      path: f.path,
      exifStatus: 'wait'
    }));
    
    this.fileQueue.push(...formattedFiles);

    if (!this.activeFilePath && this.fileQueue.length > 0) {
      this.activeFilePath = this.fileQueue[0].path;
    }
    return uniqueFiles.length; 
  },

  removeFile(index) {
    const fileToRemove = this.fileQueue[index];
    const isRemovingActive = fileToRemove && fileToRemove.path === this.activeFilePath;
    
    if (fileToRemove) {
      for (const [key] of this.processedFiles) {
        if (key.startsWith(`${fileToRemove.path}|`)) {
          this.processedFiles.delete(key);
        }
      }
    }
    this.fileQueue.splice(index, 1);

    if (isRemovingActive) {
      this.activeFilePath = this.fileQueue.length > 0 ? this.fileQueue[0].path : null;
    }
  },

  setActiveFile(path) {
    this.activeFilePath = path;
  },

  clearQueue() {
    this.cleanupBlob(); // 清空队列时释放内存
    this.fileQueue = [];
    this.processedFiles.clear(); 
    this.activeFilePath = null;
    this.progress = { current: 0, total: 0, percent: 0 };
    this.statusText = "列表已清空";
  },

  setStatus(text, type = "normal") {
    this.statusText = text;
    this.statusType = type;
  },

  updateProgress(current, total) {
    this.progress.current = current;
    this.progress.total = total;
    this.progress.percent = total > 0 ? Math.round((current / total) * 100) : 0;
  }
});