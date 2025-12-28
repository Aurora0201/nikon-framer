use image::{DynamicImage, Rgba, RgbaImage, imageops, GenericImageView}; // 必须引入 GenericImageView 才能用 .dimensions()
use ab_glyph::{FontRef, PxScale};
use std::sync::Arc;
use std::time::Instant;

// 假设 graphics 模块包含基础绘图能力 (如 draw_text_high_quality)
// 如果 graphics 也不想依赖，可以将绘图逻辑也搬过来，但通常保留 shared graphics 是合理的
use crate::graphics; 

/// ----------------------------------------------------------------------------
/// 1. 专属资源定义 (解耦，不依赖 BlurStyleResources)
/// ----------------------------------------------------------------------------
/// 专门用于 Polaroid 模式的资源容器
pub struct PolaroidResources {
    /// 厂商 Logo (通常是黑色的 Nikon/Sony 等)
    pub logo: Option<Arc<DynamicImage>>,
}

/// ----------------------------------------------------------------------------
/// 2. 布局配置结构体
/// ----------------------------------------------------------------------------
pub struct PolaroidConfig {
    pub side_border_ratio: f32,      // 侧边框比例
    pub bottom_height_multiplier: f32, // 底部留白倍数
    pub font_size_ratio: f32,        // 字体大小比例
    pub logo_height_ratio: f32,      // Logo高度比例
    pub line_gap_ratio: f32,         // 行间距
    pub content_vertical_bias: f32,  // 垂直偏移修正
}

impl Default for PolaroidConfig {
    fn default() -> Self {
        Self {
            side_border_ratio: 0.05,
            bottom_height_multiplier: 4.5,

            font_size_ratio: 0.8,
            logo_height_ratio: 1.0,

            line_gap_ratio: 0.5,
            content_vertical_bias: 0.0,
        }
    }
}

/// ----------------------------------------------------------------------------
/// 3. 核心处理函数
/// ----------------------------------------------------------------------------
pub fn process_polaroid_style(
    img: &DynamicImage,
    _camera_make: &str, 
    _camera_model: &str,
    shooting_params: &str,
    font: &FontRef,
    font_weight: &str,
    assets: &PolaroidResources, // 使用专属结构体
) -> DynamicImage {
    let cfg = PolaroidConfig::default();
    let t0 = Instant::now();

    // 修复报错关键点：引入 GenericImageView 后，这里就能正常获取 dimensions 了
    let (width, height) = img.dimensions();

    // -------------------------------------------------------------
    // A. 计算几何尺寸
    // -------------------------------------------------------------
    // 视觉一致性核心：使用短边作为基准
    let base_size = width.min(height) as f32;

    let border_size = (base_size * cfg.side_border_ratio).round() as u32;
    let bottom_area_h = (border_size as f32 * cfg.bottom_height_multiplier).round() as u32;

    let canvas_w = width + border_size * 2;
    let canvas_h = height + border_size + bottom_area_h;

    // -------------------------------------------------------------
    // B. 创建画布并合成
    // -------------------------------------------------------------
    let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, Rgba([255, 255, 255, 255]));
    imageops::overlay(&mut canvas, img, border_size as i64, border_size as i64);

    // -------------------------------------------------------------
    // C. 底部排版
    // -------------------------------------------------------------
    let footer_start_y = border_size + height;
    let footer_h = bottom_area_h;

    let font_size = border_size as f32 * cfg.font_size_ratio;
    let font_scale = PxScale::from(font_size);
    let text_color = Rgba([0, 0, 0, 255]); 
    let sub_weight = if font_weight == "ExtraBold" { "Bold" } else { font_weight };

    // --- C1. 准备 Logo ---
    let logo_target_h = (border_size as f32 * cfg.logo_height_ratio) as u32;
    let mut logo_sprite = None;

    if let Some(src_logo) = &assets.logo {
        // 使用本文件下方的私有辅助函数，不再依赖外部 graphics 里的 resize
        logo_sprite = Some(resize_by_height(src_logo, logo_target_h));
    }

    // --- C2. 计算垂直布局堆叠高度 ---
    let mut content_block_h = 0.0;
    let gap = font_size * cfg.line_gap_ratio;
    let has_text = !shooting_params.is_empty();

    if let Some(l) = &logo_sprite {
        content_block_h += l.height() as f32;
    }
    if logo_sprite.is_some() && has_text {
        content_block_h += gap;
    }
    if has_text {
        content_block_h += font_size; // 估算高度
    }

    // --- C3. 确定绘制起始 Y ---
    let footer_center_y = footer_start_y as f32 + (footer_h as f32 / 2.0);
    let bias_pixel = footer_h as f32 * cfg.content_vertical_bias;
    let mut current_draw_y = (footer_center_y - (content_block_h / 2.0) + bias_pixel).round() as i64;
    let canvas_center_x = canvas_w as i64 / 2;

    // --- D. 绘制 ---
    
    // 绘制 Logo
    if let Some(logo) = logo_sprite {
        let logo_x = canvas_center_x - (logo.width() as i64 / 2);
        imageops::overlay(&mut canvas, &logo, logo_x, current_draw_y);
        current_draw_y += logo.height() as i64 + gap as i64;
    }

    // 绘制文字
    if has_text {
        // 假设 graphics 模块里有 measure_text_width 和 draw_text_high_quality
        // 这两个是通用基础功能，通常建议保留在 graphics 模块中
        let text_width = graphics::measure_text_width(font, shooting_params, font_scale);
        let text_x = (canvas_w as i32 - text_width as i32) / 2;
        
        graphics::draw_text_high_quality(
            &mut canvas,
            text_color,
            text_x,
            current_draw_y as i32,
            font_scale,
            font,
            shooting_params,
            sub_weight
        );
    }

    println!("  - [PERF] PolaroidWhite 模式生成耗时: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}

/// ----------------------------------------------------------------------------
/// 私有辅助函数：按高度缩放图片
/// ----------------------------------------------------------------------------
fn resize_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    // 使用 Triangle 滤镜，速度和质量的平衡
    // 传入 u32::MAX 作为宽度，image 库会自动保持纵横比
    img.resize(u32::MAX, target_height, imageops::FilterType::Triangle)
}