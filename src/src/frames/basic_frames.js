// src/frames/basic_frames.js

// 1. 扫描所有图片建立字典
const presetAssets = import.meta.glob('../assets/presets/*.{png,jpg,jpeg,webp}', { 
  eager: true, 
  import: 'default' 
});

// 2. 智能查表函数
const getAssetUrl = (filename) => {
  if (!filename) return '';
  const allKeys = Object.keys(presetAssets);
  // 只要路径以 /filename 结尾就匹配，忽略 ../ 或 ../../ 的差异
  const foundKey = allKeys.find(key => key.endsWith(`/${filename}`));
  return foundKey ? presetAssets[foundKey] : '';
};

// 3. 工厂函数
const defineStaticFrame = (category, label, desc, imgFilename) => ({
  category, 
  label,
  desc,
  features: { useRawPreview: false }, 
  panelComponent: null,           
  layerComponent: null,             
  
  // 🟢 必须用这个，不能用 new URL(...)
  getPresetUrl: () => getAssetUrl(imgFilename)
});

export const basicFrames = {
  // ClassicWhite
  'WhiteClassic': defineStaticFrame('ClassicWhite', '经典白底', '经典系列，底部附带机型和摄影参数','white_classic.jpg'),
  'WhitePolaroid': defineStaticFrame('ClassicWhite', '宝丽来白', '经典系列，底部附带机型和摄影参数','white_polaroid.jpg'),
  'WhiteMaster': defineStaticFrame('ClassicWhite', '现代大师', '经典系列，底部附带机型和摄影参数','white_master.jpg'), 
  'WhiteModern': defineStaticFrame('ClassicWhite','现代白底', '经典系列，底部附带机型和摄影参数','white_modern.jpg'),

  // Transparent
  'TransparentClassic': defineStaticFrame('Transparent', '透明磨砂', '经典系列，底部附带机型和摄影参数','transparent_classic.jpg'),
  'TransparentMaster': defineStaticFrame('Transparent', '透明大师磨砂', '经典系列，底部附带机型和摄影参数','transparent_master.jpg'),
  
  // ... 其他
};