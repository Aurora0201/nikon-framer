// src-tauri/src/processor/mod.rs

pub mod white;
pub mod blur;
pub mod traits;

// 🟢 修改点：引入 ImageFormat，去掉 ImageOutputFormat (为了兼容性)
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use ab_glyph::FontRef; 

use crate::models::{StyleOptions, FontConfig};
use crate::processor::traits::FrameProcessor;

// 引用 resources 模块
use crate::resources; 

pub struct DrawContext<'a> {
    pub canvas: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub font: &'a FontRef<'a>,
    pub font_weight: &'a str,
}

pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

// 🟢 修复点：添加了缺失的分号，并补全了完整逻辑
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
    }; // 🟢 之前报错就是这里少了这个分号！

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



// --- 策略 1: 白底处理器 ---
struct BottomWhiteProcessor {
    font_config: FontConfig,
}

impl FrameProcessor for BottomWhiteProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        // 1. 加载资源 (字体 & Logo)
        // 注意：这里每次处理都加载了一次资源。
        // 如果追求极致性能，可以将 font_data 缓存到 Struct 中，但涉及生命周期会变复杂，目前这样足够快。
        let font_data = resources::load_font_data(&self.font_config.filename);
        let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体文件解析失败")?;
        
        // 根据相机厂商加载对应的 Logo 集合
        let logos = resources::load_brand_logos(make);


        // 🟢 修复：假设 blur::process 直接返回 DynamicImage
        // 我们需要手动把它包裹在 Ok() 里以符合 Result 返回值要求
        let result_img = white::process(
            img, 
            make, 
            model, 
            params, 
            &font, 
            &self.font_config.weight,
            &logos
        ); 
        
        // 如果 blur::process 可能会 panic 而不是返回 Result，这里直接 Ok 包裹
        Ok(result_img)
    }
}

// --- 策略 2: 模糊处理器 ---
struct BlurProcessor {
    font_config: FontConfig,
    shadow: f32,
}

impl FrameProcessor for BlurProcessor {
    fn process(&self, img: &DynamicImage, make: &str, model: &str, params: &str) -> Result<DynamicImage, String> {
        let font_data = resources::load_font_data(&self.font_config.filename);
        let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体文件解析失败")?;
        let logos = resources::load_brand_logos(make);

        // 🟢 修复：假设 blur::process 直接返回 DynamicImage
        // 我们需要手动把它包裹在 Ok() 里以符合 Result 返回值要求
        let result_img = blur::process(
            img, 
            make, 
            model, 
            params, 
            &font, 
            &self.font_config.weight, 
            self.shadow, 
            &logos
        ); 
        
        // 如果 blur::process 可能会 panic 而不是返回 Result，这里直接 Ok 包裹
        Ok(result_img)
    }
}

// --- 🏭 工厂函数：根据枚举创建对应的处理器 ---
pub fn create_processor(options: &StyleOptions) -> Box<dyn FrameProcessor> {
    match options {
        StyleOptions::BottomWhite { font } => {
            Box::new(BottomWhiteProcessor { 
                font_config: font.clone() 
            })
        },
        StyleOptions::GaussianBlur { font, shadow_intensity } => {
            Box::new(BlurProcessor { 
                font_config: font.clone(),
                shadow: *shadow_intensity 
            })
        },
        
    }
}