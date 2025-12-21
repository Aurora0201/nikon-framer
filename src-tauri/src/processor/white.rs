use image::{DynamicImage, ImageBuffer, Rgba, imageops, GenericImageView};
use ab_glyph::{FontRef, PxScale};
use std::time::Instant; // 🟢 引入计时器

use crate::resources::BrandLogos;
use crate::graphics;
use super::{DrawContext, clean_model_name, resize_image_by_height};

// ... (LayoutConfig, LayoutMetrics, calculate_metrics 保持不变，请勿删除) ...
// 这里为了篇幅省略了配置部分，请务必保留你原来的代码！
struct LayoutConfig {
    bottom_ratio: f32,      
    scale_model_text: f32,  
    scale_params_text: f32, 
    scale_logo_word: f32,   
    scale_logo_z: f32,      
    gap_icon_text: f32,     
    margin_left: f32,       
    line_gap: f32,          
    skew_padding_fix: i32,
    model_text_lift: f32, 
}

impl LayoutConfig {
    fn nikon_default() -> Self {
        Self {
            bottom_ratio: 0.14,
            scale_model_text: 0.95, // 记住你的修改
            scale_params_text: 0.22,
            scale_logo_word: 1.15,  
            scale_logo_z: 0.9, 
            gap_icon_text: 0.25,
            margin_left: 0.4,
            line_gap: 0.1,
            skew_padding_fix: -10,
            model_text_lift: -0.20, // 记住你的修改
        }
    }
}

struct LayoutMetrics {
    bottom_height: u32,
    base_h: f32,          
    margin_left: i32,
    gap_icon_text: i32,
    line_gap: i32,
    bar_center_y: i32,
    line1_height: f32,    
    line1_y: i32,         
}

fn calculate_metrics(img_height: u32, config: &LayoutConfig) -> LayoutMetrics {
    let bottom_height = (img_height as f32 * config.bottom_ratio) as u32;
    let base_h = bottom_height as f32 * 0.25; 

    let line1_height = base_h * config.scale_logo_word;
    let font_size_params = bottom_height as f32 * config.scale_params_text;
    let line_gap = (bottom_height as f32 * config.line_gap) as i32;
    let total_block_h = line1_height + line_gap as f32 + font_size_params;
    
    let bar_center_y = img_height as f32 + bottom_height as f32 / 2.0;
    let text_block_start_y = bar_center_y - (total_block_h / 2.0);

    LayoutMetrics {
        bottom_height,
        base_h,
        margin_left: (bottom_height as f32 * config.margin_left) as i32,
        gap_icon_text: (bottom_height as f32 * config.gap_icon_text) as i32,
        line_gap,
        bar_center_y: bar_center_y as i32,
        line1_height,
        line1_y: text_block_start_y.round() as i32,
    }
}

// 🟢 绘图逻辑
fn draw_left_icon(ctx: &mut DrawContext, icon: &DynamicImage, metrics: &LayoutMetrics) -> i32 {
    let t_start = Instant::now();
    let max_h = (metrics.bottom_height as f32 * 0.65) as u32;
    let scaled_icon = resize_image_by_height(icon, max_h);
    let icon_y = metrics.bar_center_y - (scaled_icon.height() as i32 / 2);
    let icon_x = metrics.margin_left;
    imageops::overlay(ctx.canvas, &scaled_icon, icon_x as i64, icon_y as i64);
    // println!("  - [Detail] 左侧金标绘制: {:.2?}", t_start.elapsed());
    icon_x + scaled_icon.width() as i32 + metrics.gap_icon_text
}

