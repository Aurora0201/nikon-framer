// src/processor/white/utils.rs

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size, draw_filled_rect_mut, draw_polygon_mut};
use imageproc::point::Point;
use imageproc::rect::Rect;
use ab_glyph::{Font, PxScale};
use rayon::prelude::*;
use std::f32::consts::PI;

// 引入统一错误类型
use crate::error::AppError;

/// 📐 对齐方式枚举
#[derive(Clone, Copy, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

// ============================================================================
// 1. 画布与合成 (Canvas & Composition) - 高性能区
// ============================================================================

/// 🚀 [高性能] 通用画布扩展器 (SIMD/Rayon Optimized)
///
/// 作用：创建一个比原图大的画布，填充满背景色，并将原图贴在指定位置。
/// 优化：使用 Rayon 并行处理每一行像素，避免了 "先全填白 -> 再贴图" 的内存写入冗余 (Overdraw)。
///
/// # 参数
/// * `img`: 原图
/// * `padding`: (top, bottom, left, right)
/// * `bg_color`: 背景色
pub fn create_expanded_canvas(
    img: &DynamicImage,
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
    bg_color: Rgba<u8>,
) -> Result<RgbaImage, AppError> {
    let (src_w, src_h) = img.dimensions();
    let canvas_w = src_w + left + right;
    let canvas_h = src_h + top + bottom;

    // 转换为 Rgba8 格式 (引用或拷贝)
    let src_buf = img.to_rgba8();
    
    // 预计算行的字节大小
    let row_len = (canvas_w * 4) as usize;

    // 使用 Rayon 并行迭代器生成每一行的数据
    // flat_map + collect 会自动处理内存拼接
    let raw_buffer: Vec<u8> = (0..canvas_h)
        .into_par_iter()
        .flat_map(|y| {
            // 预分配一行内存，避免扩容
            let mut row = Vec::with_capacity(row_len);
            
            // 判断当前行是否包含原图
            let is_in_src_y = y >= top && y < (top + src_h);

            if !is_in_src_y {
                // A. 纯背景区域 (顶部或底部)
                fill_row_color(&mut row, canvas_w, bg_color);
            } else {
                // B. 混合区域 (左背景 + 原图 + 右背景)
                
                // 1. 左边距
                fill_row_color(&mut row, left, bg_color);
                
                // 2. 原图拷贝 (使用 memcpy 加速)
                let src_y = y - top;
                let src_row_start = (src_y * src_w * 4) as usize;
                let src_row_end = src_row_start + (src_w * 4) as usize;
                
                // 安全边界检查
                if src_row_end <= src_buf.len() {
                    let src_slice = &src_buf.as_raw()[src_row_start..src_row_end];
                    row.extend_from_slice(src_slice);
                } else {
                    // 理论上不可达，防御性填充
                    fill_row_color(&mut row, src_w, bg_color);
                }

                // 3. 右边距
                fill_row_color(&mut row, right, bg_color);
            }
            row
        })
        .collect();

    // 构建 ImageBuffer
    RgbaImage::from_raw(canvas_w, canvas_h, raw_buffer)
        .ok_or_else(|| AppError::System("画布创建失败: 内存分配错误或尺寸溢出".to_string()))
}

/// 辅助：快速填充行颜色
#[inline(always)]
fn fill_row_color(row: &mut Vec<u8>, count: u32, color: Rgba<u8>) {
    for _ in 0..count {
        row.extend_from_slice(&color.0);
    }
}

/// 🛠️ [高性能] 逆向圆角遮罩 (Inverse Corner Mask)
///
/// 作用：在矩形原图的四个角，画上与背景色相同的“填充物”，视觉上产生圆角效果。
/// 优势：比 "先处理原图圆角再贴图" 快得多，因为它只修改四个角的少量像素，无需遍历全图。
///
/// # 参数
/// * `canvas`: 已经贴好原图的画布
/// * `img_x`, `img_y`: 原图在画布上的起始坐标
/// * `img_w`, `img_h`: 原图尺寸
/// * `radius`: 圆角半径
/// * `bg_color`: 必须与画布背景色一致
#[allow(dead_code)]
pub fn apply_inverse_corner_mask(
    canvas: &mut DynamicImage,
    img_x: u32,
    img_y: u32,
    img_w: u32,
    img_h: u32,
    radius: u32,
    bg_color: Rgba<u8>
) {
    if radius == 0 { return; }

    let r_sq = (radius * radius) as f32;
    let image_buffer = canvas.as_mut_rgba8().unwrap();

    // 🟢 修复点 1：将 check_fn 的类型改为 &dyn Fn(...)
    // 这告诉编译器："我接受任何实现了 Fn trait 的闭包引用"
    let mut mask_corner = |start_x: u32, start_y: u32, check_fn: &dyn Fn(f32, f32, f32) -> bool| {
        for dy in 0..radius {
            for dx in 0..radius {
                // 简单的抗锯齿中心采样 (+0.5)
                if check_fn(dx as f32 + 0.5, dy as f32 + 0.5, radius as f32) {
                    // 边界检查，防止越界
                    if start_x + dx < image_buffer.width() && start_y + dy < image_buffer.height() {
                        image_buffer.put_pixel(start_x + dx, start_y + dy, bg_color);
                    }
                }
            }
        }
    };

    // 🟢 修复点 2：在调用时，给闭包加上 & 符号 (传递引用)
    
    // 1. 左上角 (Top-Left)
    // 距离圆心 (r, r) 的距离 > r 则涂色
    mask_corner(img_x, img_y, &|dx, dy, r| {
        let dist_x = r - dx;
        let dist_y = r - dy;
        (dist_x * dist_x + dist_y * dist_y) > r_sq
    });

    // 2. 右上角 (Top-Right)
    mask_corner(img_x + img_w - radius, img_y, &|dx, dy, r| {
        let dist_x = dx; // 圆心在左侧
        let dist_y = r - dy;
        (dist_x * dist_x + dist_y * dist_y) > r_sq
    });

    // 3. 左下角 (Bottom-Left)
    mask_corner(img_x, img_y + img_h - radius, &|dx, dy, r| {
        let dist_x = r - dx;
        let dist_y = dy; // 圆心在上方
        (dist_x * dist_x + dist_y * dist_y) > r_sq
    });

    // 4. 右下角 (Bottom-Right)
    mask_corner(img_x + img_w - radius, img_y + img_h - radius, &|dx, dy, r| {
        let dist_x = dx;
        let dist_y = dy;
        (dist_x * dist_x + dist_y * dist_y) > r_sq
    });
}


