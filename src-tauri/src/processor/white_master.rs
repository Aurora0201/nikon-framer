use image::{DynamicImage, Rgba, GenericImageView, RgbaImage};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{draw_text_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use std::cmp::max;
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;

use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor; // 🟢 必须确保 Cargo.toml 中开启了 image 的 rayon 特性或单独引入了 rayon

// ==========================================
// 1. 数据结构定义
// ==========================================

// ==========================================
// 策略 5: 大师白底处理器 (WhiteMaster)
// ==========================================
pub struct WhiteMasterProcessor {
    pub main_font: Arc<Vec<u8>>,   // 参数字体
    pub script_font: Arc<Vec<u8>>, // 手写体
    pub serif_font: Arc<Vec<u8>>,  // 标题体
}

impl FrameProcessor for WhiteMasterProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let main = FontRef::try_from_slice(&self.main_font)
            .map_err(|_| "WhiteMaster: 参数字体解析失败")?;
        let script = FontRef::try_from_slice(&self.script_font)
            .map_err(|_| "WhiteMaster: 手写字体解析失败")?;
        let serif = FontRef::try_from_slice(&self.serif_font)
            .map_err(|_| "WhiteMaster: 衬线字体解析失败")?;

        // 🟢 使用 WhiteMasterInput 构造输入数据
        let input = WhiteMasterInput {
            iso: ctx.params.iso.map(|v| v.to_string()).unwrap_or_default(),
            aperture: ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default(),
            // 清洗快门速度字符串 (去除 's', 去除空格)
            shutter: ctx.params.shutter_speed
                .replace("s", "")
                .trim()
                .to_string(),
            focal: ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default(),
        };

        // 调用 white_master 模块的处理逻辑
        Ok(process(
            img, 
            input, 
            &main, 
            &script, 
            &serif
        ))
    }
}


/// Master 模式专用输入参数
/// 接收清洗后的参数字符串 (如 "100", "2.8", "50", "1/1000")
pub struct WhiteMasterInput {
    pub iso: String,
    pub aperture: String,
    pub shutter: String,
    pub focal: String,
}

// ==========================================
// 2. 布局配置中心
// ==========================================
struct WhiteMasterLayoutConfig {
    border_ratio: f32,
    bottom_ratio: f32,
    column_gap_ratio: f32,
    label_bottom_margin: f32,
    row_gap: f32,
    text_scale_val: f32,
    text_scale_lbl: f32,
    separator_scale: f32,
    header_bottom_margin: f32,
    header_script_size: f32,
    header_small_size: f32,
    header_gap_top: f32,
    header_gap_bottom: f32,
}

impl WhiteMasterLayoutConfig {
    fn default() -> Self {
        Self {
            border_ratio: 0.03,
            bottom_ratio: 0.4,
            column_gap_ratio: 0.18,
            label_bottom_margin: 0.18,
            row_gap: 0.001,
            text_scale_val: 0.13,
            text_scale_lbl: 0.07,
            separator_scale: 0.75,
            header_bottom_margin: 0.3,
            header_script_size: 0.12,
            header_small_size: 0.05,
            header_gap_top: -0.02,
            header_gap_bottom: 0.1,
        }
    }
}

// ==========================================
// 3. 高性能辅助函数
// ==========================================

/// 🟢 [高性能] 并行构建白底画布
/// 一次性完成：内存分配 + 边框填充 + 原图拷贝
/// 避免了 "先全填白 -> 再贴图" 的双重写入开销，大幅提升大图处理速度
fn fast_compose_white_canvas(img: &DynamicImage, border_size: u32, bottom_height: u32) -> RgbaImage {
    let (src_w, src_h) = img.dimensions();
    let canvas_w = src_w + border_size * 2;
    let canvas_h = src_h + border_size + bottom_height;

    // 引用原图数据 (零拷贝转换)
    let src_buf = img.to_rgba8(); 
    
    // 使用 Rayon 并行生成每一行的数据
    // collect() 会自动根据并行迭代器的结果分配正确的内存大小，无需手动预分配 buffer
    let raw_buffer: Vec<u8> = (0..canvas_h)
        .into_par_iter()
        .flat_map(|y| {
            // 预估这一行的大小，避免行内重分配
            let mut row = Vec::with_capacity((canvas_w * 4) as usize);
            
            // A. 顶部或底部区域 -> 全白填充
            if y < border_size || y >= (border_size + src_h) {
                row.resize((canvas_w * 4) as usize, 255);
            } 
            // B. 中间包含图片的区域
            else {
                // 1. 左边框 (白)
                let left_border_len = (border_size * 4) as usize;
                row.resize(left_border_len, 255);

                // 2. 原图数据 (内存拷贝)
                // 计算原图在当前行(y)的偏移量
                let src_y = y - border_size;
                let src_row_start = (src_y * src_w * 4) as usize;
                let src_row_end = src_row_start + (src_w * 4) as usize;
                
                // 安全获取切片并追加
                if src_row_end <= src_buf.len() {
                    let src_slice = &src_buf.as_raw()[src_row_start..src_row_end];
                    row.extend_from_slice(src_slice);
                } else {
                    // 理论上不会执行到这里，防御性填充
                    row.resize(row.len() + (src_w * 4) as usize, 255);
                }

                // 3. 右边框 (白)
                let final_len = (canvas_w * 4) as usize;
                row.resize(final_len, 255);
            }
            row
        })
        .collect(); // 合并所有行

    // 转换为 ImageBuffer
    RgbaImage::from_raw(canvas_w, canvas_h, raw_buffer).unwrap()
}

