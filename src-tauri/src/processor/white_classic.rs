use image::{DynamicImage, ImageBuffer, Rgba, imageops, GenericImageView};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{draw_text_mut, text_size, draw_filled_rect_mut};
use imageproc::rect::Rect;
use std::sync::Arc;
use std::cmp::min;
use std::time::Instant;

// 引入父模块通用工具
use super::resize_image_by_height; 

// ==========================================
// 1. 资源定义
// ==========================================
pub struct WhiteStyleResources {
    pub logo: Option<Arc<DynamicImage>>, 
}

// ==========================================
// 2. 布局配置
// ==========================================
struct LayoutConfig {
    bar_ratio_landscape: f32, 
    bar_ratio_portrait: f32,  
    
    land_side_margin_ratio: f32,
    port_side_margin_ratio: f32, 
    
    // --- 横构图 ---
    land_font_scale_model: f32,  
    land_font_scale_params: f32, 
    land_icon_scale: f32,

    // --- 竖构图 ---
    port_font_scale_model: f32,   
    port_font_scale_params: f32,  
    port_icon_scale: f32,         
    
    // 间距配置
    element_gap_ratio: f32,       
    line_width_ratio: f32,        
    portrait_line_gap_ratio: f32, 
    portrait_line_height_scale: f32, 

    // 偏移微调
    offset_y_left_ratio: f32,
    offset_y_right_ratio: f32,
    
    // 🟢 竖构图文字块垂直偏移 (修正视觉重心)
    portrait_text_offset_y_ratio: f32,
}

impl LayoutConfig {
    fn default() -> Self {
        Self {
            bar_ratio_landscape: 0.12, 
            bar_ratio_portrait: 0.13,  
            
            land_side_margin_ratio: 0.5,
            port_side_margin_ratio: 0.35,
            
            // --- 横构图 ---
            land_font_scale_model: 0.38,
            land_font_scale_params: 0.31,
            land_icon_scale: 0.52,

            // --- 竖构图 ---
            port_icon_scale: 0.38,         
            port_font_scale_model: 0.30,   
            port_font_scale_params: 0.25,  

            // 通用
            element_gap_ratio: 0.30,  
            line_width_ratio: 0.025, // 竖线较粗
            
            offset_y_left_ratio: -0.05, 
            offset_y_right_ratio: -0.05,
            
            portrait_line_gap_ratio: 0.15, 
            portrait_line_height_scale: 1.3, // 竖线较长

            // 🟢 仅针对文字块向上微调 (负值向上)
            // 建议稍微加大一点点，因为只有文字在动
            portrait_text_offset_y_ratio: -0.02,
        }
    }
}

// ==========================================
// 3. 计算结果 (Metrics)
// ==========================================
struct Metrics {
    bar_height: u32,
    land_padding_x: i32, 
    port_padding_x: i32,

    center_y: i32, // 几何中心
    
    gap: i32,
    line_w: u32,
    line_h_land: u32, 
    
    offset_y_left: i32,
    offset_y_right: i32,
    
    portrait_line_gap: i32,
    portrait_text_offset_y: i32, // 🟢 文字块的像素偏移量
}

fn calculate_metrics(w: u32, h: u32, cfg: &LayoutConfig) -> Metrics {
    let short_edge = min(w, h) as f32;
    
    let is_landscape = w >= h;
    let target_ratio = if is_landscape {
        cfg.bar_ratio_landscape
    } else {
        cfg.bar_ratio_portrait
    };
    
    let bar_height = (short_edge * target_ratio) as u32;
    let center_y = (h as f32 + bar_height as f32 / 2.0) as i32;

    Metrics {
        bar_height,
        land_padding_x: (bar_height as f32 * cfg.land_side_margin_ratio) as i32,
        port_padding_x: (bar_height as f32 * cfg.port_side_margin_ratio) as i32,

        center_y,
        
        gap: (bar_height as f32 * cfg.element_gap_ratio) as i32,
        line_w: (bar_height as f32 * cfg.line_width_ratio).max(1.0) as u32,
        
        line_h_land: (bar_height as f32 * 0.55) as u32, 

        offset_y_left: (bar_height as f32 * cfg.offset_y_left_ratio) as i32,
        offset_y_right: (bar_height as f32 * cfg.offset_y_right_ratio) as i32,
        
        portrait_line_gap: (bar_height as f32 * cfg.portrait_line_gap_ratio) as i32,
        
        // 🟢 计算竖构图文字块偏移
        portrait_text_offset_y: (bar_height as f32 * cfg.portrait_text_offset_y_ratio) as i32,
    }
}

