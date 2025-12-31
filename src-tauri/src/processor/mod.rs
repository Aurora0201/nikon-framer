pub mod white;
pub mod blur;
pub mod traits;
pub mod master;
pub mod polaroid; // 1. 确保已引入模块

use std::sync::Arc;
use image::{DynamicImage, imageops};
use ab_glyph::FontRef; 

use crate::models::StyleOptions;
use crate::processor::traits::FrameProcessor; 
// 🟢 引入 parser 的数据结构
use crate::parser::models::ParsedImageContext;
// 引入资源模块
use crate::resources::{self, Brand, FontFamily, FontWeight, LogoType};
// 引入各个子模块的特定资源结构体
use crate::processor::white::WhiteStyleResources;
use crate::processor::blur::BlurStyleResources;
use crate::processor::polaroid::PolaroidResources; // 2. 引入 PolaroidResources
use crate::processor::blur::BlurInput; // 🟢 引入新结构体
use crate::processor::master::MasterInput;

// --- 公共辅助结构与函数 ---

pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

// ==========================================
// 策略 1: 白底处理器 (BottomWhite)
// ==========================================
struct BottomWhiteProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for BottomWhiteProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "白底模式: 字体解析失败")?;
        
        // // 1. 获取正确的 Logo
        // let logo_type = if ctx.brand == Brand::Nikon {
        //     LogoType::IconYellowBox
        // } else {
        //     LogoType::Wordmark
        // };
        let logo_type= LogoType::Wordmark;
        let logo_img = resources::get_logo(ctx.brand, logo_type);

        // 2. 组装精简后的资源包
        let assets = WhiteStyleResources {
            logo: logo_img, // 🟢 只有这一个字段了
        };

        let params_str = ctx.params.format_standard();

        // 3. 调用新版接口
        Ok(white::process(
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
// 策略 2: 模糊处理器 (Blur)
// ==========================================
pub struct TransparentClassicProcessor {
    pub font_data: Arc<Vec<u8>>,
}

// 实现
impl FrameProcessor for TransparentClassicProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        // 资源获取 (保持你之前的修改：只取 Wordmark)
        let assets = BlurStyleResources {
            logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
        };
        
        let params_str = ctx.params.format_standard();
        
        // 🟢 2. 构造参数包
        let input = BlurInput {
            brand: &ctx.brand.to_string(),
            model: &ctx.model_name,
            params: &params_str,
        };
        
        // 🟢 3. 调用新接口
        Ok(blur::process(
            img, 
            &font, 
            input, 
            &assets
        ))
    }
}
// ==========================================
// 策略 3: 大师处理器 (Master)
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

        // 🟢 2. 数据转换：从 ctx.params 提取并清洗数据
        let input = MasterInput {
            // ISO: Option<u32> -> String
            iso: ctx.params.iso.map(|v| v.to_string()).unwrap_or_default(),
            
            // 光圈: Option<f32> -> String
            aperture: ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default(),
            
            // 🔴 修复点：既然编译器说 shutter_speed 是 String，就直接处理
            // 移除 .map() 和 .unwrap_or_default()
            // 如果你的 shutter_speed 确实是 Option<String> 但报错，请尝试下方的【备选方案】
            shutter: ctx.params.shutter_speed
                .replace("s", "")
                .trim()
                .to_string(),
                
            // 焦距: Option<u32> -> String
            focal: ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default(),
        };

        // 🟢 3. 调用新接口
        Ok(master::process(
            img, 
            input, 
            &main, 
            &script, 
            &serif
        ))
    }
}

// ==========================================
// 策略 4: 拍立得/极简白框处理器 (Polaroid)
// ==========================================
// 3. 新增 PolaroidProcessor 结构体
pub struct PolaroidProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for PolaroidProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "Polaroid模式: 字体解析失败")?;

        let assets = PolaroidResources {
            logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
        };
        
        let params_str = ctx.params.format_standard();

        Ok(polaroid::process_polaroid_style(
            img, 
            &ctx.brand.to_string(), 
            &ctx.model_name, 
            &params_str, 
            &font, 
            "Regular", 
            &assets
        ))
    }
}

// ==========================================
// 工厂函数: 核心装配车间
// ==========================================
pub fn create_processor(options: &StyleOptions) -> Box<dyn FrameProcessor + Send + Sync> {
    match options {
        
        // 极简白底模式
        StyleOptions::BottomWhite => {
            Box::new(BottomWhiteProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold) 
            })
        },

        // 高斯模糊模式
        StyleOptions::TransparentClassic => {
            Box::new(TransparentClassicProcessor { 
                // 🟢 1. 统一使用 Medium 字体
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
            })
        },

        // 大师模式
        StyleOptions::TransparentMaster => {
            Box::new(TransparentMasterProcessor {
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },

        // 4. 注册 PolaroidWhite 模式
        // 修复：之前这里错误地初始化了 TransparentMasterProcessor
        // 现在正确初始化 PolaroidProcessor 并使用 InterDisplay-Regular
        StyleOptions::PolaroidWhite => {
            Box::new(PolaroidProcessor {
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Regular),
            })
        },
    }
}