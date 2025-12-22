use image::{ImageBuffer, Rgba, imageops};
use imageproc::drawing::draw_text_mut;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont}; 

// 🟢 计算文字宽度
pub fn measure_text_width(font: &FontRef, text: &str, scale: PxScale) -> u32 {
    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0;
    for c in text.chars() {
        let glyph_id = scaled_font.glyph_id(c);
        width += scaled_font.h_advance(glyph_id);
    }
    width.ceil() as u32
}

// 🟢 基础像素斜切 (内部使用 helper)
fn apply_raw_skew(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, skew_factor: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let offset_total = (h as f32 * skew_factor).abs().ceil() as u32;
    let new_w = w + offset_total;
    let new_h = h;

    let mut output = ImageBuffer::from_pixel(new_w, new_h, Rgba([0, 0, 0, 0]));

    for y in 0..h {
        // x' = x + (h - y) * factor
        let shift = ((h - 1 - y) as f32 * skew_factor).round() as i32;
        for x in 0..w {
            let pixel = img.get_pixel(x, y);
            if pixel[3] > 0 {
                let new_x = x as i32 + shift;
                if new_x >= 0 && new_x < new_w as i32 {
                    output.put_pixel(new_x as u32, y, *pixel);
                }
            }
        }
    }
    output
}

// 🟢 高质量斜切文字生成器 (SSAA 抗锯齿版)
pub fn generate_skewed_text_high_quality(
    text: &str,
    font: &FontRef,
    target_scale: PxScale,
    color: Rgba<u8>,
    skew_factor: f32
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // 1. 定义超采样倍率 (4倍)
    let supersample = 4.0;
    let draw_scale = PxScale::from(target_scale.y * supersample);

    // 2. 计算超大画布的尺寸
    let text_w = measure_text_width(font, text, draw_scale);
    let text_h = draw_scale.y.ceil() as u32;
    
    let skew_padding = (text_h as f32 * skew_factor).abs().ceil() as u32;
    let padding_x = 50; 
    let canvas_w = text_w + skew_padding + padding_x * 2;
    let canvas_h = text_h + padding_x; 

    let mut large_canvas = ImageBuffer::from_pixel(canvas_w, canvas_h, Rgba([0, 0, 0, 0]));

    // 3. 在超大画布上绘制普通文字
    let start_x = padding_x as i32;
    let start_y = (padding_x / 2) as i32;
    
    draw_text_mut(&mut large_canvas, color, start_x, start_y, draw_scale, font, text);

    // 4. 应用斜切
    let skewed_large = apply_raw_skew(&large_canvas, skew_factor);

    // 5. 缩小回目标尺寸 (Lanczos3 抗锯齿)
    let final_w = (skewed_large.width() as f32 / supersample).ceil() as u32;
    let final_h = (skewed_large.height() as f32 / supersample).ceil() as u32;

    imageops::resize(&skewed_large, final_w, final_h, imageops::FilterType::Lanczos3)
}

// 🟢 高质量抗锯齿加粗绘制 (用于直体文字)
pub fn draw_text_high_quality(
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    color: Rgba<u8>,
    x: i32,
    y: i32,
    target_scale: PxScale,
    font: &FontRef,
    text: &str,
    weight_mode: &str
) {
    if weight_mode == "Normal" {
        draw_text_mut(canvas, color, x, y, target_scale, font, text);
        return;
    }

    let offset_intensity: i32 = match weight_mode {
        "Medium" => 1,    
        "Bold" => 2,      
        "ExtraBold" => 3, 
        _ => 0,
    };

    let text_w = measure_text_width(font, text, target_scale);
    let text_h = target_scale.y as u32; 
    
    let supersample = 2;
    let padding = (offset_intensity * 4) as u32 + 20;
    let temp_w = (text_w * supersample) + padding;
    let temp_h = (text_h * supersample) + padding;

    let mut temp_canvas = ImageBuffer::from_pixel(temp_w, temp_h, Rgba([0, 0, 0, 0]));
    
    let draw_scale = PxScale::from(target_scale.y * supersample as f32);
    let start_x = 10; 
    let start_y = 10; 

    draw_text_mut(&mut temp_canvas, color, start_x, start_y, draw_scale, font, text);
    
    if offset_intensity > 0 {
        let offsets = [
            (offset_intensity, 0), (-offset_intensity, 0), (0, offset_intensity), (0, -offset_intensity), 
            (offset_intensity, offset_intensity), (-offset_intensity, -offset_intensity), 
            (offset_intensity, -offset_intensity), (-offset_intensity, offset_intensity)
        ];

        for (dx, dy) in offsets.iter() {
             draw_text_mut(&mut temp_canvas, color, start_x + dx, start_y + dy, draw_scale, font, text);
        }
    }

    let final_w = temp_w / supersample;
    let final_h = temp_h / supersample;
    
    let resized_text = imageops::resize(&temp_canvas, final_w, final_h, imageops::FilterType::Triangle);

    let paste_x = x - 5; 
    let paste_y = y - 5;
    
    imageops::overlay(canvas, &resized_text, paste_x as i64, paste_y as i64);
}