pub mod white;
pub mod blur;
pub mod traits;
pub mod master;

use std::sync::Arc;
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use ab_glyph::FontRef; 

use crate::models::StyleOptions;
// 假设你在 traits.rs 里定义了 FrameProcessor，如果叫 FrameProcessor 请自行替换
use crate::processor::traits::FrameProcessor; 

// 引入重构后的 resources 模块
use crate::resources::{self, FontFamily, FontWeight, Brand, LogoType};
use crate::processor::white::WhiteStyleResources;
use crate::processor::blur::BlurStyleResources;

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

// 🟢 辅助函数：解析品牌字符串为枚举
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
        
        // 🟢 1. 解析品牌并获取资源
        let brand = parse_brand(make);
        
        let assets = if let Some(b) = brand {
            match b {
                Brand::Nikon => WhiteStyleResources {
                    main_logo:  resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:   resources::get_logo(b, LogoType::SymbolZ),       
                    badge_icon: resources::get_logo(b, LogoType::IconYellowBox), 
                },
                Brand::Sony => WhiteStyleResources {
                    main_logo:  resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:   resources::get_logo(b, LogoType::SymbolAlpha),   
                    badge_icon: None, 
                },
                Brand::Leica => WhiteStyleResources {
                    main_logo:  resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:   None,
                    badge_icon: resources::get_logo(b, LogoType::IconRedDot),    
                },
                Brand::Canon => WhiteStyleResources {
                    main_logo:  resources::get_logo(b, LogoType::Wordmark),
                    sub_logo:   None,
                    badge_icon: None,
                },
                // 其他品牌只显示主标
                _ => WhiteStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo: None,
                    badge_icon: None,
                }
            }
        } else {
            // 未知品牌，空资源
            WhiteStyleResources { main_logo: None, sub_logo: None, badge_icon: None }
        };

        // 🟢 2. 调用 white::process
        Ok(white::process(img, make, model, params, &font, "Bold", &assets))
    }
}

// ==========================================
// 策略 2: 模糊处理器 (Blur)
// ==========================================
pub struct TransparentClassicProcessor {
    pub font_data: Arc<Vec<u8>>,
    pub shadow: f32,
}

impl FrameProcessor for TransparentClassicProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        // 🟢 1. 解析品牌并获取资源
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
                // 其他品牌只显示主标
                _ => BlurStyleResources {
                    main_logo: resources::get_logo(b, LogoType::Wordmark),
                    sub_logo: None,
                }
            }
        } else {
            BlurStyleResources { main_logo: None, sub_logo: None }
        };
        
        // 🟢 2. 调用 blur::process
        Ok(blur::process(img, make, model, params, &font, "Bold", self.shadow, &assets))
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
        
        // 1. 解析主字体 (参数数值)
        let main = FontRef::try_from_slice(&self.main_font)
            .map_err(|_| "Master模式: 主字体解析失败".to_string())?;

        // 2. 解析手写体
        let script = FontRef::try_from_slice(&self.script_font)
            .unwrap_or_else(|_| {
                println!("⚠️ Master模式: 手写体解析失败，回退");
                main.clone()
            });

        // 3. 解析标题体
        let serif = FontRef::try_from_slice(&self.serif_font)
            .unwrap_or_else(|_| {
                println!("⚠️ Master模式: 标题字体解析失败，回退");
                main.clone()
            });

        // 4. 绘制 (Master 模式不需要 Brand Logo)
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
        StyleOptions::TransparentClassic { shadow_intensity } => {
            Box::new(TransparentClassicProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold),
                shadow: *shadow_intensity 
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
        
    }
}