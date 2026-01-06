export const config = {
  category: 'Signature', // 🟢 签名模式自成一派
  label: '个性签名',
  features: {
    useRawPreview: true // 开启 Blob 加载
  },
// 🟢 新增：定义该模式所需的默认参数
  // 这样，关于"签名模式需要什么参数"的知识，就完全封装在这里了
  defaultParams: {
    text: '',         // 对应之前的 signatureText (名字泛化一点更通用)
    fontScale: 0.04,
    bottomRatio: 0.04,
    color: '#FFFFFF'  // 以后如果要扩展颜色，直接加在这里
  },

  // 签名模式使用白底作为预设图
  getPresetUrl: () => new URL('../../assets/presets/white_standard.jpg', import.meta.url).href
};