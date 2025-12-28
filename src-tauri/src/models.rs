use serde::Deserialize;

// 字体配置（公用）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontConfig {
    pub filename: String,
    pub weight: String,
}

// 🟢 核心改变：使用 Enum 定义样式配置
// Serde 的 tag = "style" 会自动根据 JSON 里的 "style" 字段决定解析成哪个变体
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "style", rename_all = "PascalCase")] 
pub enum StyleOptions {
    // 变体 1：白底模式 (只关心字体)
    #[serde(rename_all = "camelCase")] // 🟢 必须加在这里！
    BottomWhite,

    // 变体 2：高斯模糊 (关心字体 + 阴影)
    #[serde(rename_all = "camelCase")] // 🟢 必须加在这里！
    TransparentClassic {
        shadow_intensity: f32, // 只有这个模式有阴影参数
    },

    // 🟢 [新增] 大师模式
    // 参数几乎和 GaussianBlur 一样，因为它们都是模糊背景
    #[serde(rename_all = "camelCase")]
    TransparentMaster,

}

// 总配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")] // 🟢 必须加在这里！
pub struct BatchContext {
    // 🟢 这里不再是 String，而是上面定义的枚举
    // 前端传来的 JSON 必须包含 "style": "BottomWhite" 等字段
    #[serde(flatten)] // 将 style 字段拉平
    pub options: StyleOptions, 
}