use std::{fs::File, io::BufReader};

use exif::{In, Reader, Tag};
use image::{DynamicImage, Rgba, imageops, GenericImageView, RgbaImage};
use imageproc::rect::Rect;
use log::{debug}; // 🟢 引入日志

// 🟢 引入我们的错误类型
use crate::error::AppError;
// 引用同级目录下的 shapes 模块
use super::shapes::draw_rounded_rect_mut;


/// 辅助：简单的 Alpha Blending (Src Over Dst)
/// 纯数学计算，不需要 Result
#[inline(always)]
fn blend_pixel(bg: Rgba<u8>, fg: Rgba<u8>) -> Rgba<u8> {
    let alpha = fg[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let r = (fg[0] as f32 * alpha + bg[0] as f32 * inv_alpha) as u8;
    let g = (fg[1] as f32 * alpha + bg[1] as f32 * inv_alpha) as u8;
    let b = (fg[2] as f32 * alpha + bg[2] as f32 * inv_alpha) as u8;
    let new_alpha = (fg[3] as f32 + bg[3] as f32 * inv_alpha) as u8;

    Rgba([r, g, b, new_alpha])
}

pub fn make_image_white(img: &DynamicImage) -> DynamicImage {
    let mut new_img = img.to_rgba8();
    
    for pixel in new_img.pixels_mut() {
        if pixel[3] > 0 {
            pixel[0] = 255;
            pixel[1] = 255;
            pixel[2] = 255;
        }
    }
    
    DynamicImage::ImageRgba8(new_img)
}

// 🟢 高性能模糊背景生成器
// 这属于图形计算，一般不会失败（除非内存耗尽 panic），所以保持不返回 Result
pub fn generate_blurred_background(
    img: &DynamicImage,
    target_w: u32,
    target_h: u32,
    blur_radius: f32,
    brightness_adj: i32, 
) -> DynamicImage {
    let (src_w, src_h) = img.dimensions();
    
    // 1. 定义极小的处理尺寸
    let min_dimension = 300.0;
    let scale_factor = (min_dimension / (src_w.min(src_h) as f64)).min(0.2); 
    
    let tiny_w = (src_w as f64 * scale_factor) as u32;
    let tiny_h = (src_h as f64 * scale_factor) as u32;

    // 2. 快速缩小
    let tiny_img = img.resize_exact(tiny_w, tiny_h, imageops::FilterType::Nearest);

    // 3. 计算裁切范围
    let ratio_target = target_w as f64 / target_h as f64;
    let ratio_tiny = tiny_w as f64 / tiny_h as f64;

    let (crop_w, crop_h) = if ratio_target > ratio_tiny {
        (tiny_w, (tiny_w as f64 / ratio_target) as u32)
    } else {
        ((tiny_h as f64 * ratio_target) as u32, tiny_h)
    };

    let crop_x = (tiny_w - crop_w) / 2;
    let crop_y = (tiny_h - crop_h) / 2;

    let cropped_tiny = tiny_img.crop_imm(crop_x, crop_y, crop_w, crop_h);

    // 4. 应用等效模糊
    let effective_blur = blur_radius * (scale_factor as f32);
    let mut blurred = cropped_tiny.blur(effective_blur);

    // 5. 调整亮度
    if brightness_adj != 0 {
        imageops::colorops::brighten(&mut blurred, brightness_adj);
    }

    // 6. 放大回目标尺寸
    blurred.resize_exact(target_w, target_h, imageops::FilterType::Triangle)
}


/// 🟢 [高性能] 绘制玻璃前景
pub fn draw_glass_foreground_on(
    canvas: &mut RgbaImage,
    img: &DynamicImage,
    dest_x: i64,
    dest_y: i64,
) {
    let (w, h) = img.dimensions();
    let (canvas_w, canvas_h) = canvas.dimensions();

    // 1. 参数计算
    let radius_ratio = 0.03;
    let radius = (w.min(h) as f32 * radius_ratio) as i32;
    let r_sq = (radius * radius) as f32;
    
    let border_thickness = (w.max(h) as f32 * 0.002).clamp(3.0, 8.0) as u32;
    let glass_border_color = Rgba([255, 255, 255, 130]);

    // 2. 绘制边框底座
    let border_x = dest_x - border_thickness as i64;
    let border_y = dest_y - border_thickness as i64;
    let border_w = w + border_thickness * 2;
    let border_h = h + border_thickness * 2;

    let border_rect = Rect::at(border_x as i32, border_y as i32)
        .of_size(border_w, border_h);
    
    draw_rounded_rect_mut(
        canvas,
        border_rect,
        radius + border_thickness as i32,
        glass_border_color,
    );

    // 3. 逐像素绘制原图
    let src_buf = img.to_rgba8();
    
    let safe_x_start = radius as u32;
    let safe_x_end = w - radius as u32;
    let safe_y_start = radius as u32;
    let safe_y_end = h - radius as u32;

    let start_x = 0.max(-dest_x) as u32;
    let start_y = 0.max(-dest_y) as u32;
    let end_x = w.min((canvas_w as i64 - dest_x) as u32);
    let end_y = h.min((canvas_h as i64 - dest_y) as u32);

    for y in start_y..end_y {
        let is_y_in_corner = y < safe_y_start || y >= safe_y_end;
        let cy = (dest_y + y as i64) as u32;
        
        for x in start_x..end_x {
            let mut p = *src_buf.get_pixel(x, y);
            
            // --- 圆角逻辑 ---
            if is_y_in_corner && (x < safe_x_start || x >= safe_x_end) {
                let dx = if x < safe_x_start {
                    (safe_x_start as f32 - x as f32) - 0.5
                } else {
                    (x as f32 - safe_x_end as f32) + 0.5
                };
                let dy = if y < safe_y_start {
                    (safe_y_start as f32 - y as f32) - 0.5
                } else {
                    (y as f32 - safe_y_end as f32) + 0.5
                };
                let dist_sq = dx * dx + dy * dy;

                if dist_sq > r_sq {
                    continue; 
                } else if dist_sq > (radius - 1) as f32 * (radius - 1) as f32 {
                    // 抗锯齿
                    let dist = dist_sq.sqrt();
                    let alpha_factor = (radius as f32 - dist).clamp(0.0, 1.0);
                    let new_alpha = (p[3] as f32 * alpha_factor) as u8;
                    p = Rgba([p[0], p[1], p[2], new_alpha]);
                }
            }
            
            // --- 写入画布 ---
            let cx = (dest_x + x as i64) as u32;
            
            if p[3] == 255 {
                canvas.put_pixel(cx, cy, p);
            } else if p[3] > 0 {
                let bg = canvas.get_pixel(cx, cy);
                let blended = blend_pixel(*bg, p);
                canvas.put_pixel(cx, cy, blended);
            }
        }
    }
}


/// ⚡️ 轻量级：仅读取 EXIF 方向信息
/// 🟢 修改：不返回错误，默认返回 1。如果打开失败，记录 debug 日志。
fn get_orientation(path: &str) -> u32 {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            debug!("⚠️ [Orientation] 无法读取方向信息 (IO error): {} -> {}", path, e);
            return 1;
        }
    };
    
    let mut bufreader = BufReader::new(&file);
    let reader = Reader::new();

    match reader.read_from_container(&mut bufreader) {
        Ok(exif) => {
            if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
                field.value.get_uint(0).unwrap_or(1)
            } else {
                1
            }
        },
        Err(_) => 1, // 解析失败也是 1
    }
}

/// 加载图片并自动旋转
/// 🔴 修改：返回 Result<DynamicImage, AppError>
pub fn load_image_auto_rotate(path: &str) -> Result<DynamicImage, AppError> {
    // 1. 获取方向
    let orientation = get_orientation(path);

    // 2. 解码图片
    // 🟢 这里使用了 ?，所以如果 image::open 失败，ImageError 会自动转为 AppError::Image 并返回
    let mut img = image::open(path)?;

    // 3. 根据方向调整
    if orientation != 1 {
        // debug!("🔄 [Load] 检测到方向 {}, 正在自动旋转...", orientation);
        img = match orientation {
            2 => img.fliph(),
            3 => img.rotate180(),
            4 => img.flipv(),
            5 => img.rotate90().fliph(),
            6 => img.rotate90(),
            7 => img.rotate270().fliph(),
            8 => img.rotate270(),
            _ => img,
        };
    }

    Ok(img)
}