// ==========================================
// 4. 核心处理逻辑
// ==========================================

pub fn process(
    img: &DynamicImage,
    input: WhiteMasterInput,
    main_font: &FontRef,
    script_font: &FontRef,
    serif_font: &FontRef,
) -> DynamicImage {
    let start_total = Instant::now();
    let cfg = WhiteMasterLayoutConfig::default();

    let (img_w, img_h) = img.dimensions();
    let is_portrait = img_h > img_w;

    // 1. 计算布局尺寸
    let border_size = (img_h as f32 * cfg.border_ratio) as u32;
    let bottom_height = (img_h as f32 * cfg.bottom_ratio) as u32;
    
    // 2. 🟢 [高性能] 并行构建画布
    // 替代了旧的 from_pixel + overlay 逻辑
    let start_compose = Instant::now();
    let canvas_buffer = fast_compose_white_canvas(img, border_size, bottom_height);
    let mut canvas = DynamicImage::ImageRgba8(canvas_buffer);
    println!("[PERF] WhiteMaster Compose: {:?}", start_compose.elapsed());

    let (canvas_w, canvas_h) = canvas.dimensions();

    // 3. 解构输入参数
    let iso_val = input.iso;
    let aperture_val = input.aperture;
    let focal_val = input.focal;
    let shutter_val = input.shutter;

    // 4. 排版计算
    let bh = bottom_height as f32;
    let center_x = canvas_w as i32 / 2;
    
    // 竖构图时缩小参数区文字
    let param_scale = if is_portrait { 0.6 } else { 1.0 };

    // --- A. 参数区坐标 ---
    let val_size = bh * cfg.text_scale_val * param_scale;
    let lbl_size = bh * cfg.text_scale_lbl * param_scale;
    let margin_bottom = bh * cfg.label_bottom_margin;
    let row_gap = if is_portrait { bh * cfg.row_gap * 0.5 } else { bh * cfg.row_gap };

    let label_draw_y = (canvas_h as f32 - margin_bottom - lbl_size) as i32;
    let value_draw_y = (label_draw_y as f32 - row_gap - val_size) as i32;

    // --- B. Header 区坐标 ---
    let params_top_y = value_draw_y as f32;
    let script_size = bh * cfg.header_script_size; 
    let small_size = bh * cfg.header_small_size;   
    let gap_top = bh * cfg.header_gap_top;
    let gap_bottom = bh * cfg.header_gap_bottom;

    let script_baseline_y = params_top_y - (bh * cfg.header_bottom_margin);
    let line2_y = script_baseline_y as i32;
    let line1_y = (script_baseline_y - (script_size * 0.5) - gap_top) as i32;
    let line3_y = (script_baseline_y + (script_size * 0.1) + gap_bottom) as i32;

    // --- C. 分隔线坐标 ---
    let sep_top = value_draw_y as f32;
    let sep_bottom = label_draw_y as f32 + lbl_size;
    let sep_full_h = sep_bottom - sep_top;
    let sep_actual_h = sep_full_h * cfg.separator_scale;
    let sep_center_y = sep_top + (sep_full_h / 2.0);

    // 5. 颜色定义 (视觉优化版)
    // 参数数值: 深灰
    let text_color = Rgba([40, 40, 40, 255]);         
    // 标签 (ISO/F): 浅灰
    let label_color = Rgba([150, 150, 150, 255]);     
    // 手写体: 钢笔蓝 (Royal Blue)
    let script_color = Rgba([35, 65, 140, 255]);       
    // Master Series 标题: 冷调灰
    let small_title_color = Rgba([100, 110, 120, 255]); 
    // 分隔线: 可见度较高的灰
    let sep_color = Rgba([160, 160, 160, 255]);       

    // 6. 绘制 Header
    draw_centered_text(&mut canvas, "MASTER SERIES", center_x, line1_y, serif_font, PxScale{x: small_size, y: small_size}, small_title_color);
    draw_centered_text(&mut canvas, "The decisive moment", center_x, line2_y, script_font, PxScale{x: script_size, y: script_size}, script_color);
    draw_wide_text(&mut canvas, center_x, line3_y, "PHOTOGRAPH", serif_font, small_size, small_title_color);

    // 7. 绘制参数列
    let gap = (canvas_w as f32 * cfg.column_gap_ratio) as i32;

    if !iso_val.is_empty() {
        draw_column_absolute(&mut canvas, center_x - gap * 1 - (gap / 2), value_draw_y, label_draw_y, &iso_val, "ISO", main_font, val_size, lbl_size, text_color, label_color);
    }
    if !aperture_val.is_empty() {
        draw_column_absolute(&mut canvas, center_x - (gap / 2), value_draw_y, label_draw_y, &aperture_val, "F", main_font, val_size, lbl_size, text_color, label_color);
    }
    if !focal_val.is_empty() {
        draw_column_absolute(&mut canvas, center_x + (gap / 2), value_draw_y, label_draw_y, &focal_val, "mm", main_font, val_size, lbl_size, text_color, label_color);
    }
    if !shutter_val.is_empty() {
        draw_column_absolute(&mut canvas, center_x + gap * 1 + (gap / 2), value_draw_y, label_draw_y, &shutter_val, "S", main_font, val_size, lbl_size, text_color, label_color);
    }

    // 8. 绘制竖线
    draw_separator(&mut canvas, center_x - gap, sep_center_y, sep_actual_h, sep_color);
    draw_separator(&mut canvas, center_x, sep_center_y, sep_actual_h, sep_color);
    draw_separator(&mut canvas, center_x + gap, sep_center_y, sep_actual_h, sep_color);

    println!("[PERF] WhiteMaster Total: {:?}", start_total.elapsed());

    canvas
}

