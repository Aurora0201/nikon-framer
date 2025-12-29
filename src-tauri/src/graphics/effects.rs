use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use imageproc::rect::Rect;
// 引用同级目录下的 shapes 模块
use super::shapes::draw_rounded_rect_mut;

// 🟢 柔光阴影生成器
#[allow(dead_code)]
pub fn create_diffuse_shadow(target_w: u32, target_h: u32, border_size: u32, intensity: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let work_width = 150u32;
    let scale_factor = target_w as f32 / work_width as f32;
    let work_height = (target_h as f32 / scale_factor) as u32;
    let padding_x = (work_width as f32 * 0.3) as u32;
    let padding_y = (work_height as f32 * 0.3) as u32;
    let canvas_w = work_width + padding_x * 2;
    let canvas_h = work_height + padding_y * 2;
    let alpha_val = (120.0 * intensity).min(255.0) as u8;
    let shadow_color = Rgba([0, 0, 0, alpha_val]); 
    let mut tiny_layer = ImageBuffer::from_pixel(canvas_w, canvas_h, Rgba([0,0,0,0]));
    let rect_w = work_width;
    let rect_h = work_height;
    let tiny_radius = ((border_size as f32 * 0.5) / scale_factor) as i32;
    let rect = Rect::at(padding_x as i32, padding_y as i32).of_size(rect_w, rect_h);
    
    draw_rounded_rect_mut(&mut tiny_layer, rect, tiny_radius, shadow_color);
    
    let sigma = 8.0 * intensity;
    let blurred_tiny = image::imageops::blur(&tiny_layer, sigma);
    let final_w = (canvas_w as f32 * scale_factor) as u32;
    let final_h = (canvas_h as f32 * scale_factor) as u32;
    imageops::resize(&blurred_tiny, final_w, final_h, imageops::FilterType::Triangle)
}

// 🟢 应用圆角和玻璃描边质感
pub fn apply_rounded_glass_effect(img: &DynamicImage) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let rgba_img = img.to_rgba8();
    let (w, h) = rgba_img.dimensions();
    
    let radius_ratio = 0.03; 
    let radius = (w.min(h) as f32 * radius_ratio) as i32;
    let border_thickness = (w.max(h) as f32 * 0.002).clamp(3.0, 8.0) as u32; 
    let glass_border_color = Rgba([255, 255, 255, 130]); 

    let mut mask_layer = ImageBuffer::from_pixel(w, h, Rgba([0,0,0,0]));
    draw_rounded_rect_mut(&mut mask_layer, Rect::at(0,0).of_size(w,h), radius, Rgba([255,255,255,255]));

    let mut masked_source = ImageBuffer::from_pixel(w, h, Rgba([0,0,0,0]));
    for (x, y, pixel) in masked_source.enumerate_pixels_mut() {
        let src_pixel = rgba_img.get_pixel(x, y);
        let mask_pixel = mask_layer.get_pixel(x, y);
        let new_alpha = ((src_pixel[3] as u16 * mask_pixel[3] as u16) / 255) as u8;
        *pixel = Rgba([src_pixel[0], src_pixel[1], src_pixel[2], new_alpha]);
    }

    let final_w = w + border_thickness * 2;
    let final_h = h + border_thickness * 2;
    let mut final_canvas = ImageBuffer::from_pixel(final_w, final_h, Rgba([0,0,0,0]));

    let border_rect = Rect::at(0, 0).of_size(final_w, final_h);
    draw_rounded_rect_mut(&mut final_canvas, border_rect, radius + border_thickness as i32, glass_border_color);
    imageops::overlay(&mut final_canvas, &masked_source, border_thickness as i64, border_thickness as i64);

    final_canvas
}

pub fn make_image_white(img: &DynamicImage) -> DynamicImage {
    let mut new_img = img.to_rgba8();
    
    for pixel in new_img.pixels_mut() {
        // pixel[3] 是 Alpha 通道。只要不是完全透明，就把 RGB 设为白色
        // 这样可以保留抗锯齿边缘的半透明效果，但颜色变白
        if pixel[3] > 0 {
            pixel[0] = 255; // R
            pixel[1] = 255; // G
            pixel[2] = 255; // B
        }
    }
    
    DynamicImage::ImageRgba8(new_img)
}