// src/processor/white/white_master_v2.rs

use image::{DynamicImage, Rgba, GenericImageView};
use ab_glyph::{Font, FontArc, PxScale};
use imageproc::drawing::{draw_filled_rect_mut, text_size};
use imageproc::rect::Rect;
use log::{info, debug};
use std::time::Instant;

use crate::error::AppError;
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor;

// 引入高性能工具箱
use super::utils::{
    create_expanded_canvas, 
    draw_text_aligned, 
    draw_param_column, 
    TextAlign
};

// ==========================================
// 1. 结构体定义
// ==========================================

pub struct WhiteMasterProcessorV2 {
    pub main_font: FontArc,   // 用于参数数值
    pub script_font: FontArc, // 用于 "The decisive moment"
    pub serif_font: FontArc,  // 用于 "MASTER SERIES" / "PHOTOGRAPH"
}

impl FrameProcessor for WhiteMasterProcessorV2 {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, AppError> {
        let t_start = Instant::now();

        // 1. 数据清洗 (Data Cleaning)
        // 避免在绘图循环中做字符串处理
        let iso = ctx.params.iso.map(|v| v.to_string()).unwrap_or_default();
        let aperture = ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default();
        let focal = ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default();
        
        // 移除 "s" 并去除空格 (例如 "1/1000 s" -> "1/1000")
        let shutter = ctx.params.shutter_speed
            .replace("s", "")
            .trim()
            .to_string();

        // 2. 核心处理
        let result = process_internal(
            img,
            &self.main_font,
            &self.script_font,
            &self.serif_font,
            &iso, &aperture, &shutter, &focal
        )?;

        info!("✨ [PERF] WhiteMaster V2 processed in {:.2?}", t_start.elapsed());
        Ok(result)
    }
}

// ==========================================
// 2. 布局配置
// ==========================================

struct MasterConfig {
    border_ratio: f32,       // 四周白边比例
    bottom_ratio: f32,       // 底部留白比例
    
    // 字体比例 (相对于 bottom_height)
    text_scale_val: f32,     // 参数数值
    text_scale_lbl: f32,     // 参数标签
    header_script_size: f32, // 手写体
    header_small_size: f32,  // 顶部/底部小标题
    
    // 间距比例
    column_gap_ratio: f32,   // 列间距
    label_bottom_margin: f32,// 参数标签距离底部的边距
    header_gap_top: f32,     // 顶部标题微调
    header_gap_bottom: f32,  // 底部标题微调
    
    // 分隔线
    separator_scale: f32,    // 分隔线高度相对于参数区高度的比例
    
    // 颜色
    color_text_val: Rgba<u8>,
    color_text_lbl: Rgba<u8>,
    color_script: Rgba<u8>,  // 皇家蓝
    color_title: Rgba<u8>,   // 冷灰
    color_sep: Rgba<u8>,
    bg_color: Rgba<u8>,
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            border_ratio: 0.03,
            bottom_ratio: 0.40,
            
            text_scale_val: 0.13,
            text_scale_lbl: 0.07,
            header_script_size: 0.18,
            header_small_size: 0.08,
            
            column_gap_ratio: 0.18,
            label_bottom_margin: 0.18,
            header_gap_top: 0.09,
            header_gap_bottom: 0.08,
            
            separator_scale: 0.75,
            
            color_text_val: Rgba([40, 40, 40, 255]),      // 深灰数值
            color_text_lbl: Rgba([150, 150, 150, 255]),   // 浅灰标签
            color_script: Rgba([35, 65, 140, 255]),       // 皇家蓝手写体
            color_title: Rgba([100, 110, 120, 255]),      // 标题冷灰
            color_sep: Rgba([180, 180, 180, 255]),        // 分隔线
            bg_color: Rgba([255, 255, 255, 255]),
        }
    }
}

// ==========================================
// 3. 核心处理逻辑
// ==========================================

