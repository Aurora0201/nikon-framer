// src/processor/white/white_modern_v2.rs

use image::{DynamicImage, Rgba, imageops, GenericImageView};
use imageproc::drawing::text_size;
use imageproc::rect::Rect;
use ab_glyph::{Font, FontArc, PxScale};
use log::{info, debug};
use std::time::Instant;
use std::cmp::max;

use crate::error::AppError;
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor;
// 假设阴影模块位置不变
use crate::graphics::shadow::ShadowProfile; 

// 引入高性能工具箱
use super::utils::{
    create_expanded_canvas, 
    draw_text_aligned, 
    draw_rounded_rect_polyfill, 
    TextAlign
};

// ==========================================
// 1. 结构体定义
// ==========================================

pub struct WhiteModernProcessorV2 {
    pub font_bold: FontArc,    // 用于参数数值
    pub font_medium: FontArc,  // 用于机型 / 参数标签
    pub font_regular: FontArc, // 备用
    pub font_script: FontArc,  // 用于品牌 (手写体)
}

impl FrameProcessor for WhiteModernProcessorV2 {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, AppError> {
        let t_start = Instant::now();

        // 1. 数据准备
        let brand = ctx.brand.to_string();
        let model = ctx.model_name.clone();
        
        let iso = ctx.params.iso.map(|v| v.to_string()).unwrap_or_default();
        let aperture = ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default();
        let focal = ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default();
        let shutter = ctx.params.shutter_speed.replace("s", "").trim().to_string();

        // 2. 核心处理
        let result = process_internal(
            img,
            &self.font_bold,
            &self.font_medium,
            &self.font_script,
            &brand, &model,
            &iso, &aperture, &shutter, &focal
        )?;

        info!("✨ [PERF] WhiteModern V2 processed in {:.2?}", t_start.elapsed());
        Ok(result)
    }
}

// ==========================================
// 2. 布局配置
// ==========================================

struct ModernConfig {
    border_ratio: f32,       // 边框比例
    bottom_ratio: f32,       // 底部比例
    
    // Header 布局
    model_text_scale: f32,   // 机型字号
    script_scale_ratio: f32, // 手写体相对于机型字号的比例
    gap_brand_model: f32,    // 品牌与机型间距
    gap_image_model: f32,    // 图片与 Header 间距
    header_y_nudge: f32,     // Header 整体微调
    script_y_nudge: f32,     // 手写体垂直微调
    model_y_nudge: f32,      // 机型垂直微调
    
    // 胶囊 (Badge) 布局
    badge_height_ratio: f32, // 胶囊高度比例
    badge_width_ratio: f32,  // 胶囊宽度比例
    badge_gap: f32,          // 胶囊间距
    gap_model_params: f32,   // Header 与胶囊的间距
    
    // 参数文字
    param_val_scale: f32,
    param_lbl_scale: f32,
    val_y_nudge_ratio: f32,  // 数值垂直修正
    
    // 颜色
    color_text_black: Rgba<u8>,
    color_text_gray: Rgba<u8>,
    color_text_blue: Rgba<u8>, // 钢笔蓝
    color_border: Rgba<u8>,    // 胶囊边框
    bg_color: Rgba<u8>,
}

impl Default for ModernConfig {
    fn default() -> Self {
        Self {
            border_ratio: 0.05,
            bottom_ratio: 0.35,
            
            model_text_scale: 0.20,
            script_scale_ratio: 1.6,
            gap_brand_model: 0.1,
            gap_image_model: 0.18,
            header_y_nudge: 0.05,
            script_y_nudge: 0.3,
            model_y_nudge: 0.18,
            
            badge_height_ratio: 0.22,
            badge_width_ratio: 1.8,
            badge_gap: 0.40,
            gap_model_params: 0.15,
            
            param_val_scale: 0.12,
            param_lbl_scale: 0.095,
            val_y_nudge_ratio: 0.28,
            
            color_text_black: Rgba([20, 20, 20, 255]),
            color_text_gray: Rgba([100, 100, 100, 255]),
            color_text_blue: Rgba([35, 65, 140, 255]),
            color_border: Rgba([180, 180, 180, 255]),
            bg_color: Rgba([255, 255, 255, 255]),
        }
    }
}

