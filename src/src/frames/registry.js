// src/frames/registry.js
import { basicFrames } from './basic_frames';
import SignatureFrame from './signature'; 

const registry = {
  ...basicFrames,
  'Signature': SignatureFrame
};


// 🟢 [新思路] UI 专用：静态定义的分类选项列表
// 简单、直观、支持排序，不需要运行时计算
export const CATEGORY_OPTIONS = [
  { value: 'ClassicWhite', label: '⚪ 经典白底 (ClassicWhite)' },
  { value: 'Transparent', label: '🌫️ 透明磨砂 (Transparent)' },
  { value: 'Signature',    label: '✍️ 个性签名 (Signature)' },
  // 未来加新分类直接在这里加一行，简单明了
];

export const frameRegistry = {
  get(key) {
    const found = registry[key];
    if (!found) {
        // 🟢 3. 如果找不到，打印警告，看看试图找什么
        console.warn(`❌ [FrameRegistry] GET 失败: key="${key}"`);
        return { features: {} }; // 兜底
    }
    return found;
  },

  /**
   * 🟢 新增：根据大类(Category)获取所有属于该类的预设
   * 还原了旧版 PRESET_CONFIGS[style] 的功能
   */
  getByCategory(category) {
    return Object.entries(registry)
      .filter(([key, config]) => config.category === category)
      .map(([key, config]) => ({
        id: key, // 具体的预设 ID (e.g. 'WhitePolaroid')
        ...config
      }));
  },

  // 获取所有可用的大类 (用于顶部的 Tab 或下拉菜单切换大类)
  getCategories() {
    const categories = new Set();
    Object.values(registry).forEach(conf => {
      if(conf.category) categories.add(conf.category);
    });
    return Array.from(categories);
  }
};