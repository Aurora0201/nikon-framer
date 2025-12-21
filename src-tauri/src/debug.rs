use std::io::Cursor;
use std::time::Instant;
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use base64::{Engine as _, engine::general_purpose};
use imageproc::drawing::draw_text_mut; // 🟢 去掉了 draw_filled_rect_mut
use imageproc::rect::Rect;
use ab_glyph::{FontRef, PxScale};

use crate::graphics;

pub fn generate_shadow_grid() -> Result<String, String> {
    let t_start = Instant::now();
    let canvas_w = 1500u32;
    let canvas_h = 1500u32;
    let mut canvas = ImageBuffer::from_pixel(canvas_w, canvas_h, Rgba([255, 255, 255, 255]));

    let font_data = include_bytes!("../assets/fonts/default.ttf").to_vec();
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体加载失败")?;

    let rows = 3u32; // 显式声明为 u32
    let cols = 3u32;
    let cell_w = canvas_w / cols;
    let cell_h = canvas_h / rows;
    
    let photo_sim_w = 200u32;
    let photo_sim_h = 200u32;
    let border_size = 0u32;

    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let intensity = 0.2 + (i as f32 * 0.2);

            let shadow_img = graphics::create_diffuse_shadow(photo_sim_w, photo_sim_h, border_size, intensity);
            let center_x = (c * cell_w) as i32 + (cell_w as i32 / 2);
            let center_y = (r * cell_h) as i32 + (cell_h as i32 / 2);

            let shadow_x = center_x - (shadow_img.width() as i32 / 2);
            let shadow_y = center_y - (shadow_img.height() as i32 / 2);
            
            imageops::overlay(&mut canvas, &shadow_img, shadow_x as i64, shadow_y as i64);

            let photo_rect = Rect::at(
                center_x - (photo_sim_w as i32 / 2), 
                center_y - (photo_sim_h as i32 / 2)
            ).of_size(photo_sim_w, photo_sim_h);
            
            graphics::draw_rounded_rect_mut(&mut canvas, photo_rect, 10, Rgba([240, 240, 240, 255]));

            let label = format!("Int: {:.1}", intensity);
            let text_scale = PxScale::from(24.0);
            let text_w = graphics::measure_text_width(&font, &label, text_scale);
            let text_x = center_x - (text_w as i32 / 2);
            let text_y = center_y + (photo_sim_h as i32 / 2) + 20;
            draw_text_mut(&mut canvas, Rgba([0, 0, 0, 255]), text_x, text_y, text_scale, &font, &label);
        }
    }

    println!("🚀 [DEBUG] 阴影网格生成耗时: {:.2?}", t_start.elapsed());
    
    let rgb_canvas = DynamicImage::ImageRgba8(canvas).to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    rgb_canvas.write_to(&mut buffer, image::ImageFormat::Jpeg).map_err(|e| format!("生成失败: {}", e))?;
    let base64_str = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/jpeg;base64,{}", base64_str))
}

pub fn generate_weight_grid() -> Result<String, String> {
    let t_start = Instant::now();
    let canvas_w = 1500u32;
    let canvas_h = 1500u32;
    let mut canvas = ImageBuffer::from_pixel(canvas_w, canvas_h, Rgba([255, 255, 255, 255]));

    let font_data = include_bytes!("../assets/fonts/default.ttf").to_vec();
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体加载失败")?;

    let rows = 2u32;
    let cols = 2u32;
    let cell_w = canvas_w / cols;
    let cell_h = canvas_h / rows;

    let modes = vec!["Normal", "Medium", "Bold", "ExtraBold"];

    for (i, mode) in modes.iter().enumerate() {
        // 🔴 修复点1：先强转 i 为 u32，再做除法
        let idx = i as u32;
        let r = idx / cols;
        let c = idx % cols;
        
        // 🔴 修复点2：先乘完 u32，再转 i32 做加法
        let center_x = (c * cell_w) as i32 + (cell_w as i32 / 2);
        let center_y = (r * cell_h) as i32 + (cell_h as i32 / 2);

        let test_text = "Nikon Z8";
        let text_scale = PxScale::from(100.0);
        let text_color = Rgba([0, 0, 0, 255]);
        
        let label = format!("Mode: {}", mode);
        let label_w = graphics::measure_text_width(&font, &label, PxScale::from(30.0));
        draw_text_mut(&mut canvas, Rgba([100, 100, 100, 255]), center_x - (label_w as i32 / 2), center_y + 80, PxScale::from(30.0), &font, &label);
        
        // 🟢 使用新的高质量绘制函数
        // 我们先简单计算一下文字宽度用于居中 (用 Normal 估算即可)
        let text_w = graphics::measure_text_width(&font, test_text, text_scale);
        let text_x = center_x - (text_w as i32 / 2);
        let text_y = center_y - 50;

        graphics::draw_text_high_quality(&mut canvas, text_color, text_x, text_y, text_scale, &font, test_text, mode);
    }

    println!("🚀 [DEBUG] 字体粗细网格生成耗时: {:.2?}", t_start.elapsed());
    
    let rgb_canvas = DynamicImage::ImageRgba8(canvas).to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    rgb_canvas.write_to(&mut buffer, image::ImageFormat::Jpeg).map_err(|e| format!("生成失败: {}", e))?;
    let base64_str = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/jpeg;base64,{}", base64_str))
}