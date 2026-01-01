use image::{DynamicImage, Rgba, RgbaImage, imageops, GenericImageView}; 
use ab_glyph::{FontRef, PxScale};
// 🟢 1. 引入标准绘图函数 draw_text_mut
use imageproc::drawing::{text_size, draw_text_mut}; 
use std::sync::Arc;
use std::time::Instant;
use std::cmp::min;

// 复用父模块的通用工具 (resize_image_by_height)
use super::resize_image_by_height;

// ==========================================
// 1. 数据结构
// ==========================================

pub struct PolaroidResources {
    pub logo: Option<Arc<DynamicImage>>,
}

pub struct PolaroidInput<'a> {
    pub brand: &'a str,
    pub model: &'a str,
    pub params: &'a str,
}

// ==========================================
// 2. 布局配置
// ==========================================
pub struct PolaroidConfig {
    pub side_border_ratio: f32,      
    pub bottom_height_multiplier: f32, 
    
    pub font_scale: f32,             
    pub logo_height_ratio: f32,      
    
    pub line_gap_ratio: f32,         
    pub content_vertical_bias: f32,  
    
    pub text_color: Rgba<u8>,        
    pub bg_color: Rgba<u8>,          
}

impl Default for PolaroidConfig {
    fn default() -> Self {
        Self {
            side_border_ratio: 0.05,
            bottom_height_multiplier: 4.5, 

            font_scale: 0.8,               
            logo_height_ratio: 1.0,        

            line_gap_ratio: 0.6,           
            content_vertical_bias: 0.0,   

            text_color: Rgba([20, 20, 20, 255]), 
            bg_color: Rgba([255, 255, 255, 255]),
        }
    }
}

// ==========================================
// 3. 核心处理函数
// ==========================================
pub fn process(
    img: &DynamicImage,
    font: &FontRef,
    input: PolaroidInput, 
    assets: &PolaroidResources,
) -> DynamicImage {
    let cfg = PolaroidConfig::default();
    let t0 = Instant::now();

    let (width, height) = img.dimensions();

    // -------------------------------------------------------------
    // A. 计算几何尺寸
    // -------------------------------------------------------------
    let base_size = min(width, height) as f32;
    let border_size = (base_size * cfg.side_border_ratio).round() as u32;
    let bottom_area_h = (border_size as f32 * cfg.bottom_height_multiplier).round() as u32;

    let canvas_w = width + border_size * 2;
    let canvas_h = height + border_size + bottom_area_h;

    // -------------------------------------------------------------
    // B. 创建画布并合成
    // -------------------------------------------------------------
    let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, cfg.bg_color);
    imageops::overlay(&mut canvas, img, border_size as i64, border_size as i64);

    // -------------------------------------------------------------
    // C. 底部内容排版
    // -------------------------------------------------------------
    let font_size = border_size as f32 * cfg.font_scale;
    let scale = PxScale::from(font_size);
    
    // C1. 准备 Logo
    let mut logo_draw_w = 0;
    let mut logo_draw_h = 0;
    let mut scaled_logo = None;

    if let Some(src_logo) = &assets.logo {
        let target_h = (border_size as f32 * cfg.logo_height_ratio) as u32;
        let resized = resize_image_by_height(src_logo, target_h);
        
        logo_draw_w = resized.width();
        logo_draw_h = resized.height();
        scaled_logo = Some(resized);
    }

    // C2. 准备文字
    let has_text = !input.params.is_empty();
    let (text_w, text_h) = if has_text {
        let (w, h) = text_size(scale, font, input.params);
        (w as u32, h as u32)
    } else {
        (0, 0)
    };

    // C3. 计算总高度
    let gap = font_size * cfg.line_gap_ratio;
    let mut total_content_h = 0.0;

    if logo_draw_h > 0 {
        total_content_h += logo_draw_h as f32;
    }
    if logo_draw_h > 0 && has_text {
        total_content_h += gap;
    }
    if has_text {
        total_content_h += text_h as f32;
    }

    // C4. 计算绘制起始点
    let footer_start_y = border_size + height;
    let footer_center_y = footer_start_y as f32 + (bottom_area_h as f32 / 2.0);
    
    let start_y = footer_center_y - (total_content_h / 2.0) + (bottom_area_h as f32 * cfg.content_vertical_bias);
    let mut cursor_y = start_y as i32;
    let center_x = canvas_w as i32 / 2;

    // -------------------------------------------------------------
    // D. 绘制
    // -------------------------------------------------------------

    // 1. 绘制 Logo
    if let Some(logo) = scaled_logo {
        let logo_x = center_x - (logo_draw_w as i32 / 2);
        imageops::overlay(&mut canvas, &logo, logo_x as i64, cursor_y as i64);
        cursor_y += logo_draw_h as i32 + gap as i32;
    }

    // 2. 绘制文字
    if has_text {
        let text_x = center_x - (text_w as i32 / 2);
        
        // 🟢 2. 直接调用标准库函数
        // 这里不再需要传 "Normal" 或 "Bold"，粗细完全由 `font` 变量决定的
        draw_text_mut(
            &mut canvas,
            cfg.text_color,
            text_x,
            cursor_y,
            scale,
            font,
            input.params
        );
    }

    println!("  - [PERF] Polaroid Process: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}