fn process_internal(
    img: &DynamicImage,
    main_font: &FontArc,
    script_font: &FontArc,
    serif_font: &FontArc,
    iso: &str, aperture: &str, shutter: &str, focal: &str
) -> Result<DynamicImage, AppError> {

    let cfg = MasterConfig::default();
    let (src_w, src_h) = img.dimensions();

    // -------------------------------------------------------------
    // A. 尺寸计算
    // -------------------------------------------------------------
    let border = (src_h as f32 * cfg.border_ratio).round() as u32;
    let bottom = (src_h as f32 * cfg.bottom_ratio).round() as u32;
    
    // Master 风格：四周有 border，底部额外增加 bottom
    // Canvas Height = src_h + border(Top) + border(Bottom) + bottom(Extra)
    // 但通常设计是：Top=border, Bottom=border+bottom, Left=border, Right=border
    let top_pad = border;
    let bottom_pad = border + bottom;
    let left_pad = border;
    let right_pad = border;

    debug!("📐 [Layout] Master: {}x{}, BottomArea={}", src_w, src_h, bottom);

    // -------------------------------------------------------------
    // B. 画布构建 (高性能 Rayon)
    // -------------------------------------------------------------
    let t_canvas = Instant::now();
    let mut canvas = DynamicImage::ImageRgba8(
        create_expanded_canvas(
            img, top_pad, bottom_pad, left_pad, right_pad, cfg.bg_color
        )?
    );
    debug!("  -> [PERF] Canvas compose: {:.2?}", t_canvas.elapsed());

    let (canvas_w, canvas_h) = canvas.dimensions();
    let center_x = (canvas_w / 2) as i32;
    let bh = bottom as f32; // 底部核心区域的高度基准

    // -------------------------------------------------------------
    // C. 坐标系统计算
    // -------------------------------------------------------------

    // C1. 参数区 (Params)
    let val_size = bh * cfg.text_scale_val;
    let lbl_size = bh * cfg.text_scale_lbl;
    let margin_bottom = bh * cfg.label_bottom_margin;
    
    // 计算参数行的 Y 坐标
    // Label 在底部
    let label_y = (canvas_h as f32 - margin_bottom - lbl_size) as i32;
    // Value 在 Label 上方 (加一点间距)
    let value_y = label_y - (val_size as i32) - (bh * 0.02) as i32;

    // C2. 标题区 (Header)
    // 位于 content_base_y 和 value_y 之间
    let params_top_y = value_y as f32;
    let script_size = bh * cfg.header_script_size;
    let small_size = bh * cfg.header_small_size;
    
    // 脚本体基线
    let script_baseline_y = params_top_y - (bh * 0.4); // 稍微往上提
    
    let line_script_y = script_baseline_y as i32;
    let line_top_y = (script_baseline_y - (script_size * 0.8) + (bh * cfg.header_gap_top)) as i32;
    let line_bottom_y = (script_baseline_y + (script_size * 0.5) + (bh * cfg.header_gap_bottom)) as i32;

    // C3. 分隔线 (Separators)
    let sep_top = value_y as f32;
    let sep_bottom = (label_y as f32) + lbl_size;
    let sep_h = (sep_bottom - sep_top) * cfg.separator_scale;
    let sep_center_y = sep_top + (sep_bottom - sep_top) / 2.0;
    
    // 动态线宽: 基于画布宽度的 0.15%
    let sep_w = (canvas_w as f32 * 0.0015).max(2.0) as u32;

    // -------------------------------------------------------------
    // D. 绘制内容
    // -------------------------------------------------------------

    // 1. 绘制 Header
    // Line 1: MASTER SERIES
    draw_text_aligned(
        &mut canvas, serif_font, "MASTER SERIES", 
        center_x, line_top_y, small_size, cfg.color_title, TextAlign::Center
    );
    
    // Line 2: The decisive moment (Script)
    draw_text_aligned(
        &mut canvas, script_font, "The decisive moment", 
        center_x, line_script_y, script_size, cfg.color_script, TextAlign::Center
    );
    
    // Line 3: PHOTOGRAPH (Wide Spacing)
    // 这里调用私有辅助函数来实现宽字间距
    draw_wide_text(
        &mut canvas, serif_font, "PHOTOGRAPH", 
        center_x, line_bottom_y, small_size, cfg.color_title
    );

    // 2. 绘制参数列 & 分隔线
    let gap = (canvas_w as f32 * cfg.column_gap_ratio) as i32;
    let col_w = gap / 2; // 列宽的一半，用于定位

    // Column 1: ISO
    if !iso.is_empty() {
        draw_param_column(
            &mut canvas, center_x - gap - col_w, value_y, label_y, 
            iso, "ISO", main_font, val_size, lbl_size, cfg.color_text_val, cfg.color_text_lbl
        );
    }
    
    // Column 2: Aperture
    if !aperture.is_empty() {
        draw_param_column(
            &mut canvas, center_x - col_w, value_y, label_y, 
            aperture, "F", main_font, val_size, lbl_size, cfg.color_text_val, cfg.color_text_lbl
        );
    }
    
    // Column 3: Focal Length
    if !focal.is_empty() {
        draw_param_column(
            &mut canvas, center_x + col_w, value_y, label_y, 
            focal, "mm", main_font, val_size, lbl_size, cfg.color_text_val, cfg.color_text_lbl
        );
    }
    
    // Column 4: Shutter
    if !shutter.is_empty() {
        draw_param_column(
            &mut canvas, center_x + gap + col_w, value_y, label_y, 
            shutter, "S", main_font, val_size, lbl_size, cfg.color_text_val, cfg.color_text_lbl
        );
    }

    // 3. 绘制分隔线 (使用圆角矩形 polyfill 提升质感)
    // 🟢 修改后：使用 draw_filled_rect_mut (极速，稳定)
    let sep_h_u32 = sep_h as u32;
    let start_y = (sep_center_y - sep_h / 2.0) as i32;
    
    // Line 1 (Left)
    let rect1 = Rect::at(center_x - gap - (sep_w as i32 / 2), start_y).of_size(sep_w, sep_h_u32);
    draw_filled_rect_mut(&mut canvas, rect1, cfg.color_sep);

    // Line 2 (Center)
    let rect2 = Rect::at(center_x - (sep_w as i32 / 2), start_y).of_size(sep_w, sep_h_u32);
    draw_filled_rect_mut(&mut canvas, rect2, cfg.color_sep);

    // Line 3 (Right)
    let rect3 = Rect::at(center_x + gap - (sep_w as i32 / 2), start_y).of_size(sep_w, sep_h_u32);
    draw_filled_rect_mut(&mut canvas, rect3, cfg.color_sep);

    Ok(canvas)
}

