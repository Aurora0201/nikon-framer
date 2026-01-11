// src/processor/white/white_polaroid_v2.rs

use image::{DynamicImage, Rgba, imageops, GenericImageView};
use ab_glyph::FontArc;
use log::{info, debug};
use std::time::Instant;
use std::sync::Arc;
use std::cmp::min;

use crate::error::AppError;
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::{FrameProcessor};
use crate::resources::{self, LogoType};

// 引入我们新建的高性能工具箱
use super::utils::{create_expanded_canvas, draw_text_aligned, TextAlign};

// ==========================================
// 1. 结构体定义
// ==========================================

pub struct WhitePolaroidProcessorV2 {
    pub font_data: FontArc,
}

impl FrameProcessor for WhitePolaroidProcessorV2 {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, AppError> {
        let t_start = Instant::now();

        // 1. 准备资源
        // Logo 获取可能会失败，但为了不中断流程，我们允许 Option
        let logo_type = LogoType::Wordmark;
        let logo_img = resources::get_logo(ctx.brand, logo_type);
        
        // 格式化参数字符串
        let params_str = ctx.params.format_standard();

        // 2. 执行核心逻辑
        let result = process_internal(
            img, 
            &self.font_data, 
            &ctx.brand.to_string(),
            &ctx.model_name,
            &params_str,
            logo_img
        )?;

        info!("✨ [PERF] WhitePolaroid V2 processed in {:.2?}", t_start.elapsed());
        Ok(result)
    }
}

// ==========================================
// 2. 布局配置 (可单独提取到 config.rs)
// ==========================================

struct PolaroidConfig {
    side_border_ratio: f32,       // 边框相对于短边的比例
    bottom_height_multiplier: f32,// 底部高度是边框的几倍
    font_scale: f32,             // 字体大小比例
    logo_height_ratio: f32,      // Logo 高度比例
    line_gap_ratio: f32,         // 行间距
    content_vertical_bias: f32,  // 垂直偏移 (0.0 居中)
    
    text_color: Rgba<u8>,
    bg_color: Rgba<u8>,
}

impl Default for PolaroidConfig {
    fn default() -> Self {
        Self {
            side_border_ratio: 0.05,
            bottom_height_multiplier: 4.5,
            
            font_scale: 0.8,
            logo_height_ratio: 1.0,
            
            line_gap_ratio: 0.6,
            content_vertical_bias: 0.0,
            
            text_color: Rgba([20, 20, 20, 255]),
            bg_color: Rgba([255, 255, 255, 255]),
        }
    }
}

// ==========================================
// 3. 核心处理逻辑 (Internal)
// ==========================================

fn process_internal(
    img: &DynamicImage,
    font: &FontArc,
    _brand: &str, // Polaroid 风格通常不强制显示 Brand 文字，除非没 Logo
    _model: &str,
    params: &str,
    logo_opt: Option<Arc<DynamicImage>>,
) -> Result<DynamicImage, AppError> {
    
    let cfg = PolaroidConfig::default();
    let (src_w, src_h) = img.dimensions();

    // -------------------------------------------------------------
    // A. 几何计算 (Metrics)
    // -------------------------------------------------------------
    let base_size = min(src_w, src_h) as f32;
    
    // 计算边距
    let border_size = (base_size * cfg.side_border_ratio).round() as u32;
    // 计算底部留白高度
    let bottom_area_h = (border_size as f32 * cfg.bottom_height_multiplier).round() as u32;

    debug!("📐 [Layout] Polaroid: border={}, bottom={}", border_size, bottom_area_h);

    // -------------------------------------------------------------
    // B. 画布构建 (使用 utils 中的高性能并行算法)
    // -------------------------------------------------------------
    // Polaroid 布局：上下左右都有 border，但底部额外增加 bottom_area_h
    // 即：Top=border, Bottom=bottom_area_h (实际上通常 polaroid 下方留白包含 border)
    // 这里我们按原版逻辑：canvas_h = h + border + bottom_area_h，贴图在 (border, border)
    // 这意味着 Top=border, Bottom=bottom_area_h, Left=border, Right=border
    
    let t_canvas = Instant::now();
    let mut canvas = DynamicImage::ImageRgba8(
        create_expanded_canvas(
            img, 
            border_size, 
            bottom_area_h, // 注意：这里 bottom 传的是额外的底部高度
            border_size, 
            border_size, 
            cfg.bg_color
        )?
    );
    debug!("  -> [PERF] Canvas compose: {:.2?}", t_canvas.elapsed());

    let (canvas_w, canvas_h) = canvas.dimensions();

    // -------------------------------------------------------------
    // C. 底部内容排版
    // -------------------------------------------------------------
    let font_size = border_size as f32 * cfg.font_scale;
    
    // C1. 准备 Logo (缩放)
    let mut scaled_logo = None;
    let mut logo_draw_h = 0;
    
    if let Some(src_logo) = logo_opt {
        let target_h = (border_size as f32 * cfg.logo_height_ratio) as u32;
        // 使用高性能缩放 (Triangle)
        let resized = src_logo.resize(
            src_logo.width(), // 宽度不限，保持比例
            target_h, 
            imageops::FilterType::Triangle
        );
        logo_draw_h = resized.height();
        scaled_logo = Some(resized);
    }

    // C2. 准备文字尺寸
    let has_text = !params.is_empty();
    // 使用 utils 中的 text_size (其实是 imageproc 的，但在 utils 引入了)
    let text_dims = if has_text {
        imageproc::drawing::text_size(
            ab_glyph::PxScale::from(font_size), 
            font, 
            params
        )
    } else {
        (0, 0)
    };
    let text_h = text_dims.1 as u32;

    // C3. 计算垂直堆叠的总高度 (Logo + Gap + Text)
    let gap = if has_text && logo_draw_h > 0 {
        font_size * cfg.line_gap_ratio
    } else {
        0.0
    };

    let total_content_h = logo_draw_h as f32 + gap + text_h as f32;

    // C4. 计算绘制起始 Y 坐标
    // 底部区域的起点 Y
    let footer_start_y = border_size + src_h;
    // 底部区域的中心 Y
    let footer_center_y = footer_start_y as f32 + (bottom_area_h as f32 / 2.0);
    
    // 内容块的起始 Y (居中 + 偏移)
    let start_y = footer_center_y - (total_content_h / 2.0) + (bottom_area_h as f32 * cfg.content_vertical_bias);
    
    let mut cursor_y = start_y as i32;
    let center_x = canvas_w as i32 / 2;

    // -------------------------------------------------------------
    // D. 绘制 (Drawing)
    // -------------------------------------------------------------

    // 1. 绘制 Logo
    if let Some(logo) = scaled_logo {
        let logo_x = center_x - (logo.width() as i32 / 2);
        // imageops::overlay 不需要 Result
        imageops::overlay(&mut canvas, &logo, logo_x as i64, cursor_y as i64);
        
        cursor_y += logo_draw_h as i32 + gap as i32;
    }

    // 2. 绘制文字 (使用 utils::draw_text_aligned)
    if has_text {
        draw_text_aligned(
            &mut canvas,
            font,
            params,
            center_x,
            cursor_y, // 这里 cursor_y 是文字顶部
            font_size,
            cfg.text_color,
            TextAlign::Center // 🟢 极简：直接调用居中绘制
        );
    }

    Ok(canvas)
}