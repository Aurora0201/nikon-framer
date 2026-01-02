import { reactive, computed } from 'vue';
// Tauri v2 使用 @tauri-apps/api/core，如果是 v1 请改为 @tauri-apps/api/tauri
import { convertFileSrc } from '@tauri-apps/api/core';

// 🟢 预设配置 (建议放在 src/assets/presets/ 下，这里为了演示路径写文件名)
// 注意：前端显示的图片 ID 必须与 Rust 枚举后缀逻辑对应
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
};

const MODE_OPTIONS = [
  { value: 'ClassicWhite', label: '经典白底 (ClassicWhite)' },
  { value: 'Transparent', label: '透明相框 (Transparent)' },
];


// 🟢 [核心修复] 使用 Glob 导入a
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
  activePresetId: 'WhiteClassic', // 默认选中 ID
  
  // 🟢 [新增] 预设加载状态 (用于控制 PresetPanel 的 loading 动画)
  isLoadingPresets: false,
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

  get previewSource() {
    const allPresets = [...PRESET_CONFIGS.ClassicWhite, ...PRESET_CONFIGS.Transparent];
    const currentConfig = allPresets.find(p => p.id === this.activePresetId);
    
    const presetPreview = {
      type: 'preset',
      url: currentConfig ? getPresetUrl(currentConfig.img) : null,
      text: '效果预览'
    };

    if (!this.activeFilePath) return presetPreview;

    // 🟢 [修复 1] 使用复合 Key 获取缓存
    // 只有当 "当前文件 + 当前模式" 都有结果时，才返回 Result
    const cacheKey = `${this.activeFilePath}|${this.activePresetId}`;
    const resultData = this.processedFiles.get(cacheKey);

    if (resultData) {
      return {
        type: 'result',
        url: resultData, 
        text: '已生成'
      };
    } else {
      return presetPreview;
    }
  },

  // --- Actions ---

  // 🟢 [修改] 切换大类模式 (支持 Loading 状态)
  async setMode(newMode) {
    // 1. 开始加载
    this.isLoadingPresets = true;

    // 2. (可选) 模拟一个微小的延迟，让 Loading 动画展示出来，提升交互质感
    // 如果未来这里变成 await invoke('get_presets_from_rust')，这个逻辑就非常有用了
    await new Promise(resolve => setTimeout(resolve, 300));

    // 3. 执行原有的切换逻辑
    this.settings.style = newMode;
    
    // 切换模式后，自动选中该模式下的第一个预设
    const presets = this.currentPresets;
    if (presets.length > 0) {
      this.applyPreset(presets[0]);
    } else {
      this.activePresetId = null;
    }

    // 4. 结束加载
    this.isLoadingPresets = false;
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

  // 🟢 [修复 2] 存入缓存时，带上 PresetId
  markFileProcessed(originalPath, outputPath) {
    // 注意：这里需要知道这张图是哪个模式生成的。
    // 在目前的逻辑里，Rust 生成完时，activePresetId 通常就是当前模式。
    // 如果支持后台批量生成，这里可能需要传 style 参数进来。
    // 假设目前是单张实时处理：
    const key = `${originalPath}|${this.activePresetId}`;
    this.processedFiles.set(key, outputPath);
  },
  
  // 重载版本：如果 Watcher 明确知道是检查哪个 style 的文件
  // 我们可以在 store 里加一个更明确的方法，或者让上面的方法支持第三个参数
  // 为了配合 Workspace.vue 中的 checkPreviewStatus:
  markFileProcessedWithStyle(originalPath, style, outputPath) {
    const key = `${originalPath}|${style}`;
    this.processedFiles.set(key, outputPath);
  },

  // 🟢 [修复 3] 清除缓存时，带上 PresetId
  clearProcessedStatus(originalPath) {
    // 默认清除当前模式的缓存
    const key = `${originalPath}|${this.activePresetId}`;
    if (this.processedFiles.has(key)) {
      this.processedFiles.delete(key);
    }
  },
  
  // 配合 Workspace.vue 的重载版本
  clearProcessedStatusWithStyle(originalPath, style) {
    const key = `${originalPath}|${style}`;
    if (this.processedFiles.has(key)) {
      this.processedFiles.delete(key);
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
      // 🟢 [修复 4] 移除文件时，要清理该文件对应的“所有模式”的缓存
      // Map 的遍历删除性能开销极小，直接遍历即可
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