// ==========================================
// 辅助：横构图测量
// ==========================================
fn measure_right_width_land(
    font: &FontRef,
    params: &str,
    assets: &WhiteStyleResources,
    metrics: &Metrics,
    cfg: &LayoutConfig
) -> u32 {
    let mut total_width = 0;
    let icon_h = (metrics.bar_height as f32 * cfg.land_icon_scale) as u32;
    
    if let Some(logo) = &assets.logo {
        let (lw, lh) = logo.dimensions();
        let aspect_ratio = lw as f32 / lh as f32;
        let target_h = if aspect_ratio > 1.5 { (icon_h as f32 * 0.65) as u32 } else { icon_h };
        let target_w = (target_h as f32 * aspect_ratio) as u32;
        total_width += target_w;
    }
    if !params.is_empty() {
        total_width += metrics.gap as u32; 
        total_width += metrics.line_w;     
        total_width += metrics.gap as u32; 
    }
    if !params.is_empty() {
        let font_size = metrics.bar_height as f32 * cfg.land_font_scale_params; 
        let scale = PxScale::from(font_size);
        let (w, _h) = text_size(scale, font, params);
        total_width += w as u32;
    }
    total_width + metrics.land_padding_x as u32
}

// ==========================================
// 4. 绘图实现 (横构图)
// ==========================================
fn draw_left_section_landscape(
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &FontRef,
    text: &str,
    metrics: &Metrics,
    cfg: &LayoutConfig,
    max_width: u32,
) {
    if text.is_empty() { return; }
    let default_font_size = metrics.bar_height as f32 * cfg.land_font_scale_model;
    let mut scale = PxScale::from(default_font_size);
    let text_h;
    
    loop {
        let size = text_size(scale, font, text);
        if size.0 as u32 <= max_width || scale.x < 10.0 {
            text_h = size.1;
            break;
        }
        scale = PxScale::from(scale.x * 0.95);
    }
    
    let text_y = metrics.center_y - (text_h as i32 / 2) + metrics.offset_y_left;
    draw_text_mut(canvas, Rgba([0, 0, 0, 255]), metrics.land_padding_x, text_y, scale, font, text);
}

fn draw_right_section_landscape(
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &FontRef,
    params: &str,
    assets: &WhiteStyleResources,
    width: u32,
    metrics: &Metrics,
    cfg: &LayoutConfig
) {
    let right_edge = width as i32 - metrics.land_padding_x;
    let mut cursor_x = right_edge;

    let font_size = metrics.bar_height as f32 * cfg.land_font_scale_params;
    let icon_h = (metrics.bar_height as f32 * cfg.land_icon_scale) as u32;

    if !params.is_empty() {
        let scale = PxScale::from(font_size);
        let (w, h) = text_size(scale, font, params);
        let y = (metrics.center_y + metrics.offset_y_right) - (h as i32 / 2);
        let x = cursor_x - w as i32;
        draw_text_mut(canvas, Rgba([60, 60, 60, 255]), x, y, scale, font, params);
        cursor_x = x - metrics.gap;
    }
    if !params.is_empty() {
        let x = cursor_x - metrics.line_w as i32;
        let y = metrics.center_y - (metrics.line_h_land as i32 / 2);
        let rect = Rect::at(x, y).of_size(metrics.line_w, metrics.line_h_land);
        draw_filled_rect_mut(canvas, rect, Rgba([160, 160, 160, 255]));
        cursor_x = x - metrics.gap;
    }
    if let Some(logo) = &assets.logo {
        let (lw, lh) = logo.dimensions();
        let aspect_ratio = lw as f32 / lh as f32;
        let target_h = if aspect_ratio > 1.5 { (icon_h as f32 * 0.65) as u32 } else { icon_h };
        let scaled_logo = resize_image_by_height(logo, target_h);
        let y = metrics.center_y - (scaled_logo.height() as i32 / 2);
        let x = cursor_x - scaled_logo.width() as i32;
        imageops::overlay(canvas, &scaled_logo, x as i64, y as i64);
    }
}

