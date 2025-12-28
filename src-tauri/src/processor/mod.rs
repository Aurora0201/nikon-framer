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

// 辅助函数：解析品牌字符串为枚举
fn parse_brand(make: &str) -> Option<Brand> {
    let m = make.to_lowercase();
    if m.contains("nikon") {
        Some(Brand::Nikon)
    } else if m.contains("sony") {
        Some(Brand::Sony)
    } else if m.contains("canon") {
        Some(Brand::Canon)
    } else if m.contains("fujifilm") || m.contains("fuji") {
        Some(Brand::Fujifilm)
    } else if m.contains("leica") {
        Some(Brand::Leica)
    } else if m.contains("hasselblad") {
        Some(Brand::Hasselblad)
    } else {
        None
    }
}

// ==========================================
// 策略 1: 白底处理器 (BottomWhite)
// ==========================================
struct BottomWhiteProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for BottomWhiteProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "白底模式: 标准字体解析失败")?;
        
        let brand = parse_brand(make);
        
        let assets = if let Some(b) = brand {
            match b {
                Brand::Nikon => WhiteStyleResources {
                    main_logo:  resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:   resources::get_logo(b, LogoType::SymbolZ),       
                    badge_icon: resources::get_logo(b, LogoType::IconYellowBox), 
                },
                _ => WhiteStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo: None,
                    badge_icon: None,
                }
            }
        } else {
            WhiteStyleResources { main_logo: None, sub_logo: None, badge_icon: None }
        };

        Ok(white::process(img, make, model, params, &font, "Bold", &assets))
    }
}

// ==========================================
// 策略 2: 模糊处理器 (Blur)
// ==========================================
pub struct TransparentClassicProcessor {
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for TransparentClassicProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        let brand = parse_brand(make);
        
        let assets = if let Some(b) = brand {
            match b {
                Brand::Nikon => BlurStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:  resources::get_logo(b, LogoType::SymbolZ),
                },
                Brand::Sony => BlurStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:  resources::get_logo(b, LogoType::SymbolAlpha),
                },
                _ => BlurStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo: None,
                }
            }
        } else {
            BlurStyleResources { main_logo: None, sub_logo: None }
        };
        let default_shadow = 150.0;
        
        Ok(blur::process(img, make, model, params, &font, "Bold", default_shadow, &assets))
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
    fn process(&self, img: &DynamicImage, _make: &str, _model: &str, params: &str) -> Result<DynamicImage, String> {
        
        let main = FontRef::try_from_slice(&self.main_font)
            .map_err(|_| "Master模式: 主字体解析失败".to_string())?;

        let script = FontRef::try_from_slice(&self.script_font)
            .unwrap_or_else(|_| {
                println!("⚠️ Master模式: 手写体解析失败，回退");
                main.clone()
            });

        let serif = FontRef::try_from_slice(&self.serif_font)
            .unwrap_or_else(|_| {
                println!("⚠️ Master模式: 标题字体解析失败，回退");
                main.clone()
            });

        let result_img = master::process(
            img, 
            params, 
            &main,   
            &script, 
            &serif   
        );

        Ok(result_img)
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
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        // 解析字体
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "Polaroid模式: 字体解析失败")?;

        // 1. 解析品牌
        let brand = parse_brand(make);

        // 2. 准备 PolaroidResources (适配器模式)
        // Polaroid 模式只需要一个 Logo，通常是 Wordmark (黑色字体)
        let assets = if let Some(b) = brand {
            PolaroidResources {
                logo: resources::get_logo(b, LogoType::Wordmark),
            }
        } else {
            PolaroidResources { logo: None }
        };
        
        // 3. 调用 polaroid::process_polaroid_style
        // 注意：这里使用 "Regular" 因为你要求的是 Regular 字体
        Ok(polaroid::process_polaroid_style(
            img, 
            make, 
            model, 
            params, 
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