use image::{DynamicImage, GenericImageView, Rgba, imageops};
use ab_glyph::{FontRef, PxScale};
// 🟢 1. 引入 draw_text_mut
use imageproc::drawing::{text_size, draw_text_mut};
use std::time::Instant;
use std::sync::Arc;
use std::cmp::min;
use crate::graphics::effects::generate_blurred_background;

use crate::graphics;
// 引入父模块通用工具
use super::resize_image_by_height;

// ==========================================
// 1. 数据结构定义
// ==========================================

pub struct BlurStyleResources {
    pub logo: Option<Arc<DynamicImage>>, 
}

pub struct BlurInput<'a> {
    pub brand: &'a str,
    pub model: &'a str,
    pub params: &'a str,
}

// ==========================================
// 2. 布局配置
// ==========================================
struct BlurConfig {
    border_ratio: f32,       
    bottom_extra_ratio: f32, 

    blur_sigma: f32,         
    bg_brightness: i32,      
    process_limit: u32,      

    font_scale_model: f32,   
    font_scale_params: f32,  
    
    logo_height_ratio: f32,  

    gap_logo_text_ratio: f32, 
    gap_lines_ratio: f32,     
    
    text_color_model: Rgba<u8>,
    text_color_params: Rgba<u8>,
}

impl Default for BlurConfig {
    fn default() -> Self {
        Self {
            border_ratio: 0.08,        
            bottom_extra_ratio: 0.85,  

            blur_sigma: 120.0,          
            bg_brightness: -150,       
            process_limit: 400,        

            font_scale_model: 0.56,    
            font_scale_params: 0.45,   

            logo_height_ratio: 0.85,   
            
            gap_logo_text_ratio: 0.6,  
            gap_lines_ratio: 0.60,     

            text_color_model: Rgba([255, 255, 255, 255]),
            text_color_params: Rgba([220, 220, 220, 255]),
        }
    }
}

// ==========================================
// 3. 核心处理逻辑
// ==========================================
pub fn process(
    img: &DynamicImage,
    font: &FontRef,
    input: BlurInput,
    assets: &BlurStyleResources 
) -> DynamicImage {
    let t0 = Instant::now();
    let cfg = BlurConfig::default();
    let (width, height) = img.dimensions();

    // -------------------------------------------------------------
    // A. 尺寸计算
    // -------------------------------------------------------------
    let ref_size = min(width, height) as f32;
    let border_size = (ref_size * cfg.border_ratio) as u32;
    let bottom_extra_h = (border_size as f32 * cfg.bottom_extra_ratio) as u32;

    let canvas_w = width + border_size * 2;
    let canvas_h = height + border_size * 2 + bottom_extra_h;

    // -------------------------------------------------------------
    // B. 背景生成 (重构)
    // -------------------------------------------------------------
    let t_blur = Instant::now();
    
    // 🟢 [修改] 调用公共高性能方法
    // 这里传入 cfg.bg_brightness (通常是 -150)
    let mut canvas = generate_blurred_background(
        img, 
        canvas_w, 
        canvas_h, 
        cfg.blur_sigma, 
        cfg.bg_brightness 
    ).to_rgba8(); // 注意：generate 返回 DynamicImage，这里转为 RgbaImage

    println!("  - [PERF] Blur Background: {:.2?}", t_blur.elapsed());

    // -------------------------------------------------------------
    // C. 前景合成
    // -------------------------------------------------------------
    let glass_img = graphics::apply_rounded_glass_effect(img);
    let overlay_x = (canvas_w - glass_img.width()) / 2;
    let border_thickness_diff = (glass_img.height().saturating_sub(height)) / 2;
    let overlay_y = (border_size as i64) - (border_thickness_diff as i64);

    imageops::overlay(&mut canvas, &glass_img, overlay_x as i64, overlay_y);

    // -------------------------------------------------------------
    // D. 字体与排版计算
    // -------------------------------------------------------------
    let font_size_model = border_size as f32 * cfg.font_scale_model;
    let font_size_params = border_size as f32 * cfg.font_scale_params;
    let scale_model = PxScale::from(font_size_model);
    let scale_params = PxScale::from(font_size_params);

    // 🟢 直接使用 input.model (Parser 已经清洗过)
    let model_str = input.model; 

    // --- 1. 测量第一行 [Logo] [Gap] [Model] ---
    let mut line1_width = 0;
    let mut line1_height = 0;
    let mut logo_draw_w = 0;
    let mut logo_draw_h = 0;
    let mut scaled_logo = None;

    if let Some(logo) = &assets.logo {
        let target_h = (font_size_model * cfg.logo_height_ratio) as u32;
        let white_logo = graphics::make_image_white(logo);
        let resized = resize_image_by_height(&white_logo, target_h);
        
        logo_draw_w = resized.width() as u32;
        logo_draw_h = resized.height() as u32;
        scaled_logo = Some(resized);
        
        line1_width += logo_draw_w;
    }

    let (model_text_w, model_text_h) = if !model_str.is_empty() {
        let (w, h) = text_size(scale_model, font, model_str);
        (w as u32, h as u32)
    } else {
        (0, 0)
    };

    if model_text_w > 0 {
        if logo_draw_w > 0 {
            line1_width += (font_size_model * cfg.gap_logo_text_ratio) as u32;
        }
        line1_width += model_text_w;
        line1_height = model_text_h; 
    }
    if line1_height == 0 { line1_height = logo_draw_h; }

    // --- 2. 测量第二行 [Params] ---
    let (params_w, params_h) = if !input.params.is_empty() {
        let (w, h) = text_size(scale_params, font, input.params);
        (w as u32, h as u32)
    } else {
        (0, 0)
    };

    // --- 3. 垂直布局 ---
    let gap_lines = (font_size_model * cfg.gap_lines_ratio) as u32;
    let total_block_h = line1_height + gap_lines + params_h;

    let bottom_area_y = border_size + height; 
    let bottom_area_h = border_size + bottom_extra_h; 
    let block_start_y = bottom_area_y as u32 + (bottom_area_h - total_block_h) / 2;

    // -------------------------------------------------------------
    // E. 绘制
    // -------------------------------------------------------------
    
    // --- 第一行 ---
    if line1_width > 0 {
        let mut cursor_x = (canvas_w - line1_width) / 2;
        let line1_base_y = block_start_y; 

        // Logo
        if let Some(logo) = scaled_logo {
            let offset_y = if line1_height > logo_draw_h {
                (line1_height - logo_draw_h) / 2
            } else { 0 };
            
            imageops::overlay(&mut canvas, &logo, cursor_x as i64, (line1_base_y + offset_y) as i64);
            cursor_x += logo_draw_w + (font_size_model * cfg.gap_logo_text_ratio) as u32;
        }

        // 机型文字
        if model_text_w > 0 {
            // 🟢 2. 直接使用 draw_text_mut
            draw_text_mut(
                &mut canvas, 
                cfg.text_color_model, 
                cursor_x as i32, 
                line1_base_y as i32, 
                scale_model, 
                font, 
                model_str
            );
        }
    }

    // --- 第二行 ---
    if params_w > 0 {
        let line2_x = (canvas_w - params_w) / 2;
        let line2_y = block_start_y + line1_height + gap_lines;
        
        // 🟢 2. 直接使用 draw_text_mut
        draw_text_mut(
            &mut canvas, 
            cfg.text_color_params, 
            line2_x as i32, 
            line2_y as i32, 
            scale_params, 
            font, 
            input.params
        );
    }

    println!("  - [PERF] Blur Total Time: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}