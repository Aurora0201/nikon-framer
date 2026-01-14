use std::path::{ PathBuf};
use serde::Deserialize;
use crate::utils::calculate_target_path_core;

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
    // ===================================
    // 2. 🟢 带参数模式 (Struct Variants)
    // ===================================
    // 当前端传 "style": "Signature" 时，
    // Serde 会自动寻找同级字段 text, fontScale 等
    #[serde(rename_all = "camelCase")] 
    Signature {
        text: String,
        font_scale: f32,    // 对应 JSON: fontScale
        bottom_ratio: f32,  // 对应 JSON: bottomRatio
        // color: String,   // 预留: 如果以后要传颜色
    },
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
            // 🟢 签名模式的后缀
            Self::Signature { .. } => "Signature",
            // 以后新增样式，只需要在这里加一行
        }
    }

    // 🟢 新增：判断该模式是否“可编辑/参数敏感”
    // 如果是可编辑模式，就不应该进行“跳过重复文件”的检查，
    // 因为用户可能改了签名内容，即使文件名没变，也需要重新生成。
    pub fn is_editable(&self) -> bool {
        match self {
            Self::Signature { .. } => true, // 签名模式是可变的
            _ => false,                     // 其他模式是静态的
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

    // 🟢 [新增] 导出配置
    // 对应前端 JSON: { "options": { ... }, "export": { ... } }
    // 注意：前端传参时，建议把 exportSettings 改名为 export 传过来，或者这里用 #[serde(rename="exportSettings")]
    #[serde(rename="exportSettings")]
    pub export: ExportConfig,
}

// 🟢 3. 统一路径计算逻辑 (Single Source of Truth)
impl BatchContext {
    pub fn calculate_target_path(&self, original_file_path: &str) -> Result<PathBuf, String> {
        // 🟢 直接调用核心函数，传入自己的字段
        calculate_target_path_core(
            original_file_path, 
            &self.export, 
            &self.options
        )
    }
}


// 🟢 [新增] 导出配置结构体
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    // 目标文件夹：Some(路径) 代表自定义，None 代表原图同级
    pub target_dir: Option<String>, 
    // 格式：jpg, png
    pub format: ExportImageFormat, 
    // 质量：1-100 (仅 JPG 有效)
    pub quality: u8,
}


// 1. 定义支持的格式枚举
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")] 
pub enum ExportImageFormat {
    Jpg,
    Png,
    // 未来想支持 WebP，只需在这里加一行：
    // Webp, 
}

impl ExportImageFormat {
    // 获取扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            // Self::Webp => "webp",
        }
    }

    // 判断是否支持透明通道 (Alpha)
    pub fn supports_alpha(&self) -> bool {
        match self {
            Self::Jpg => false, // JPG 不支持，需要转 RGB
            Self::Png => true,
        }
    }
    
    // 可以在这里封装 MIME type
    pub fn mime_type(&self) -> &'static str {
         match self {
            Self::Jpg => "image/jpeg",
            Self::Png => "image/png",
        }
    }
}