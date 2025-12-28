use image::{DynamicImage, ImageBuffer, Rgba, imageops, GenericImageView};
use ab_glyph::{FontRef, PxScale};
use std::time::Instant;
use std::sync::Arc;

use crate::graphics;
// 引入父模块公共工具
use super::{DrawContext, clean_model_name, resize_image_by_height};

// 🟢 [关键修改] 定义白底模板所需的资源槽位
// 模板只关心"位置"，不关心"内容"
pub struct WhiteStyleResources {
    // 对应主Logo位置 (如 "Nikon", "Sony")
    pub main_logo: Option<Arc<DynamicImage>>, 
    
    // 对应副Logo位置 (如 "Z", "Alpha")
    pub sub_logo:  Option<Arc<DynamicImage>>, 
    
    // 对应左侧装饰图标位置 (如 "Yellow Box", "Red Dot")
    pub badge_icon: Option<Arc<DynamicImage>>, 
}

/// 布局配置：集中管理所有"魔数"
struct LayoutConfig {
    bottom_ratio: f32,      // 底部白条高度占原图高度的比例
    
    scale_model_text: f32,  // 机型文字大小
    scale_params_text: f32, // 参数文字大小
    scale_logo_main: f32,   // 主Logo大小 (原 word)
    scale_logo_sub: f32,    // 副Logo大小 (原 z)
    
    gap_icon_text: f32,     // 左侧图标和文字的间距
    margin_left: f32,       // 左边距
    line_gap: f32,          // 两行文字之间的间距
    
    skew_padding_fix: i32,  // 斜体文字的左侧修正
    
    // 机型文字(如"50")的垂直偏移比例
    model_text_y_offset_ratio: f32, 
}

impl LayoutConfig {
    fn default_config() -> Self {
        Self {
            bottom_ratio: 0.14,
            
            scale_model_text: 0.95,
            scale_params_text: 0.22,
            scale_logo_main: 1.15, // 原 word scale
            scale_logo_sub: 0.9,   // 原 z scale
            
            gap_icon_text: 0.25,
            margin_left: 0.4,
            line_gap: 0.1,
            skew_padding_fix: -10,
            
            // 0.25 表示向下微调，使底部视觉更平衡
            model_text_y_offset_ratio: 0.25, 
        }
    }
}

struct LayoutMetrics {
    bottom_height: u32,
    base_h: f32,
    margin_left: i32,
    gap_icon_text: i32,
    line_gap: i32,
    bar_center_y: i32,
    line1_height: f32,
    line1_y: i32,
}

fn calculate_metrics(img_height: u32, config: &LayoutConfig) -> LayoutMetrics {
    let bottom_height = (img_height as f32 * config.bottom_ratio) as u32;
    // base_h 是计算文字大小的基准单位
    let base_h = bottom_height as f32 * 0.25; 

    // 使用主Logo的比例来定行高
    let line1_height = base_h * config.scale_logo_main;
    let font_size_params = bottom_height as f32 * config.scale_params_text;
    let line_gap = (bottom_height as f32 * config.line_gap) as i32;
    let total_block_h = line1_height + line_gap as f32 + font_size_params;
    
    // 文字块整体垂直居中于白条
    let bar_center_y = img_height as f32 + bottom_height as f32 / 2.0;
    let text_block_start_y = bar_center_y - (total_block_h / 2.0);

    LayoutMetrics {
        bottom_height,
        base_h,
        margin_left: (bottom_height as f32 * config.margin_left) as i32,
        gap_icon_text: (bottom_height as f32 * config.gap_icon_text) as i32,
        line_gap,
        bar_center_y: bar_center_y as i32,
        line1_height,
        line1_y: text_block_start_y.round() as i32,
    }
}

// 绘图逻辑：左侧装饰图标 (Badge Icon)
fn draw_left_icon(ctx: &mut DrawContext, icon: &DynamicImage, metrics: &LayoutMetrics) -> i32 {
    let max_h = (metrics.bottom_height as f32 * 0.65) as u32;
    let scaled_icon = resize_image_by_height(icon, max_h);
    // 垂直居中于白条区域
    let icon_y = metrics.bar_center_y - (scaled_icon.height() as i32 / 2);
    let icon_x = metrics.margin_left;
    imageops::overlay(ctx.canvas, &scaled_icon, icon_x as i64, icon_y as i64);
    
    icon_x + scaled_icon.width() as i32 + metrics.gap_icon_text
}