// ==========================================
// 3. 核心处理逻辑
// ==========================================

fn process_internal(
    img: &DynamicImage,
    font_bold: &FontArc,
    font_medium: &FontArc,
    font_script: &FontArc,
    brand: &str, model: &str,
    iso: &str, aperture: &str, shutter: &str, focal: &str
) -> Result<DynamicImage, AppError> {

    let cfg = ModernConfig::default();
    let (src_w, src_h) = img.dimensions();

    // -------------------------------------------------------------
    // A. 尺寸计算
    // -------------------------------------------------------------
    // 竖构图优化：整体比例缩小，避免留白过多
    let is_portrait = src_h > src_w;
    let portrait_scale = if is_portrait { 0.55 } else { 1.0 };

    let border = (src_h as f32 * cfg.border_ratio * portrait_scale).round() as u32;
    let bottom = (src_h as f32 * cfg.bottom_ratio * portrait_scale).round() as u32;

    // Modern 布局：Top=border, Bottom=bottom+border, Left=border, Right=border
    let top_pad = border;
    let bottom_pad = border + bottom;
    let left_pad = border;
    let right_pad = border;

    debug!("📐 [Layout] Modern: {}x{}, Border={}, Bottom={}", src_w, src_h, border, bottom);

    // -------------------------------------------------------------
    // B. 画布构建 & 阴影处理
    // -------------------------------------------------------------
    let t_canvas = Instant::now();
    
    // 1. 快速创建底板 (此时原图已被贴上)
    let mut canvas = DynamicImage::ImageRgba8(
        create_expanded_canvas(
            img, top_pad, bottom_pad, left_pad, right_pad, cfg.bg_color
        )?
    );

    // 2. 绘制阴影 (Shadow)
    // 注意：阴影通常画在图片周围。create_expanded_canvas 已经贴了图。
    // 如果 ShadowProfile 是叠加式的（半透明），直接画在上面即可。
    // 如果 ShadowProfile 可能会覆盖原图内容，我们需要在画完阴影后，
    // 把原图再贴一遍以确保清晰度（这比手动计算遮罩快得多）。
    
    let img_center_x = (left_pad + src_w / 2) as i64;
    let img_center_y = (top_pad + src_h / 2) as i64;
    
    // 假设 ShadowProfile 存在并可用
    ShadowProfile::preset_standard().draw_adaptive_shadow_on(
        canvas.as_mut_rgba8().unwrap(),
        (src_w, src_h),
        (img_center_x, img_center_y)
    );

    // 3. 重绘原图 (确保原图在阴影之上，边缘清晰)
    // 这一步开销很小 (Memcpy)，但能保证视觉正确性
    imageops::overlay(&mut canvas, img, left_pad as i64, top_pad as i64);

    debug!("  -> [PERF] Canvas & Shadow: {:.2?}", t_canvas.elapsed());

    let (canvas_w, _canvas_h) = canvas.dimensions();
    let center_x = (canvas_w / 2) as i32;
    let bh = bottom as f32; // 底部核心区域高度

    // -------------------------------------------------------------
    // C. 绘制 Header (Brand + Model)
    // -------------------------------------------------------------
    let content_start_y = (top_pad + src_h) as i32;
    
    // 字号计算
    let model_size = bh * cfg.model_text_scale;
    let script_size = model_size * cfg.script_scale_ratio;

    // 测量宽度
    let (brand_w, brand_h) = text_size(PxScale::from(script_size), font_script, brand);
    let (model_w, model_h) = text_size(PxScale::from(model_size), font_medium, model);

    // 布局坐标
    let gap_px = (bh * cfg.gap_brand_model) as i32;
    let header_total_w = brand_w as i32 + gap_px + model_w as i32;
    let start_x = center_x - (header_total_w / 2);

    let header_base_y = content_start_y + (bh * cfg.gap_image_model) as i32;
    let header_y = header_base_y + (bh * cfg.header_y_nudge) as i32;
    
    // 对齐基准线 (以机型文字的垂直中心为基准)
    let header_center_y_line = header_y + (model_h as i32 / 2);

    // 1. 绘制 Brand (Script)
    let brand_offset_ratio = get_brand_script_offset(brand); // 品牌微调
    let brand_offset_px = (script_size * brand_offset_ratio) as i32;
    
    let script_y_start = header_center_y_line - (brand_h as i32 / 2);
    let script_final_y = script_y_start - (script_size * cfg.script_y_nudge) as i32 + brand_offset_px;

    draw_text_aligned(
        &mut canvas, font_script, brand,
        start_x, script_final_y,
        script_size, cfg.color_text_blue, TextAlign::Left
    );

    // 2. 绘制 Model
    let model_x = start_x + brand_w as i32 + gap_px;
    let model_final_y = header_y - (model_size * cfg.model_y_nudge) as i32;

    draw_text_aligned(
        &mut canvas, font_medium, model,
        model_x, model_final_y,
        model_size, cfg.color_text_blue, TextAlign::Left
    );

    // -------------------------------------------------------------
    // D. 绘制胶囊参数 (Badges)
    // -------------------------------------------------------------
    let badge_h = (bh * cfg.badge_height_ratio) as u32;
    let badge_w = (badge_h as f32 * cfg.badge_width_ratio) as u32;
    let badge_gap = (badge_w as f32 * cfg.badge_gap) as i32;
    
    // 胶囊描边宽度 (基于原图宽度自适应)
    let badge_stroke = max(4, (src_w as f32 * 0.0030) as u32) as i32;
    let badge_radius = (badge_h / 3) as i32;

    let params = vec![
        (shutter, "S"),
        (iso, "ISO"),
        (focal, "mm"),
        (aperture, "F"),
    ];

    let total_badges_w = (badge_w as i32 * 4) + (badge_gap * 3);
    let mut current_badge_x = center_x - (total_badges_w / 2);
    let badges_y = header_y + model_h as i32 + (bh * cfg.gap_model_params) as i32;

    let val_size = bh * cfg.param_val_scale;
    let lbl_size = bh * cfg.param_lbl_scale;

    for (val, lbl) in params {
        // 1. 绘制外框 (实心圆角矩形 - 灰色)
        let rect_outer = Rect::at(current_badge_x, badges_y).of_size(badge_w, badge_h);
        draw_rounded_rect_polyfill(&mut canvas, rect_outer, badge_radius, cfg.color_border);

        // 2. 绘制内胆 (实心圆角矩形 - 白色) -> 形成镂空效果
        // 内胆半径稍微减小，防止边角穿帮
        let inner_radius = max(0, badge_radius - badge_stroke);
        let rect_inner = Rect::at(
            current_badge_x + badge_stroke, 
            badges_y + badge_stroke
        ).of_size(
            badge_w - (badge_stroke as u32 * 2), 
            badge_h - (badge_stroke as u32 * 2)
        );
        draw_rounded_rect_polyfill(&mut canvas, rect_inner, inner_radius, cfg.bg_color);

        // 3. 绘制数值 (Bold) - 居中
        // 计算数值垂直居中修正
        let (_, val_h) = text_size(PxScale::from(val_size), font_bold, val);
        let val_center_y = badges_y + (badge_h as i32 / 2);
        // 上移一点点，让视觉更平衡
        let val_draw_y = val_center_y - (val_h as i32 / 2) - (val_h as f32 * cfg.val_y_nudge_ratio) as i32;
        
        let badge_center_x = current_badge_x + (badge_w as i32 / 2);
        
        draw_text_aligned(
            &mut canvas, font_bold, val,
            badge_center_x, val_draw_y,
            val_size, cfg.color_text_black, TextAlign::Center
        );

        // 4. 绘制标签 (Medium) - 胶囊下方
        let lbl_y = badges_y + badge_h as i32 + (bh * 0.08) as i32;
        draw_text_aligned(
            &mut canvas, font_medium, lbl,
            badge_center_x, lbl_y,
            lbl_size, cfg.color_text_gray, TextAlign::Center
        );

        current_badge_x += badge_w as i32 + badge_gap;
    }

    Ok(canvas)
}

// 辅助函数：品牌微调
fn get_brand_script_offset(brand: &str) -> f32 {
    let b = brand.trim().to_lowercase();
    match b.as_str() {
        "sony" => 0.05, 
        "fujifilm" | "fuji" => 0.05,
        "olympus" => 0.10,
        _ => 0.0, 
    }
}