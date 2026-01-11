// src/processor/white/white_classic_v2.rs

use image::{DynamicImage, Rgba, imageops, GenericImageView};
use imageproc::drawing::{draw_filled_rect_mut, text_size};
use imageproc::rect::Rect;
use ab_glyph::{FontArc, PxScale};
use log::{info, debug};
use std::time::Instant;
use std::cmp::min;

use crate::error::AppError;
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor;
use crate::resources::{self, LogoType};

// 引入高性能工具箱
use super::utils::{create_expanded_canvas, draw_text_aligned, TextAlign};

// ==========================================
// 1. 结构体定义
// ==========================================

pub struct WhiteClassicProcessorV2 {
    pub font_data: FontArc,
}

impl FrameProcessor for WhiteClassicProcessorV2 {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, AppError> {
        let t_start = Instant::now();

        // 1. 准备资源
        // Classic 风格使用的是 Wordmark (文字标)
        let logo_type = LogoType::Wordmark;
        let logo_img = resources::get_logo(ctx.brand, logo_type);
        
        // 格式化文本
        let model_text = format!("{} {}", ctx.brand, ctx.model_name).to_uppercase();
        let params_text = ctx.params.format_standard();

        // 2. 执行核心逻辑
        let result = process_internal(
            img, 
            &self.font_data, 
            &model_text,
            &params_text,
            logo_img
        )?;

        info!("✨ [PERF] WhiteClassic V2 processed in {:.2?}", t_start.elapsed());
        Ok(result)
    }
}

// ==========================================
// 2. 布局配置
// ==========================================

struct ClassicConfig {
    // 基础比例
    bar_ratio_land: f32,    // 横构图底栏高度比例
    bar_ratio_port: f32,    // 竖构图底栏高度比例
    
    // 边距与间距
    padding_ratio_land: f32,
    padding_ratio_port: f32,
    element_gap_ratio: f32, // 元素间距 (Logo - Line - Text)
    text_gap_ratio_port: f32, // 新增
    
    // 字体缩放
    font_scale_main_land: f32,
    font_scale_sub_land: f32,
    font_scale_main_port: f32,
    font_scale_sub_port: f32,

    // 图标与线条
    icon_scale_land: f32,
    icon_scale_port: f32,
    line_width_ratio: f32,
    line_height_scale: f32, // 线条相对于文字高度的比例
    
    // 颜色
    color_text_main: Rgba<u8>,
    color_text_sub: Rgba<u8>,
    color_line: Rgba<u8>,
    bg_color: Rgba<u8>,
}

impl Default for ClassicConfig {
    fn default() -> Self {
        Self {
            bar_ratio_land: 0.12,
            bar_ratio_port: 0.13,
            
            padding_ratio_land: 0.5,
            padding_ratio_port: 0.35,
            element_gap_ratio: 0.30,
            
            text_gap_ratio_port: 0.06, // 新增
            // 横构图字体
            font_scale_main_land: 0.38,
            font_scale_sub_land: 0.31,
            
            // 竖构图字体 (稍小，因为堆叠)
            font_scale_main_port: 0.30,
            font_scale_sub_port: 0.25,
            
            icon_scale_land: 0.35,
            icon_scale_port: 0.38,
            
            line_width_ratio: 0.025,
            line_height_scale: 1.5, // 竖线比文字略高
            
            color_text_main: Rgba([0, 0, 0, 255]),      // 纯黑
            color_text_sub: Rgba([60, 60, 60, 255]),    // 深灰
            color_line: Rgba([160, 160, 160, 255]),     // 浅灰线条
            bg_color: Rgba([255, 255, 255, 255]),       // 纯白背景
        }
    }
}

// ==========================================
// 3. 核心处理逻辑
// ==========================================

