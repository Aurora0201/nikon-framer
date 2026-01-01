import { reactive, computed } from 'vue';
// Tauri v2 使用 @tauri-apps/api/core，如果是 v1 请改为 @tauri-apps/api/tauri
import { convertFileSrc } from '@tauri-apps/api/core';

// 🟢 预设配置 (建议放在 src/assets/presets/ 下，这里为了演示路径写文件名)
// 注意：前端显示的图片 ID 必须与 Rust 枚举后缀逻辑对应
const PRESET_CONFIGS = {
  ClassicWhite: [
    { id: 'WhiteClassic', name: 'Standard White', desc: '标准白底 / 简约风格', img: 'white_standard.jpg' },
    { id: 'WhitePolaroid', name: 'Polaroid White', desc: '宝丽来风格', img: 'polaroid_white.jpg' },
  ],
  Transparent: [
    { id: 'TransparentMaster', name: 'Glass Blur', desc: '大师风格 / 模糊', img: 'transparent_standard.jpg' },
    { id: 'TransparentClassic', name: 'Classic Blur', desc: '经典效果 / 模糊', img: 'transparent_classic.jpg' },
  ],
};

const MODE_OPTIONS = [
  { value: 'ClassicWhite', label: '经典白底 (ClassicWhite)' },
  { value: 'Transparent', label: '透明相框 (Transparent)' },
];


// 🟢 [核心修复] 使用 Glob 导入
// 1. eager: true 表示直接加载路径字符串，而不是返回 Promise
// 2. import: 'default' 确保直接拿到图片 URL
// 3. 注意：这里的路径 './assets/presets/*' 必须是相对于 store.js 的准确路径！
const presetAssets = import.meta.glob('./assets/presets/*.{png,jpg,jpeg,svg}', { 
  eager: true, 
  import: 'default' 
});

// 🟢 [核心修复] 查表获取路径
const getPresetUrl = (filename) => {
  // 构造 Key，必须和上面 glob 里的路径匹配
  // 如果 store.js 在 src/，assets 在 src/assets，则 key 应该是 ./assets/presets/xxx.jpg
  const key = `./assets/presets/${filename}`;
  
  const foundUrl = presetAssets[key];
  
  if (!foundUrl) {
    console.warn(`⚠️ [资源丢失] 找不到预设图: ${key}`);
    // 打印一下所有可用的 key，方便调试
    // console.log("可用列表:", Object.keys(presetAssets));
    return '';
  }
  
  return foundUrl;
};

export const store = reactive({
  // --- 核心状态 ---
  fileQueue: [],
  activeFilePath: null,
  activePresetId: 'BottomWhite', // 默认选中 ID
  
  // 🟢 [新增] 结果映射表：Key=原图路径, Value=处理后的路径
  processedFiles: new Map(),

  isProcessing: false,
  isDragging: false,
  progress: { current: 0, total: 0, percent: 0 },
  statusText: "准备就绪",
  statusType: "normal",
  
  settings: {
    style: 'ClassicWhite', // 当前大类
    shadowIntensity: 40,
    paddingRatio: 10,
  },

  // --- Getters (计算属性) ---

  get modeOptions() { return MODE_OPTIONS; },

  get currentPresets() { return PRESET_CONFIGS[this.settings.style] || []; },

  // 🟢 [核心修改] 智能计算当前预览图 URL
  get previewSource() {
    // 1. 先找到当前选中的预设配置 (为了拿 img 文件名)
    const allPresets = [...PRESET_CONFIGS.ClassicWhite, ...PRESET_CONFIGS.Transparent];
    const currentConfig = allPresets.find(p => p.id === this.activePresetId);
    
    // 准备默认的预设预览对象 (兜底)
    const presetPreview = {
      type: 'preset',
      url: currentConfig ? getPresetUrl(currentConfig.img) : null,
      text: '效果预览'
    };

    // 2. 如果没有选文件，直接显示预设
    if (!this.activeFilePath) {
      return presetPreview;
    }

    // ---------------------------------------------------------
    // 🔴 你的报错是因为缺少了下面这一行定义！
    // 必须先从 Map 中获取数据，赋值给 resultData 变量
    // ---------------------------------------------------------
    const resultData = this.processedFiles.get(this.activeFilePath);

    // 3. 检查是否有结果
    if (resultData) {
      // ✅ 情况 A: 有结果 -> 显示真实结果 (Base64)
      return {
        type: 'result',
        // resultData 现在是 "data:image/jpeg;base64,..."，直接用
        url: resultData, 
        text: '已生成'
      };
    } else {
      // ❌ 情况 B: 没结果 -> 显示预设图
      return presetPreview;
    }
  },

  // --- Actions ---

  // 切换大类模式
  setMode(newMode) {
    this.settings.style = newMode;
    // 切换模式后，自动选中该模式下的第一个预设
    const presets = this.currentPresets;
    if (presets.length > 0) {
      this.applyPreset(presets[0]);
    } else {
      this.activePresetId = null;
    }
  },

  // 切换具体预设
  applyPreset(preset) {
    if (this.activePresetId !== preset.id) {
        this.activePresetId = preset.id;
        // 🟢 切换预设意味着之前的预览结果(如果有)不再适用当前效果
        // 我们不在这里强制删除，而是依赖 WorkspacePanel 的 Watcher 去问 Rust
        // 如果 Rust 说新模式下没文件，Watcher 会调用 clearProcessedStatus，界面就会自动变回预设图
    }
  },

  // 🟢 [新增] 标记某张图已处理 (Rust 生成成功后调用)
  markFileProcessed(originalPath, outputPath) {
    this.processedFiles.set(originalPath, outputPath);
  },

  // 🟢 [新增] 清除某张图的处理状态 (Watcher 发现文件不存在时调用)
  clearProcessedStatus(originalPath) {
    if (this.processedFiles.has(originalPath)) {
      this.processedFiles.delete(originalPath);
    }
  },

  // --- 文件列表操作 (保持原有逻辑) ---
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
    
    // 移除文件时，也要清理掉它的缓存状态
    if (fileToRemove) {
      this.processedFiles.delete(fileToRemove.path);
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
    this.fileQueue = [];
    this.processedFiles.clear(); // 清空所有缓存
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