// ==========================================
// 4. 私有辅助函数
// ==========================================

/// 绘制宽字距文本 (特供 Master 风格)
/// 逻辑：计算总宽 -> 居中起始点 -> 逐字绘制并增加间距
fn draw_wide_text<F: Font>(
    canvas: &mut DynamicImage, 
    font: &F, 
    text: &str, 
    center_x: i32, 
    y: i32, 
    size: f32, 
    color: Rgba<u8>
) {
    let scale = PxScale::from(size);
    let tracking = size * 0.4; // 字间距系数
    
    // 1. 预计算每个字符的宽度
    let char_widths: Vec<f32> = text.chars().map(|c| {
        let (w, _) = text_size(scale, font, &c.to_string());
        w as f32
    }).collect();
    
    // 2. 计算总宽度 (字符宽 + 间距)
    let total_chars_width: f32 = char_widths.iter().sum();
    let total_spacing = if text.len() > 1 {
        tracking * (text.len() - 1) as f32
    } else {
        0.0
    };
    let total_width = total_chars_width + total_spacing;

    // 3. 计算起始 X
    let mut current_x = center_x as f32 - (total_width / 2.0);

    // 4. 逐字绘制
    for (i, c) in text.chars().enumerate() {
        // draw_text_aligned 这里用 Left 对齐即可，因为我们已经算好了确切的 current_x
        draw_text_aligned(
            canvas, font, &c.to_string(), 
            current_x.round() as i32, y, 
            size, color, TextAlign::Left
        );
        current_x += char_widths[i] + tracking;
    }
}