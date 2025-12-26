// src-tauri/src/processor/mod.rs

pub mod white;
pub mod blur;
pub mod traits;
pub mod master;

use std::sync::Arc; // 🟢 引入 Arc 用于共享只读资源
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use ab_glyph::FontRef; 

use crate::models::StyleOptions;
use crate::processor::traits::FrameProcessor;

// 🟢 引入重构后的 resources 模块 (包含 FontFamily, FontWeight)
use crate::resources::{self, FontFamily, FontWeight};

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
    // 🟢 使用 Arc<Vec<u8>>，直接指向全局缓存，零拷贝
    pub font_data: Arc<Vec<u8>>,
}

impl FrameProcessor for BottomWhiteProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        // 直接从 Arc 内存中解析 FontRef
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "白底模式: 标准字体解析失败")?;
        
        let logos = resources::load_brand_logos(make);
        
        // 白底模式强制使用 Bold
        Ok(white::process(img, make, model, params, &font, "Bold", &logos))
    }
}

// ==========================================
// 策略 2: 模糊处理器 (Blur)
// ==========================================
pub struct BlurProcessor {
    // 🟢 使用 Arc
    pub font_data: Arc<Vec<u8>>,
    pub shadow: f32,
}

impl FrameProcessor for BlurProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        let font = FontRef::try_from_slice(&self.font_data)
            .map_err(|_| "模糊模式: 标准字体解析失败")?;
            
        let logos = resources::load_brand_logos(make);
        
        Ok(blur::process(img, make, model, params, &font, "Bold", self.shadow, &logos))
    }
}

// ==========================================
// 策略 3: 大师处理器 (Master)
// ==========================================
pub struct MasterProcessor {
    // 🟢 持有三个不同字体的 Arc 指针
    pub main_font: Arc<Vec<u8>>,   // 参数字体
    pub script_font: Arc<Vec<u8>>, // 手写体
    pub serif_font: Arc<Vec<u8>>,  // 标题体
}

impl FrameProcessor for MasterProcessor {
    fn process(&self, img: &DynamicImage, _make: &str, _model: &str, params: &str) -> Result<DynamicImage, String> {
        
        // 1. 解析主字体 (参数数值)
        let main = FontRef::try_from_slice(&self.main_font)
            .map_err(|_| "Master模式: 主字体解析失败".to_string())?;

        // 2. 解析手写体 (回退机制：如果失败使用主字体)
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

        // 4. 绘制
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
        
        // 🟢 极简白底模式
        // 设计决策: 使用 InterDisplay Bold，现代且清晰
        StyleOptions::BottomWhite => {
            Box::new(BottomWhiteProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold) 
            })
        },

        // 🟢 高斯模糊模式
        // 设计决策: 同上，保持一致性
        StyleOptions::GaussianBlur { shadow_intensity } => {
            Box::new(BlurProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold),
                shadow: *shadow_intensity 
            })
        },

        // 🟢 大师模式 (精心搭配的字体组合)
        StyleOptions::Master => {
            Box::new(MasterProcessor {
                // 1. 参数数值: InterDisplay Medium (比 Bold 稍微精致一点，更有高级感)
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                
                // 2. 手写体: MrDafoe (艺术签名感)
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                
                // 3. 标题小字: AbhayaLibre (衬线体，显得正式、经典)
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },
        
    }
}