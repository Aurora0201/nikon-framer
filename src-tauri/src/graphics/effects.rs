use std::{fs::File, io::BufReader};

use exif::{In, Reader, Tag};
use image::{DynamicImage, Rgba, imageops, GenericImageView, RgbaImage};
use imageproc::rect::Rect;
// 引用同级目录下的 shapes 模块
use super::shapes::draw_rounded_rect_mut;


/// 辅助：简单的 Alpha Blending (Src Over Dst)
/// 只有在边缘抗锯齿时才会调用，调用频率极低
#[inline(always)]
fn blend_pixel(bg: Rgba<u8>, fg: Rgba<u8>) -> Rgba<u8> {
    let alpha = fg[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let r = (fg[0] as f32 * alpha + bg[0] as f32 * inv_alpha) as u8;
    let g = (fg[1] as f32 * alpha + bg[1] as f32 * inv_alpha) as u8;
    let b = (fg[2] as f32 * alpha + bg[2] as f32 * inv_alpha) as u8;
    // 简单起见，Alpha 混合通常取 max 或者累加，这里保持背景 Alpha (通常是边框的半透明)
    // 或者混合 Alpha: fg.a + bg.a * (1 - fg.a)
    let new_alpha = (fg[3] as f32 + bg[3] as f32 * inv_alpha) as u8;

    Rgba([r, g, b, new_alpha])
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

// 🟢 [新增] 公共的高性能模糊背景生成器
// 逻辑源自 Master 模式的优化算法：缩图 -> 裁切 -> 模糊 -> 调亮 -> 放大
pub fn generate_blurred_background(
    img: &DynamicImage,
    target_w: u32,
    target_h: u32,
    blur_radius: f32,
    brightness_adj: i32, // 新增：亮度调整参数
) -> DynamicImage {
    let (src_w, src_h) = img.dimensions();
    
    // 1. 定义极小的处理尺寸 (保持短边 300px 用于模糊采样)
    let min_dimension = 300.0;
    let scale_factor = (min_dimension / (src_w.min(src_h) as f64)).min(0.2); 
    
    let tiny_w = (src_w as f64 * scale_factor) as u32;
    let tiny_h = (src_h as f64 * scale_factor) as u32;

    // 2. 快速缩小 (Nearest)
    let tiny_img = img.resize_exact(tiny_w, tiny_h, imageops::FilterType::Nearest);

    // 3. 计算裁切范围 (Aspect Fill 核心逻辑)
    // 确保模糊背景填满目标画布，且不拉伸变形
    let ratio_target = target_w as f64 / target_h as f64;
    let ratio_tiny = tiny_w as f64 / tiny_h as f64;

    let (crop_w, crop_h) = if ratio_target > ratio_tiny {
        // 目标更宽，裁掉上下
        (tiny_w, (tiny_w as f64 / ratio_target) as u32)
    } else {
        // 目标更高，裁掉左右
        ((tiny_h as f64 * ratio_target) as u32, tiny_h)
    };

    let crop_x = (tiny_w - crop_w) / 2;
    let crop_y = (tiny_h - crop_h) / 2;

    let cropped_tiny = tiny_img.crop_imm(crop_x, crop_y, crop_w, crop_h);

    // 4. 应用等效模糊
    let effective_blur = blur_radius * (scale_factor as f32);
    let mut blurred = cropped_tiny.blur(effective_blur);

    // 5. 调整亮度 (在小图上做，极快)
    if brightness_adj != 0 {
        imageops::colorops::brighten(&mut blurred, brightness_adj);
    }

    // 6. 放大回目标尺寸 (Triangle 插值保证平滑)
    blurred.resize_exact(target_w, target_h, imageops::FilterType::Triangle)
}


/// 🟢 [高性能] 直接将原图作为圆角玻璃前景绘制到目标画布上
/// 避免生成中间的大尺寸 glass_img，大幅减少内存分配和拷贝
pub fn draw_glass_foreground_on(
    canvas: &mut RgbaImage,      // 目标画布
    img: &DynamicImage,          // 源图
    dest_x: i64,                 // 目标位置 X
    dest_y: i64,                 // 目标位置 Y
) {
    let (w, h) = img.dimensions();
    let (canvas_w, canvas_h) = canvas.dimensions();

    // 1. 参数计算
    let radius_ratio = 0.03;
    let radius = (w.min(h) as f32 * radius_ratio) as i32;
    let r_sq = (radius * radius) as f32;
    
    let border_thickness = (w.max(h) as f32 * 0.002).clamp(3.0, 8.0) as u32;
    let glass_border_color = Rgba([255, 255, 255, 130]);

    // 2. 先在画布上画出边框底座 (直接操作 canvas)
    // 边框比原图大，所以要偏移回去
    let border_x = dest_x - border_thickness as i64;
    let border_y = dest_y - border_thickness as i64;
    let border_w = w + border_thickness * 2;
    let border_h = h + border_thickness * 2;

    // 绘制圆角矩形边框
    // 注意：draw_rounded_rect_mut 需要 Rect，坐标需要处理 i32 转换
    let border_rect = Rect::at(border_x as i32, border_y as i32)
        .of_size(border_w, border_h);
    
    draw_rounded_rect_mut(
        canvas,
        border_rect,
        radius + border_thickness as i32,
        glass_border_color,
    );

    // 3. 逐像素绘制原图 (带圆角裁切)
    // 这是一个手动的 "Overlay + Mask" 过程
    let src_buf = img.to_rgba8();
    
    let safe_x_start = radius as u32;
    let safe_x_end = w - radius as u32;
    let safe_y_start = radius as u32;
    let safe_y_end = h - radius as u32;

    // 为了性能，我们手动计算相交区域，只遍历可见部分
    // 避免 dest_x 为负数时的越界问题
    let start_x = 0.max(-dest_x) as u32;
    let start_y = 0.max(-dest_y) as u32;
    let end_x = w.min((canvas_w as i64 - dest_x) as u32);
    let end_y = h.min((canvas_h as i64 - dest_y) as u32);

    for y in start_y..end_y {
        let is_y_in_corner = y < safe_y_start || y >= safe_y_end;
        
        // 计算目标画布上的绝对 Y
        let cy = (dest_y + y as i64) as u32;
        
        for x in start_x..end_x {
            let mut p = *src_buf.get_pixel(x, y);
            
            // --- 圆角逻辑 (与之前相同) ---
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
                    continue; // 在圆角外，不绘制 (保留底下的玻璃边框)
                } else if dist_sq > (radius - 1) as f32 * (radius - 1) as f32 {
                    // 抗锯齿
                    let dist = dist_sq.sqrt();
                    let alpha_factor = (radius as f32 - dist).clamp(0.0, 1.0);
                    let new_alpha = (p[3] as f32 * alpha_factor) as u8;
                    p = Rgba([p[0], p[1], p[2], new_alpha]);
                }
            }
            
            // --- 写入画布 (Overlay 混合) ---
            let cx = (dest_x + x as i64) as u32;
            
            // 简单的 SrcOver 混合 (假设 canvas 不透明则直接覆盖更快)
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


/// ⚡️ 轻量级：仅读取 EXIF 方向信息，不解码图片
fn get_orientation(path: &str) -> u32 {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 1, // 打开失败当做默认方向
    };
    
    let mut bufreader = BufReader::new(&file);
    let reader = Reader::new();

    // read_from_container 只需要读取文件头部信息，开销很小
    match reader.read_from_container(&mut bufreader) {
        Ok(exif) => {
            if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
                // 尝试获取 u32 值，默认为 1
                field.value.get_uint(0).unwrap_or(1)
            } else {
                1
            }
        },
        Err(_) => 1,
    }
}

pub fn load_image_auto_rotate(path: &str) -> Result<DynamicImage, String> {
    // 1. 先获取方向 (轻量级 IO 操作)
    // 放在图片解码之前，如果这一步失败不影响后续解码，且几乎不占内存
    let orientation = get_orientation(path);

    // 2. 解码图片 (重量级内存操作)
    // 此时 img 可能是 Rgb8 (3字节) 或 Rgba8 (4字节)，保留原格式最省内存
    let mut img = image::open(path).map_err(|e| format!("图片加载失败: {}", e))?;

    // 3. 根据方向调整 (覆盖所有 8 种情况)
    // 🟢 优化：使用 img.rotate90() 等方法，它们会保留原图色彩空间(RGB/RGBA)，
    // 而不是像之前那样强制转为 ImageRgba8。
    if orientation != 1 {
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