// 1. 声明子模块
pub mod white_classic;
pub mod transparent_classic;
pub mod traits;
pub mod transparent_master;
pub mod white_polaroid;
pub mod white_master;

// 2. 引入标准库与第三方库
use std::sync::Arc;
use image::{DynamicImage, imageops};
use ab_glyph::FontRef; 

// 3. 引入项目内部模块
use crate::models::StyleOptions;
use crate::processor::traits::FrameProcessor; 
use crate::parser::models::ParsedImageContext;

// 引入资源管理
use crate::resources::{self, FontFamily, FontWeight, LogoType};

// 引入各处理器的特定结构体 (Input & Resources)
use crate::processor::white_classic::WhiteStyleResources;
use crate::processor::transparent_classic::{BlurStyleResources, BlurInput};
use crate::processor::white_polaroid::{PolaroidResources, PolaroidInput};
use crate::processor::transparent_master::TransparentMasterInput;
// 🟢 引入 WhiteMaster 专用输入结构
use crate::processor::white_master::WhiteMasterInput;

// --- 公共辅助函数 ---

/// 根据高度调整图片大小 (保持长宽比)
pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

// ==========================================
// 策略 1: 极简白底处理器 (WhiteClassic)
// ==========================================
struct BottomWhiteProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for BottomWhiteProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "白底模式: 字体解析失败")?;
        
        // 资源准备
        let logo_type = LogoType::Wordmark;
        let logo_img = resources::get_logo(ctx.brand, logo_type);

        let assets = WhiteStyleResources {
            logo: logo_img,
        };

        let params_str = ctx.params.format_standard();

        // 调用处理逻辑
        Ok(white_classic::process(
            img, 
            &ctx.brand.to_string(), 
            &ctx.model_name,        
            &params_str,            
            &font, 
            &assets                 
        ))
    }
}

// ==========================================
// 策略 2: 经典透明处理器 (TransparentClassic)
// ==========================================
pub struct TransparentClassicProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for TransparentClassicProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        let assets = BlurStyleResources {
            logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
        };
        
        let params_str = ctx.params.format_standard();
        
        let input = BlurInput {
            brand: &ctx.brand.to_string(),
            model: &ctx.model_name,
            params: &params_str,
        };
        
        Ok(transparent_classic::process(
            img, 
            &font, 
            input, 
            &assets
        ))
    }
}

// ==========================================
// 策略 3: 大师透明处理器 (TransparentMaster)
// ==========================================
pub struct TransparentMasterProcessor {
    pub main_font: Arc<Vec<u8>>,   // 参数字体
    pub script_font: Arc<Vec<u8>>, // 手写体
    pub serif_font: Arc<Vec<u8>>,  // 标题体
}

impl FrameProcessor for TransparentMasterProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let main = FontRef::try_from_slice(&self.main_font).unwrap();
        let script = FontRef::try_from_slice(&self.script_font).unwrap();
        let serif = FontRef::try_from_slice(&self.serif_font).unwrap();

        // 构造输入数据
        let input = TransparentMasterInput {
            iso: ctx.params.iso.map(|v| v.to_string()).unwrap_or_default(),
            aperture: ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default(),
            shutter: ctx.params.shutter_speed
                .replace("s", "")
                .trim()
                .to_string(),
            focal: ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default(),
        };

        Ok(transparent_master::process(
            img, 
            input, 
            &main, 
            &script, 
            &serif
        ))
    }
}

// ==========================================
// 策略 4: 拍立得白底处理器 (WhitePolaroid)
// ==========================================
pub struct WhitePolaroidProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for WhitePolaroidProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "Polaroid模式: 字体解析失败")?;

        let assets = PolaroidResources {
            logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
        };
        
        let params_str = ctx.params.format_standard();

        let input = PolaroidInput {
            brand: &ctx.brand.to_string(),
            model: &ctx.model_name,
            params: &params_str,
        };

        Ok(white_polaroid::process(
            img, 
            &font, 
            input, 
            &assets
        ))
    }
}

// ==========================================
// 策略 5: 大师白底处理器 (WhiteMaster)
// ==========================================
pub struct WhiteMasterProcessor {
    pub main_font: Arc<Vec<u8>>,   // 参数字体
    pub script_font: Arc<Vec<u8>>, // 手写体
    pub serif_font: Arc<Vec<u8>>,  // 标题体
}

impl FrameProcessor for WhiteMasterProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let main = FontRef::try_from_slice(&self.main_font)
            .map_err(|_| "WhiteMaster: 参数字体解析失败")?;
        let script = FontRef::try_from_slice(&self.script_font)
            .map_err(|_| "WhiteMaster: 手写字体解析失败")?;
        let serif = FontRef::try_from_slice(&self.serif_font)
            .map_err(|_| "WhiteMaster: 衬线字体解析失败")?;

        // 🟢 使用 WhiteMasterInput 构造输入数据
        let input = WhiteMasterInput {
            iso: ctx.params.iso.map(|v| v.to_string()).unwrap_or_default(),
            aperture: ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default(),
            // 清洗快门速度字符串 (去除 's', 去除空格)
            shutter: ctx.params.shutter_speed
                .replace("s", "")
                .trim()
                .to_string(),
            focal: ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default(),
        };

        // 调用 white_master 模块的处理逻辑
        Ok(white_master::process(
            img, 
            input, 
            &main, 
            &script, 
            &serif
        ))
    }
}

// ==========================================
// 工厂函数: 核心装配车间
// ==========================================
pub fn create_processor(options: &StyleOptions) -> Box<dyn FrameProcessor + Send + Sync> {
    match options {
        
        // 1. 极简白底模式
        StyleOptions::WhiteClassic => {
            Box::new(BottomWhiteProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold) 
            })
        },

        // 2. 高斯模糊模式
        StyleOptions::TransparentClassic => {
            Box::new(TransparentClassicProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
            })
        },

        // 3. 大师透明模式
        StyleOptions::TransparentMaster => {
            Box::new(TransparentMasterProcessor {
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },

        // 4. 拍立得模式
        StyleOptions::WhitePolaroid => {
            Box::new(WhitePolaroidProcessor {
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
            })
        },

        // 5. 大师白底模式 (🟢 新增)
        StyleOptions::WhiteMaster => {
            Box::new(WhiteMasterProcessor {
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },

    }
}