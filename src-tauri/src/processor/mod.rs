pub mod white;
pub mod blur;
pub mod traits;
pub mod master;
pub mod polaroid; // 1. 确保已引入模块

use std::sync::Arc;
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
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

// --- 公共辅助结构与函数 ---

pub struct DrawContext<'a> {
    pub canvas: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub font: &'a FontRef<'a>,
    pub font_weight: &'a str,
}

pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

pub fn clean_model_name(make: &str, model: &str) -> String {
    let make_clean = make.replace("CORPORATION", "").trim().to_string(); 
    let model_upper = model.to_uppercase();
    let make_upper = make_clean.to_uppercase();
    
    // 提取型号主体
    let model_base = if let Some(idx) = model_upper.find(&make_upper) {
        let start = idx + make_upper.len();
        let rest = &model[start..];
        rest.trim().to_string()
    } else {
        model.to_string()
    }; 

    // 去除 NIKON 前缀
    let mut no_make = if model_base.to_uppercase().starts_with("NIKON") {
        model_base[5..].trim().to_string()
    } else {
        model_base
    };
    
    no_make = no_make.trim().to_string();
    
    // 去除 Z 前缀 (如果需要)
    if no_make.to_uppercase().starts_with("Z") {
        no_make = no_make[1..].trim().to_string();
    }
    
    no_make
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
            .map_err(|_| "白底模式: 标准字体解析失败")?;
        
        // 1. 获取资源：根据 Parser 解析出的 Brand 获取 Logo
        // 白底模式逻辑：Nikon 用 Wordmark + Z标(如果有)；其他品牌用 Wordmark
        let assets = match ctx.brand {
            Brand::Nikon => WhiteStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                // 这里的判断逻辑属于“排版策略”，Parser 只告诉我们需要什么，这里决定怎么用
                sub_logo: if ctx.model_name.contains("Z") { 
                    resources::get_logo(ctx.brand, LogoType::SymbolZ) 
                } else { None },
                badge_icon: resources::get_logo(ctx.brand, LogoType::IconYellowBox), 
            },
            Brand::Sony => WhiteStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                sub_logo: resources::get_logo(ctx.brand, LogoType::SymbolAlpha), // Sony 加个 Alpha 标
                badge_icon: None,
            },
            _ => WhiteStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                sub_logo: None,
                badge_icon: None,
            }
        };

        // 2. 格式化参数
        let params_str = ctx.params.format_standard();

        // 3. 调用旧的绘图函数 (桥接模式)
        // 注意：我们传的是 ctx.model_name (已经清洗过是 "Z 8" 而不是 "NIKON Z 8")
        // 以及 ctx.brand.to_string() (因为我们实现了 Display 特征)
        Ok(white::process(
            img, 
            &ctx.brand.to_string(), 
            &ctx.model_name, 
            &params_str, 
            &font, 
            "Bold", 
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

impl FrameProcessor for TransparentClassicProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        // 资源获取逻辑
        let assets = match ctx.brand {
            Brand::Nikon => BlurStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                sub_logo: if ctx.model_name.contains("Z") {
                    resources::get_logo(ctx.brand, LogoType::SymbolZ)
                } else { None },
            },
            Brand::Sony => BlurStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                sub_logo: resources::get_logo(ctx.brand, LogoType::SymbolAlpha),
            },
            _ => BlurStyleResources {
                main_logo: resources::get_logo(ctx.brand, LogoType::Wordmark),
                sub_logo: None,
            }
        };
        
        let params_str = ctx.params.format_standard();
        let default_shadow = 150.0;
        
        Ok(blur::process(
            img, 
            &ctx.brand.to_string(), 
            &ctx.model_name, 
            &params_str, 
            &font, 
            "Bold", 
            default_shadow, 
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

        let params_str = ctx.params.format_standard();

        Ok(master::process(
            img, 
            &params_str, 
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
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold),
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