// ============================================================================
// 2. 绘图原语 (Drawing Primitives) - 标准化区
// ============================================================================

/// ✍️ 通用文本绘制 (支持对齐)
///
/// 封装了 `text_size` 计算，自动处理左、中、右对齐的坐标偏移。
pub fn draw_text_aligned<F: Font>(
    canvas: &mut DynamicImage,
    font: &F,
    text: &str,
    x: i32, 
    y: i32, // 基准 Y 坐标 (通常是文字顶部或中心，取决于调用者逻辑，这里imageproc默认是顶部)
    size: f32,
    color: Rgba<u8>,
    align: TextAlign,
) {
    if text.is_empty() { return; }
    
    let scale = PxScale::from(size);
    let (w, _h) = text_size(scale, font, text);

    let draw_x = match align {
        TextAlign::Left => x,
        TextAlign::Center => x - (w as i32 / 2),
        TextAlign::Right => x - (w as i32),
    };

    draw_text_mut(canvas, color, draw_x, y, scale, font, text);
}

/// 🔷 绘制高质量实心圆角矩形 (Polyfill)
///
/// 使用多边形拟合圆角，比像素扫描质量更高。
/// 用于绘制徽章、标签背景、分隔线等。
pub fn draw_rounded_rect_polyfill(
    canvas: &mut DynamicImage, 
    rect: Rect, 
    radius: i32, 
    color: Rgba<u8>
) {
    let x = rect.left() as f32;
    let y = rect.top() as f32;
    let w = rect.width() as f32;
    let h = rect.height() as f32;
    
    let r = (radius as f32).min(w / 2.0).min(h / 2.0);

    if r <= 0.5 {
        draw_filled_rect_mut(canvas, rect, color);
        return;
    }

    let segments = 16; 
    let mut points: Vec<Point<i32>> = Vec::with_capacity(4 * (segments + 1)); 

    let mut add_arc = |cx: f32, cy: f32, start_angle: f32| {
        for i in 0..=segments {
            let angle = start_angle + (i as f32 / segments as f32) * (PI / 2.0);
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            points.push(Point::new(px.round() as i32, py.round() as i32));
        }
    };

    add_arc(x + w - r, y + r, -PI / 2.0);     // 右上
    add_arc(x + w - r, y + h - r, 0.0);       // 右下
    add_arc(x + r, y + h - r, PI / 2.0);      // 左下
    add_arc(x + r, y + r, PI);                // 左上

    // 🟢 🟢 🟢 修复开始 🟢 🟢 🟢
    // imageproc 要求首尾点不能相同，否则会 panic。
    // 在绘制胶囊形状（完全圆角）时，数学计算会导致首尾点重合，必须手动去重。
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        if first == last {
            points.pop(); // 移除最后一个重复的点
        }
    }
    // 🟢 🟢 🟢 修复结束 🟢 🟢 🟢

    draw_polygon_mut(canvas, &points, color);
}

/// 🧱 绘制垂直参数列 (Value + Label)
///
/// 专用于 WhiteMaster 风格的布局：上方是数值，下方是标签，整体居中。
pub fn draw_param_column<F: Font>(
    canvas: &mut DynamicImage,
    center_x: i32,
    val_y: i32,
    lbl_y: i32,
    value: &str,
    label: &str,
    font: &F,
    val_size: f32,
    lbl_size: f32,
    val_color: Rgba<u8>,
    lbl_color: Rgba<u8>
) {
    // 数值
    draw_text_aligned(
        canvas, font, value, 
        center_x, val_y, val_size, val_color, TextAlign::Center
    );
    // 标签
    draw_text_aligned(
        canvas, font, label, 
        center_x, lbl_y, lbl_size, lbl_color, TextAlign::Center
    );
}