// 绘图逻辑：主行 (Main Logo + Sub Logo + Model Text)
fn draw_main_line_elements(
    ctx: &mut DrawContext,
    start_x: i32,
    assets: &WhiteStyleResources, // 🟢 改为接收通用资源包
    camera_make: &str,
    camera_model: &str,
    metrics: &LayoutMetrics,
    config: &LayoutConfig
) {
    let mut current_x = start_x;
    let line1_y = metrics.line1_y;

    // 1. 绘制主Logo (Main Logo / Wordmark)
    if let Some(main_img) = &assets.main_logo {
        let target_h = (metrics.base_h as f32 * config.scale_logo_main) as u32;
        // 注意：main_img 是 Arc<DynamicImage>，可以直接解引用传给需要 &DynamicImage 的函数
        let scaled_word = resize_image_by_height(main_img, target_h);
        
        // 垂直居中于第一行高度内
        let word_y = line1_y + ((metrics.line1_height as i32 - scaled_word.height() as i32) / 2);
        imageops::overlay(ctx.canvas, &scaled_word, current_x as i64, word_y as i64);
        current_x += scaled_word.width() as i32 + (metrics.base_h as f32 * 0.3) as i32;
    }

    // 2. 绘制副Logo (Sub Logo / Series Symbol)
    let mut sub_bottom_y = line1_y + metrics.line1_height as i32; 
    if let Some(sub_img) = &assets.sub_logo {
        let target_h = (metrics.base_h as f32 * config.scale_logo_sub) as u32;
        let scaled_sub = resize_image_by_height(sub_img, target_h);
        
        let sub_y = line1_y + ((metrics.line1_height as i32 - scaled_sub.height() as i32) / 2);
        imageops::overlay(ctx.canvas, &scaled_sub, current_x as i64, sub_y as i64);
        
        // 记录副Logo的底部位置，作为后续对齐基准
        sub_bottom_y = sub_y + scaled_sub.height() as i32;
        current_x += scaled_sub.width() as i32 + (metrics.base_h as f32 * 0.15) as i32;
    }

    // 3. 绘制机型文字 (Model Number)
    if !camera_model.is_empty() {
        let model_text = clean_model_name(camera_make, camera_model);
        let text_size = metrics.base_h as f32 * config.scale_model_text;
        
        // 生成斜体文字 (黑色)
        let italic_img = graphics::generate_skewed_text_high_quality(
            &model_text, ctx.font, PxScale::from(text_size), Rgba([0, 0, 0, 255]), 0.23
        );

        // 计算基础位置：
        // 如果有副Logo，则与副Logo底部对齐；否则与主Logo(第一行)垂直居中
        let align_bottom_y = if assets.sub_logo.is_some() {
            sub_bottom_y - italic_img.height() as i32
        } else {
            // 如果没有副Logo，回退到垂直居中逻辑 (比如 Canon 只有主标)
            let row_center = line1_y + (metrics.line1_height as i32 / 2);
            row_center + (italic_img.height() as i32 / 2) // 粗略估算底部
        };
        
        // 应用垂直偏移
        let offset = (metrics.base_h * config.model_text_y_offset_ratio) as i32;
        
        let draw_y = align_bottom_y + offset;
        let draw_x = current_x + config.skew_padding_fix;
        
        imageops::overlay(ctx.canvas, &italic_img, draw_x as i64, draw_y as i64);
    }
}

fn draw_params_line(ctx: &mut DrawContext, start_x: i32, params: &str, metrics: &LayoutMetrics, config: &LayoutConfig) {
    if params.is_empty() { return; }
    let line2_y = metrics.line1_y + metrics.line1_height as i32 + metrics.line_gap;
    let sub_weight = if ctx.font_weight == "ExtraBold" { "Bold" } else { ctx.font_weight };
    let font_size = metrics.bottom_height as f32 * config.scale_params_text;
    
    // 参数行文字颜色 (灰色)
    graphics::draw_text_high_quality(
        ctx.canvas, Rgba([100, 100, 100, 255]), start_x, line2_y, 
        PxScale::from(font_size), ctx.font, params, sub_weight
    );
}

// =========================================================
// 🟢 主处理函数
// =========================================================
pub fn process(
    img: &DynamicImage,
    camera_make: &str,
    camera_model: &str,
    shooting_params: &str,
    font: &FontRef,
    font_weight: &str,
    assets: &WhiteStyleResources // 🟢 接收通用的资源包
) -> DynamicImage {
    let t0 = Instant::now();
    let (width, height) = img.dimensions();
    
    let config = LayoutConfig::default_config();
    let metrics = calculate_metrics(height, &config);
    let new_height = height + metrics.bottom_height;
    
    // 1. 创建白底画布
    let mut canvas = ImageBuffer::from_pixel(width, new_height, Rgba([255, 255, 255, 255]));
    
    // 2. 贴入原图
    imageops::overlay(&mut canvas, img, 0, 0);

    // 构造绘图上下文
    let mut ctx = DrawContext { canvas: &mut canvas, font, font_weight };

    // 3. 绘制底部信息
    let mut content_start_x = metrics.margin_left;
    
    // 🟢 如果有装饰图标 (Badge Icon)，先画它
    if let Some(icon) = &assets.badge_icon {
        content_start_x = draw_left_icon(&mut ctx, icon, &metrics);
    }

    // 🟢 绘制主行 (传入通用资源包)
    draw_main_line_elements(&mut ctx, content_start_x, assets, camera_make, camera_model, &metrics, &config);
    
    // 绘制参数行
    draw_params_line(&mut ctx, content_start_x, shooting_params, &metrics, &config);

    println!("  - [PERF] 白底模式-绘制阶段总耗时: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}