fn process_internal(
    img: &DynamicImage,
    font: &FontArc,
    model_text: &str,
    params_text: &str,
    logo_opt: Option<std::sync::Arc<DynamicImage>>,
) -> Result<DynamicImage, AppError> {
    
    let cfg = ClassicConfig::default();
    let (src_w, src_h) = img.dimensions();
    let is_landscape = src_w >= src_h;

    // A. 尺寸计算
    let short_edge = min(src_w, src_h) as f32;
    let ratio = if is_landscape { cfg.bar_ratio_land } else { cfg.bar_ratio_port };
    let bar_height = (short_edge * ratio).round() as u32;

    debug!("📐 [Layout] Classic: {}x{}, Bar={}", src_w, src_h, bar_height);

    // B. 画布构建
    let t_canvas = Instant::now();
    let mut canvas = DynamicImage::ImageRgba8(
        create_expanded_canvas(img, 0, bar_height, 0, 0, cfg.bg_color)?
    );
    debug!("  -> [PERF] Canvas compose: {:.2?}", t_canvas.elapsed());

    let (canvas_w, _canvas_h) = canvas.dimensions();
    
    // C. 绘制内容
    let bh = bar_height as f32;
    let center_y = (src_h + bar_height / 2) as i32;
    let gap = (bh * cfg.element_gap_ratio) as i32;
    let line_w = (bh * cfg.line_width_ratio).max(1.0) as u32;

    if is_landscape {
        // ===========================================
        // 🟢 布局 1: 横构图 (左右分栏)
        // Left: Model Name
        // Right: Logo | Line | Params (整体右对齐)
        // ===========================================
        
        let padding_x = (bh * cfg.padding_ratio_land) as i32;
        
        // 1. 左侧：机型名称 (保持不变)
        let main_size = bh * cfg.font_scale_main_land;
        draw_text_aligned(
            &mut canvas, font, model_text,
            padding_x, center_y - (main_size as i32 / 2),
            main_size, cfg.color_text_main, TextAlign::Left
        );

        // 2. 右侧：从右向左绘制 (Params -> Line -> Logo)
        // 这样视觉上就是 (Logo | Line | Params) 靠右对齐
        let mut cursor_x = (canvas_w as i32) - padding_x;
        let icon_h = (bh * cfg.icon_scale_land) as u32;

        // A. 参数 (最右侧)
        if !params_text.is_empty() {
            let sub_size = bh * cfg.font_scale_sub_land;
            // 使用右对齐绘制
            draw_text_aligned(
                &mut canvas, font, params_text,
                cursor_x, center_y - (sub_size as i32 / 2),
                sub_size, cfg.color_text_sub, TextAlign::Right
            );
            // 🟢 修复：需要测量文字宽度，以便向左移动光标给线和Logo留位置
            let (text_w, _) = text_size(PxScale::from(sub_size), font, params_text);
            cursor_x -= text_w as i32 + gap;
        }

        // B. 竖线 (中间)
        if logo_opt.is_some() && !params_text.is_empty() {
            let line_h = (icon_h as f32 * 1.5) as u32;
            let line_y = center_y - (line_h as i32 / 2);
            // 线条画在当前光标的左侧
            let rect = Rect::at(cursor_x - line_w as i32, line_y).of_size(line_w, line_h);
            draw_filled_rect_mut(&mut canvas, rect, cfg.color_line);
            
            cursor_x -= line_w as i32 + gap;
        }

        // C. Logo (最左侧)
        if let Some(logo) = &logo_opt {
            // 🔴 修改前：使用了 logo.width() 作为限制，这可能会导致大图被限制宽度而达不到目标高度
            // let resized = logo.resize(logo.width(), icon_h, imageops::FilterType::Triangle);

            // 🟢 修改后：使用 u32::MAX 作为宽度限制，强制高度统一为 icon_h
            // 宽度会根据比例自动调整
            let resized = logo.resize(u32::MAX, icon_h, imageops::FilterType::Triangle);
            
            let logo_w = resized.width() as i32;
            let logo_y = center_y - (resized.height() as i32 / 2);
            
            // Logo 的右边缘是当前的 cursor_x，所以左边缘是 cursor_x - logo_w
            imageops::overlay(&mut canvas, &resized, (cursor_x - logo_w) as i64, logo_y as i64);
        }

    } else {
        // ===========================================
        // 🟢 布局 2: 竖构图 (保持不变)
        // ===========================================
        let padding_x = (bh * cfg.padding_ratio_port) as i32;
        let mut cursor_x = padding_x;
        let icon_h = (bh * cfg.icon_scale_port) as u32;
        
        // A. Logo
        if let Some(logo) = &logo_opt {
            // 🔴 修改前
            // let resized = logo.resize(logo.width(), icon_h, imageops::FilterType::Triangle);

            // 🟢 修改后：同样使用 u32::MAX 强制固定高度
            let resized = logo.resize(u32::MAX, icon_h, imageops::FilterType::Triangle);

            let logo_y = center_y - (resized.height() as i32 / 2);
            imageops::overlay(&mut canvas, &resized, cursor_x as i64, logo_y as i64);
            cursor_x += resized.width() as i32 + gap;
        }

        // B. 竖线
        if logo_opt.is_some() {
            let line_h = (icon_h as f32 * cfg.line_height_scale) as u32;
            let line_y = center_y - (line_h as i32 / 2);
            let rect = Rect::at(cursor_x, line_y).of_size(line_w, line_h);
            draw_filled_rect_mut(&mut canvas, rect, cfg.color_line);
            cursor_x += line_w as i32 + gap;
        }

        // C. 文字堆叠
        let main_size = bh * cfg.font_scale_main_port;
        let sub_size = bh * cfg.font_scale_sub_port;
        let text_gap = (bh * cfg.text_gap_ratio_port) as i32;
        let main_y = center_y - (text_gap / 2) - (main_size as i32);
        let sub_y = center_y + (text_gap / 2);

        draw_text_aligned(&mut canvas, font, model_text, cursor_x, main_y, main_size, cfg.color_text_main, TextAlign::Left);
        draw_text_aligned(&mut canvas, font, params_text, cursor_x, sub_y, sub_size, cfg.color_text_sub, TextAlign::Left);
    }

    Ok(canvas)
}