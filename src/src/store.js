import { reactive } from 'vue';

// 🟢 定义所有可用的边框模式
// 这里的 value 必须严格对应后端 Rust Enum 的 Variant 名称
// 这里的 value 也必须对应 PRESET_CONFIGS 的 key
const MODE_OPTIONS = [
  { value: 'ClassicWhite', label: '经典白底 (ClassicWhite)' },
  { value: 'Transparent', label: '透明相框 (Transparent)' },
  // 未来扩展非常容易：
  // { value: 'Master', label: '大师水印 (Master)' },
];
// 🟢 将配置数据提取到 Store 外部或内部均可，这里为了整洁放在 Store 定义中
// 🟢 关键：这里的 id 就是发给后端的 style 参数
const PRESET_CONFIGS = {
  // === 白底模式 ===
  ClassicWhite: [
    {
      id: 'BottomWhite', // 👈 后端收到 { style: "white_std" }
      name: 'Standard White',
      desc: '标准白底 / 简约风格',
      img: 'white_standard.jpg',
      params: { paddingRatio: 10, shadowIntensity: 0 }
    },
  ],
  
  // === 透明模式 ===
  Transparent: [
    {
      id: 'GaussianBlur', // 👈 后端收到 { style: "trans_std" }
      name: 'Glass / Blur',
      desc: '大师风格 / 背景模糊',
      img: 'transparent_standard.jpg',
      params: { shadowIntensity: 60, blurRadius: 20 }
    },
    {
      id: 'Master', // 👈 后端收到 { style: "trans_std" }
      name: 'Glass / Blur',
      desc: '经典效果 / 背景模糊',
      img: 'transparent_classic.jpg',
      params: { shadowIntensity: 60, blurRadius: 20 }
    },
  ],
};


export const store = reactive({
  // ... 状态数据保持不变 ...
  fileQueue: [],
  activeFilePath: null, 
  activePresetId: null,
  isProcessing: false,
  isDragging: false,
  progress: { current: 0, total: 0, percent: 0 },
  statusText: "准备就绪",
  statusType: "normal",
  settings: {
    style: 'ClassicWhite',
    shadowIntensity: 40,
    paddingRatio: 10,
  },

// 🟢 [新增] 暴露模式选项列表给 UI 组件使用
  get modeOptions() {
    return MODE_OPTIONS;
  },

  // 🟢 获取当前模式下的预设列表
  get currentPresets() {
    return PRESET_CONFIGS[this.settings.style] || [];
  },

  // 应用预设
  applyPreset(preset) {
    console.log(`Store 应用预设: ${preset.name}`);
    this.activePresetId = preset.id;
    if (preset.params) {
      Object.assign(this.settings, preset.params);
    }
  },
  
  // 🟢 [新增] 动态获取当前模式下的预设列表
  // 使用 Getter 语法，像计算属性一样自动更新
  get currentPresets() {
    return PRESET_CONFIGS[this.settings.style] || [];
  },


  // --- 动作 (Actions) ---
  
  // 🟢 [新增] 应用预设 (核心业务逻辑)
  applyPreset(preset) {
    console.log(`Store 应用预设: ${preset.name}`);
    
    // 1. 设置选中状态
    this.activePresetId = preset.id;
    
    // 2. 将预设参数覆盖到全局设置 (UI 滑块会跟着动)
    if (preset.params) {
      Object.assign(this.settings, preset.params);
    }
  },
  // --- 动作 (Actions) ---
  
  // 🟢 1. 智能添加文件
  addFiles(newFiles) {
    const existingPaths = new Set(this.fileQueue.map(f => f.path));
    
    // 过滤去重
    const uniqueFiles = newFiles.filter(f => !existingPaths.has(f.path));
    
    // 统一格式化 (Store 负责初始化数据状态)
    const formattedFiles = uniqueFiles.map(f => ({
      name: f.name,
      path: f.path,
      exifStatus: 'wait' // 统一在这里定义初始状态
    }));
    
    this.fileQueue.push(...formattedFiles);

    // 自动选中逻辑：如果当前没有选中文件，且添加了新文件，默认选中第一个
    if (!this.activeFilePath && this.fileQueue.length > 0) {
      this.activeFilePath = this.fileQueue[0].path;
    }

    return uniqueFiles.length; 
  },

  // 🟢 2. 智能移除文件
  removeFile(index) {
    // 先判断要删除的是不是当前选中的文件
    const fileToRemove = this.fileQueue[index];
    const isRemovingActive = fileToRemove && fileToRemove.path === this.activeFilePath;

    // 删除
    this.fileQueue.splice(index, 1);

    // 如果删除了当前选中的，自动修补选中状态
    if (isRemovingActive) {
      // 如果列表还有文件，选中列表头；否则置空
      this.activeFilePath = this.fileQueue.length > 0 ? this.fileQueue[0].path : null;
    }
  },

  // 设置当前激活的文件
  setActiveFile(path) {
    this.activeFilePath = path;
  },

  // 🟢 3. 彻底清空
  clearQueue() {
    this.fileQueue = [];
    this.activeFilePath = null; // 数据层负责重置选中
    this.progress = { current: 0, total: 0, percent: 0 };
    this.statusText = "列表已清空";
    this.statusType = "normal";
  },

  // ... setStatus, updateProgress 保持不变 ...
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