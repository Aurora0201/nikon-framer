use image::{DynamicImage, GenericImageView, Rgba, imageops};
use ab_glyph::{FontRef, PxScale};
use std::time::Instant; // 🟢 引入计时器

use crate::resources::BrandLogos;
use crate::graphics;
use super::{clean_model_name, format_model_text};

pub fn process(
    img: &DynamicImage,
    camera_make: &str,
    camera_model: &str,
    shooting_params: &str,
    font: &FontRef,
    font_weight: &str,
    shadow_intensity: f32,
    logos: &BrandLogos 
) -> DynamicImage {
    let t0 = Instant::now();
    let (width, height) = img.dimensions();
    let border_size = (width as f32 * 0.08) as u32; 
    let bottom_extra = (border_size as f32 * 0.6) as u32; 
    let canvas_w = width + border_size * 2;
    let canvas_h = height + border_size * 2 + bottom_extra;

    // 1. 模糊背景 (性能热点)
    let t_blur = Instant::now();
    let process_limit = 400u32; 
    let scale_factor_bg = (width.max(height) as f32 / process_limit as f32).max(1.0);
    let small_w = (canvas_w as f32 / scale_factor_bg) as u32;
    let small_h = (canvas_h as f32 / scale_factor_bg) as u32;
    let small_img = img.resize_exact(small_w, small_h, imageops::FilterType::Nearest);
    let mut blurred = small_img.blur(30.0);
    imageops::colorops::brighten(&mut blurred, -180);
    let mut canvas = blurred.resize_exact(canvas_w, canvas_h, imageops::FilterType::Triangle).to_rgba8();
    println!("  - [PERF] 高斯模糊背景生成: {:.2?}", t_blur.elapsed());

    // 2. 玻璃与阴影
    let t_shadow = Instant::now();
    let glass_img = graphics::apply_rounded_glass_effect(img);
    let shadow_img = graphics::create_diffuse_shadow(glass_img.width(), glass_img.height(), border_size, shadow_intensity);
    
    let target_center_x = (border_size as i64) + (width as i64 / 2);
    let offset_y = (border_size as f32 * 0.3) as i64;
    let target_center_y = (border_size as i64) + (height as i64 / 2) + offset_y;
    
    let draw_x = target_center_x - (shadow_img.width() as i64 / 2);
    let draw_y = target_center_y - (shadow_img.height() as i64 / 2);
    imageops::overlay(&mut canvas, &shadow_img, draw_x as i64, draw_y as i64);

    let border_thickness = (glass_img.width() - width) / 2;
    let overlay_x = border_size as i64 - border_thickness as i64;
    let overlay_y = border_size as i64 - border_thickness as i64;
    imageops::overlay(&mut canvas, &glass_img, overlay_x, overlay_y);
    println!("  - [PERF] 阴影与玻璃特效合成: {:.2?}", t_shadow.elapsed());

    // 3. 文字布局参数
    let text_color = Rgba([255, 255, 255, 255]); 
    let sub_text_color = Rgba([200, 200, 200, 255]); 
    let font_size_model = border_size as f32 * 0.55; 
    let font_size_params = border_size as f32 * 0.32; 
    let scale_model = PxScale::from(font_size_model);
    let scale_params = PxScale::from(font_size_params);
    let text_area_start_y = (border_size + height) as f32;
    let text_area_total_h = (border_size + bottom_extra) as f32;
    let line_gap = font_size_model * 0.12; 
    let text_block_h = font_size_model + line_gap + font_size_params;
    let padding_top = (text_area_total_h - text_block_h) / 2.0;
    let line1_y = (text_area_start_y + padding_top).round() as i32;
    let line2_y = (text_area_start_y + padding_top + font_size_model + line_gap).round() as i32;

    // 4. 居中绘制机型
    if !camera_model.is_empty() {
        if let Some(logo) = &logos.icon {
            let model_text = clean_model_name(camera_make, camera_model);
            let target_logo_h = (font_size_model * 0.85) as u32;
            let ratio = logo.width() as f32 / logo.height() as f32;
            let target_logo_w = (target_logo_h as f32 * ratio) as u32;
            let scaled_logo = logo.resize_exact(target_logo_w, target_logo_h, imageops::FilterType::Lanczos3);

            let text_w = if model_text.is_empty() { 0 } else { graphics::measure_text_width(font, &model_text, scale_model) };
            let spacing = if model_text.is_empty() { 0 } else { 15 };
            let total_w = target_logo_w + spacing + text_w;

            let mut current_x = (canvas_w as i32 - total_w as i32) / 2;
            let text_visual_center = line1_y + (font_size_model as i32 / 2);
            let logo_y = text_visual_center - (scaled_logo.height() as i32 / 2);
            
            imageops::overlay(&mut canvas, &scaled_logo, current_x as i64, logo_y as i64);
            current_x += target_logo_w as i32 + spacing as i32;

            if !model_text.is_empty() {
                graphics::draw_text_high_quality(&mut canvas, text_color, current_x, line1_y, scale_model, font, &model_text, font_weight);
            }
        } else {
            let full_text = format_model_text(camera_model);
            let text_w = graphics::measure_text_width(font, &full_text, scale_model);
            let text_x = ((canvas_w as i32 - text_w as i32) / 2).max(0);
            graphics::draw_text_high_quality(&mut canvas, text_color, text_x, line1_y, scale_model, font, &full_text, font_weight);
        }
    }

    if !shooting_params.is_empty() {
        let text_w = graphics::measure_text_width(font, shooting_params, scale_params);
        let text_x = ((canvas_w as i32 - text_w as i32) / 2).max(0);
        let sub_weight = if font_weight == "ExtraBold" { "Bold" } else { font_weight };
        graphics::draw_text_high_quality(&mut canvas, sub_text_color, text_x, line2_y, scale_params, font, shooting_params, sub_weight);
    }

    println!("  - [PERF] 高斯模糊模式-绘制阶段总耗时: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}