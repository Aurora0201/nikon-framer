use image::{DynamicImage, ImageBuffer, Rgba, imageops, GenericImageView, RgbaImage};
use imageproc::rect::Rect;
// 引用同级目录下的 shapes 模块
use super::shapes::draw_rounded_rect_mut;

pub fn apply_rounded_glass_effect(img: &DynamicImage) -> RgbaImage {
    // 1. 准备数据，避免不必要的 clone
    let (w, h) = img.dimensions();
    
    // 参数计算
    let radius_ratio = 0.03;
    let radius = (w.min(h) as f32 * radius_ratio) as i32;
    // 半径平方，用于距离判断
    let r_sq = (radius * radius) as f32;
    // 边框逻辑保持不变
    let border_thickness = (w.max(h) as f32 * 0.002).clamp(3.0, 8.0) as u32;
    let glass_border_color = Rgba([255, 255, 255, 130]);

    // 2. 仅分配一次最终画布 (内存优化点：减少 2/3 的内存占用)
    let final_w = w + border_thickness * 2;
    let final_h = h + border_thickness * 2;
    let mut final_canvas = ImageBuffer::from_pixel(final_w, final_h, Rgba([0, 0, 0, 0]));

    // 3. 绘制玻璃边框底色
    let border_rect = Rect::at(0, 0).of_size(final_w, final_h);
    draw_rounded_rect_mut(
        &mut final_canvas,
        border_rect,
        radius + border_thickness as i32,
        glass_border_color,
    );

    // 4. 定义需要处理圆角的区域范围
    // 安全区域：中间不需要计算圆角的十字架区域
    let safe_x_start = radius as u32;
    let safe_x_end = w - radius as u32;
    let safe_y_start = radius as u32;
    let safe_y_end = h - radius as u32;

    // 5. 核心优化：直接在该画布上操作，无需中间层
    // 我们遍历原图的像素，将其“贴”到 final_canvas 上
    // 为了性能，我们不使用全图迭代器，而是手动拆分循环，或在循环中快速跳过

    // 这里为了代码简洁且高性能，我们遍历 source，但根据坐标决定处理逻辑
    // 由于 image 库的 get_pixel 有边界检查开销，我们在 Release 模式下直接通过坐标计算会更快
    
    // 获取原图的只读视图（如果原本就是 Rgba8，这里开销很小）
    let src_buf = img.to_rgba8(); 

    // A. 快速复制中间的大块区域 (内存拷贝，极快)
    // 技巧：我们可以把原图切成 9 宫格，中间的 5 格直接 copy，只有 4 个角需要遍历
    // 为了实现简单，我们采用逐行扫描，但在中间部分直接整行复制并非易事（因为要处理 alpha 混合）。
    // 但鉴于 overlay 的逻辑是 src 覆盖 dst，只要 alpha=255，直接覆盖即可。
    
    for y in 0..h {
        let is_y_in_corner = y < safe_y_start || y >= safe_y_end;
        
        for x in 0..w {
            let mut p = *src_buf.get_pixel(x, y); // 获取原图像素

            // 目标坐标
            let dest_x = x + border_thickness;
            let dest_y = y + border_thickness;

            // 只有在四个角落区域，才需要进行圆角遮罩计算
            if is_y_in_corner && (x < safe_x_start || x >= safe_x_end) {
                // 计算相对于圆心的坐标
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
                    // 情况1：完全在圆角外 -> 不绘制（保留底下的玻璃边框）
                    // 相当于蒙版 alpha = 0
                    continue; 
                } else if dist_sq > (radius - 1) as f32 * (radius - 1) as f32 {
                    // 情况2：圆角边缘 -> 简单的抗锯齿处理 (Anti-Aliasing)
                    // 计算覆盖率 (粗略版)
                    let dist = dist_sq.sqrt();
                    let alpha_factor = (radius as f32 - dist).clamp(0.0, 1.0);
                    
                    // 修改原像素 Alpha
                    let new_alpha = (p[3] as f32 * alpha_factor) as u8;
                    p = Rgba([p[0], p[1], p[2], new_alpha]);
                }
                // 情况3：完全在圆角内 -> 原样绘制
            }

            // 执行混合绘制 (Overlay)
            // 因为 final_canvas 上已经有边框颜色了，我们需要做 alpha blending
            // image::imageops::overlay 会自动处理，但这里我们是像素级操作
            // 手动 Blend: src over dst
            if p[3] == 255 {
                final_canvas.put_pixel(dest_x, dest_y, p);
            } else if p[3] > 0 {
                let bg = final_canvas.get_pixel(dest_x, dest_y);
                final_canvas.put_pixel(dest_x, dest_y, blend_pixel(*bg, p));
            }
            // if p[3] == 0, do nothing (keep border)
        }
    }

    final_canvas
}

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
