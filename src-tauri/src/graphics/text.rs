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