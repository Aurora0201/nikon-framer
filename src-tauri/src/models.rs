use serde::Deserialize;

// 字体配置（公用）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
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
    WhiteClassic,

    #[serde(rename_all = "camelCase")] // 🟢 必须加在这里！
    WhitePolaroid,

    // 大师白底 (WhiteMaster)
    #[serde(rename_all = "camelCase")]
    WhiteMaster,

    // 变体 2：高斯模糊 (关心字体 + 阴影)
    #[serde(rename_all = "camelCase")] // 🟢 必须加在这里！
    TransparentClassic,

    // 🟢 [新增] 大师模式
    // 参数几乎和 GaussianBlur 一样，因为它们都是模糊背景
    #[serde(rename_all = "camelCase")]
    TransparentMaster,

    #[serde(rename_all = "camelCase")]
    WhiteModern, // 🟢 新增

}

// 🟢 新增：为枚举实现方法
impl StyleOptions {
    pub fn filename_suffix(&self) -> &'static str {
        match self {
            Self::WhiteClassic => "WhiteClassic",      // 对应生成 xxx_White.jpg
            Self::TransparentClassic => "TransparentClassic", // 对应生成 xxx_Blur.jpg
            Self::TransparentMaster => "TransparentMaster",// 对应生成 xxx_Master.jpg
            Self::WhitePolaroid => "WhitePolaroid",
            Self::WhiteMaster => "WhiteMaster",
            Self::WhiteModern => "WhiteModern",
            // 以后新增样式，只需要在这里加一行
        }
    }
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