// ==========================================
// 🟢 竖构图专用布局 (Logo | Line | StackedText)
// ==========================================
fn draw_portrait_layout(
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &FontRef,
    text_model: &str,
    text_params: &str,
    assets: &WhiteStyleResources,
    metrics: &Metrics,
    cfg: &LayoutConfig
) {
    // 1. 准备字体 & 尺寸
    let scale_model = PxScale::from(metrics.bar_height as f32 * cfg.port_font_scale_model);
    let scale_params = PxScale::from(metrics.bar_height as f32 * cfg.port_font_scale_params);
    let icon_h_port = (metrics.bar_height as f32 * cfg.port_icon_scale) as u32;

    // 2. 测量文字块
    let (_, h_model) = text_size(scale_model, font, text_model);
    let (_, h_params) = text_size(scale_params, font, text_params);
    let text_block_h = h_model as i32 + metrics.portrait_line_gap + h_params as i32;

    // 3. 测量 Logo (用于决定竖线高度)
    let mut logo_draw_h = 0;
    let mut scaled_logo_opt = None;
    if let Some(logo) = &assets.logo {
        let (lw, lh) = logo.dimensions();
        let aspect_ratio = lw as f32 / lh as f32;
        let mut target_h = icon_h_port;
        if aspect_ratio > 1.5 {
            target_h = (icon_h_port as f32 * 0.65) as u32;
        }
        let scaled = resize_image_by_height(logo, target_h);
        logo_draw_h = scaled.height() as i32;
        scaled_logo_opt = Some(scaled);
    }

    // 4. 决定竖线高度 (明显长于内容)
    let content_max_h = std::cmp::max(logo_draw_h, text_block_h);
    let line_height = (content_max_h as f32 * cfg.portrait_line_height_scale) as u32;

    // 5. 坐标计算
    let start_x = metrics.port_padding_x;
    let mut cursor_x = start_x;

    // 🟢 区分中心点
    // A. 几何中心 (用于 Logo 和 Line) -> 严格垂直居中
    let geom_center_y = metrics.center_y;
    
    // B. 视觉中心 (用于文字块) -> 包含微调偏移
    let text_visual_center_y = metrics.center_y + metrics.portrait_text_offset_y;

    // --- A. 绘制 Logo (几何居中) ---
    if let Some(scaled) = scaled_logo_opt {
        let y = geom_center_y - (scaled.height() as i32 / 2);
        imageops::overlay(canvas, &scaled, cursor_x as i64, y as i64);
        cursor_x += scaled.width() as i32 + metrics.gap;
    }

    // --- B. 绘制竖线 (几何居中) ---
    if assets.logo.is_some() {
        let y = geom_center_y - (line_height as i32 / 2);
        let rect = Rect::at(cursor_x, y).of_size(metrics.line_w, line_height);
        draw_filled_rect_mut(canvas, rect, Rgba([160, 160, 160, 255]));
        cursor_x += metrics.line_w as i32 + metrics.gap;
    }

    // --- C. 绘制文字块 (视觉微调居中) ---
    // 计算文字块相对于 visual_center_y 的起始点
    let text_block_start_y = text_visual_center_y - (text_block_h / 2);
    
    // Line 1: Model
    draw_text_mut(canvas, Rgba([0, 0, 0, 255]), cursor_x, text_block_start_y, scale_model, font, text_model);

    // Line 2: Params
    let params_y = text_block_start_y + h_model as i32 + metrics.portrait_line_gap;
    draw_text_mut(canvas, Rgba([60, 60, 60, 255]), cursor_x, params_y, scale_params, font, text_params);
}

// ==========================================
// 5. 主入口
// ==========================================
pub fn process(
    img: &DynamicImage,
    camera_make: &str,
    camera_model: &str,
    params: &str,
    font: &FontRef,
    assets: &WhiteStyleResources
) -> DynamicImage {
    let t0 = Instant::now();
    let (w, h) = (img.width(), img.height());

    let cfg = LayoutConfig::default();
    let metrics = calculate_metrics(w, h, &cfg);
    
    let new_h = h + metrics.bar_height;
    let mut canvas = ImageBuffer::from_pixel(w, new_h, Rgba([255, 255, 255, 255]));
    
    imageops::overlay(&mut canvas, img, 0, 0);

    let text_model = format!("{} {}", camera_make, camera_model).to_uppercase();

    if w >= h {
        // === 横构图 ===
        let right_width = measure_right_width_land(font, params, assets, &metrics, &cfg);
        let safe_gap = 50; 
        let max_left_width = if w > (right_width + metrics.land_padding_x as u32 + safe_gap) {
            w - right_width - metrics.land_padding_x as u32 - safe_gap
        } else {
            100 
        };
        draw_left_section_landscape(&mut canvas, font, &text_model, &metrics, &cfg, max_left_width);
        draw_right_section_landscape(&mut canvas, font, params, assets, w, &metrics, &cfg);
        println!("  - [PERF] White Layout: Landscape");
    } else {
        // === 竖构图 ===
        draw_portrait_layout(&mut canvas, font, &text_model, params, assets, &metrics, &cfg);
        println!("  - [PERF] White Layout: Portrait (Logo & Line Centered | Text Offset)");
    }

    println!("  - [PERF] Total Time: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}