// ==========================================
// 5. 绘制辅助函数
// ==========================================

/// 绘制宽字距文本 (PHOTOGRAPH)
fn draw_wide_text(canvas: &mut DynamicImage, center_x: i32, y: i32, text: &str, font: &FontRef, size: f32, color: Rgba<u8>) {
    let scale = PxScale { x: size, y: size };
    let tracking = size * 0.4; 
    let mut total_width = 0.0;
    
    // 计算总宽
    let char_widths: Vec<f32> = text.chars().map(|c| {
        let (w, _) = imageproc::drawing::text_size(scale, font, &c.to_string());
        total_width += w as f32 + tracking;
        w as f32
    }).collect();
    
    if total_width > 0.0 { total_width -= tracking; }
    
    // 逐字绘制
    let mut current_x = center_x as f32 - (total_width / 2.0);
    for (i, c) in text.chars().enumerate() {
        draw_text_mut(canvas, color, current_x as i32, y, scale, font, &c.to_string());
        current_x += char_widths[i] + tracking;
    }
}

/// 绘制参数列 (数值 + 标签)
fn draw_column_absolute(canvas: &mut DynamicImage, x: i32, val_y: i32, lbl_y: i32, value: &str, label: &str, font: &FontRef, val_size: f32, lbl_size: f32, val_color: Rgba<u8>, lbl_color: Rgba<u8>) {
    draw_centered_text(canvas, value, x, val_y, font, PxScale { x: val_size, y: val_size }, val_color);
    draw_centered_text(canvas, label, x, lbl_y, font, PxScale { x: lbl_size, y: lbl_size }, lbl_color);
}

/// 绘制分隔线 (动态加粗版)
/// 替代了细线绘制，使用矩形填充以确保在高像素图片下可见
fn draw_separator(canvas: &mut DynamicImage, x: i32, center_y: f32, height: f32, color: Rgba<u8>) {
    let (w, _) = canvas.dimensions();
    
    // 动态计算线宽：0.0015 比例系数
    // 6000px 图片 -> 9px 宽
    // 最小宽度限制为 4px
    let thickness = max(4, (w as f32 * 0.0015).ceil() as u32);

    // 计算起始 X 坐标 (保持居中)
    let start_x = x - (thickness as i32 / 2);
    let start_y = (center_y - (height / 2.0)) as i32;

    // 创建矩形
    let rect = Rect::at(start_x, start_y).of_size(thickness, height as u32);

    // 绘制填充矩形
    draw_filled_rect_mut(canvas, rect, color);
}

/// 绘制居中文本
fn draw_centered_text(canvas: &mut DynamicImage, text: &str, x: i32, y: i32, font: &FontRef, scale: PxScale, color: Rgba<u8>) {
    let (text_w, _text_h) = imageproc::drawing::text_size(scale, font, text);
    let draw_x = x - (text_w as i32 / 2);
    draw_text_mut(canvas, color, draw_x, y, scale, font, text);
}