fn draw_main_line_elements(
    ctx: &mut DrawContext,
    start_x: i32,
    logos: &BrandLogos,
    camera_make: &str,
    camera_model: &str,
    metrics: &LayoutMetrics,
    config: &LayoutConfig
) {
    let t_start = Instant::now();
    let mut current_x = start_x;
    let line1_y = metrics.line1_y;

    // 1. Nikon Logo
    if let Some(word_logo) = &logos.word {
        let target_h = (metrics.base_h as f32 * config.scale_logo_word) as u32;
        let scaled_word = resize_image_by_height(word_logo, target_h);
        let word_y = line1_y + ((metrics.line1_height as i32 - scaled_word.height() as i32) / 2);
        imageops::overlay(ctx.canvas, &scaled_word, current_x as i64, word_y as i64);
        current_x += scaled_word.width() as i32 + (metrics.base_h as f32 * 0.3) as i32;
    }

    // 2. Z Symbol
    let mut z_bottom_y = line1_y + metrics.line1_height as i32; 
    if let Some(z_img) = &logos.z_symbol {
        let target_h = (metrics.base_h as f32 * config.scale_logo_z) as u32;
        let scaled_z = resize_image_by_height(z_img, target_h);
        let z_y = line1_y + ((metrics.line1_height as i32 - scaled_z.height() as i32) / 2);
        imageops::overlay(ctx.canvas, &scaled_z, current_x as i64, z_y as i64);
        z_bottom_y = z_y + scaled_z.height() as i32;
        current_x += scaled_z.width() as i32 + (metrics.base_h as f32 * 0.15) as i32;
    }

    // 3. Model Number
    if !camera_model.is_empty() {
        let model_text = clean_model_name(camera_make, camera_model);
        let text_size = metrics.base_h as f32 * config.scale_model_text;
        
        let t_skew = Instant::now();
        let italic_img = graphics::generate_skewed_text_high_quality(
            &model_text, ctx.font, PxScale::from(text_size), Rgba([0, 0, 0, 255]), 0.23
        );
        // println!("  - [Detail] 斜体生成: {:.2?}", t_skew.elapsed());

        let align_bottom_y = z_bottom_y - italic_img.height() as i32;
        let lift = (metrics.base_h * config.model_text_lift) as i32;
        let draw_y = align_bottom_y - lift;
        let draw_x = current_x + config.skew_padding_fix;
        imageops::overlay(ctx.canvas, &italic_img, draw_x as i64, draw_y as i64);
    }
    // println!("  - [Detail] 第一行元素绘制总耗时: {:.2?}", t_start.elapsed());
}

fn draw_params_line(ctx: &mut DrawContext, start_x: i32, params: &str, metrics: &LayoutMetrics, config: &LayoutConfig) {
    if params.is_empty() { return; }
    let line2_y = metrics.line1_y + metrics.line1_height as i32 + metrics.line_gap;
    let sub_weight = if ctx.font_weight == "ExtraBold" { "Bold" } else { ctx.font_weight };
    let font_size = metrics.bottom_height as f32 * config.scale_params_text;
    graphics::draw_text_high_quality(
        ctx.canvas, Rgba([100, 100, 100, 255]), start_x, line2_y, 
        PxScale::from(font_size), ctx.font, params, sub_weight
    );
}

// 主处理函数
pub fn process(
    img: &DynamicImage,
    camera_make: &str,
    camera_model: &str,
    shooting_params: &str,
    font: &FontRef,
    font_weight: &str,
    logos: &BrandLogos 
) -> DynamicImage {
    let t0 = Instant::now();
    let (width, height) = img.dimensions();
    
    let config = LayoutConfig::nikon_default();
    let metrics = calculate_metrics(height, &config);
    let new_height = height + metrics.bottom_height;
    
    // 1. 创建画布
    let mut canvas = ImageBuffer::from_pixel(width, new_height, Rgba([255, 255, 255, 255]));
    imageops::overlay(&mut canvas, img, 0, 0);
    // println!("  - [Detail] 画布创建与原图叠加: {:.2?}", t0.elapsed());

    let mut ctx = DrawContext { canvas: &mut canvas, font, font_weight };

    // 2. 绘制各个部分
    let mut content_start_x = metrics.margin_left;
    if let Some(icon) = &logos.icon {
        content_start_x = draw_left_icon(&mut ctx, icon, &metrics);
    }

    draw_main_line_elements(&mut ctx, content_start_x, logos, camera_make, camera_model, &metrics, &config);
    draw_params_line(&mut ctx, content_start_x, shooting_params, &metrics, &config);

    println!("  - [PERF] 白底模式-绘制阶段总耗时: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}