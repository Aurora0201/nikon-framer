use image::{DynamicImage, GenericImageView, Rgba, imageops};
use ab_glyph::{FontRef, PxScale};
use std::time::Instant;
use std::sync::Arc;

use crate::graphics;
// 引入父模块公共工具
use super::{clean_model_name, resize_image_by_height};

// 🟢 [关键修改] 定义模糊模板所需的资源槽位
// 模糊模式通常不需要 badge_icon (左上角小标)，只需要中间的主副标
pub struct BlurStyleResources {
    // 对应主Logo位置 (如 "Nikon", "Sony")
    pub main_logo: Option<Arc<DynamicImage>>, 
    
    // 对应副Logo位置 (如 "Z", "Alpha")
    pub sub_logo:  Option<Arc<DynamicImage>>, 
}

/// 内部配置结构体：统一管理参数
struct BlurConfig {
    // --- 基础布局 ---
    border_ratio: f32,       // 边框占宽度的比例
    bottom_extra_ratio: f32, // 底部留白高度比例

    // --- 背景与特效 ---
    blur_sigma: f32,         // 模糊强度
    bg_brightness: i32,      // 背景亮度调整
    process_limit: u32,      // 处理时的最大像素限制(优化性能)

    // --- 字体与排版 ---
    font_size_model_ratio: f32,  // 机型文字大小
    font_size_params_ratio: f32, // 参数文字大小
    line_gap_ratio: f32,         // 两行文字的基础间距
    text_block_centering_ratio: f32, // 文字块整体垂直居中比例

    // --- Logo 与 机型文字微调 ---
    logo_main_scale: f32,  // 主Logo大小比例 (原 word)
    logo_sub_scale: f32,   // 副Logo大小比例 (原 z)
    model_text_scale: f32, // 机型文字大小比例
    
    // 机型数字(如"50")的独立垂直偏移比例
    model_text_y_shift_ratio: f32, 
}

impl Default for BlurConfig {
    fn default() -> Self {
        Self {
            border_ratio: 0.08,
            bottom_extra_ratio: 0.6,
            
            blur_sigma: 30.0,
            bg_brightness: -180,
            process_limit: 400,

            font_size_model_ratio: 0.55,
            font_size_params_ratio: 0.32,
            line_gap_ratio: 0.12,
            text_block_centering_ratio: 0.5,

            logo_main_scale: 0.8,
            logo_sub_scale: 0.6,
            model_text_scale: 0.65,

            // 0.10 大约下移 15px (视分辨率而定)
            model_text_y_shift_ratio: 0.10, 
        }
    }
}

pub fn process(
    img: &DynamicImage,
    camera_make: &str,
    camera_model: &str,
    shooting_params: &str,
    font: &FontRef,
    font_weight: &str,
    shadow_intensity: f32,
    assets: &BlurStyleResources // 🟢 接收通用的资源包
) -> DynamicImage {
    // 初始化配置
    let cfg = BlurConfig::default();
    
    let t0 = Instant::now();
    let (width, height) = img.dimensions();

    // 1. 基础尺寸
    let border_size = (width as f32 * cfg.border_ratio) as u32; 
    let bottom_extra = (border_size as f32 * cfg.bottom_extra_ratio) as u32; 
    let canvas_w = width + border_size * 2;
    let canvas_h = height + border_size * 2 + bottom_extra;

    // 2. 模糊背景
    let t_blur = Instant::now();
    let scale_factor_bg = (width.max(height) as f32 / cfg.process_limit as f32).max(1.0);
    let small_w = (canvas_w as f32 / scale_factor_bg) as u32;
    let small_h = (canvas_h as f32 / scale_factor_bg) as u32;
    
    let small_img = img.resize_exact(small_w, small_h, imageops::FilterType::Nearest);
    let mut blurred = small_img.blur(cfg.blur_sigma);
    imageops::colorops::brighten(&mut blurred, cfg.bg_brightness);
    
    let mut canvas = blurred.resize_exact(canvas_w, canvas_h, imageops::FilterType::Triangle).to_rgba8();
    println!("  - [PERF] 高斯模糊背景生成: {:.2?}", t_blur.elapsed());

    // 3. 玻璃与阴影
    let t_shadow = Instant::now();
    let glass_img = graphics::apply_rounded_glass_effect(img);
    let shadow_img = graphics::create_diffuse_shadow(glass_img.width(), glass_img.height(), border_size, shadow_intensity);
    
    let target_center_x = (border_size as i64) + (width as i64 / 2);
    let offset_y = (border_size as f32 * 0.3) as i64;
    let target_center_y = (border_size as i64) + (height as i64 / 2) + offset_y;
    
    let draw_x = target_center_x - (shadow_img.width() as i64 / 2);
    let draw_y = target_center_y - (shadow_img.height() as i64 / 2);
    imageops::overlay(&mut canvas, &shadow_img, draw_x as i64, draw_y as i64);

    let border_thickness = (glass_img.width() - width) / 2;
    let overlay_x = border_size as i64 - border_thickness as i64;
    let overlay_y = border_size as i64 - border_thickness as i64;
    imageops::overlay(&mut canvas, &glass_img, overlay_x, overlay_y);
    println!("  - [PERF] 阴影与玻璃特效合成: {:.2?}", t_shadow.elapsed());

    // 4. 文字布局计算
    let text_color = Rgba([255, 255, 255, 255]); 
    let sub_text_color = Rgba([200, 200, 200, 255]); 
    
    let font_size_model = border_size as f32 * cfg.font_size_model_ratio; 
    let font_size_params = border_size as f32 * cfg.font_size_params_ratio; 
    let scale_params = PxScale::from(font_size_params);
    
    let text_area_start_y = (border_size + height) as f32;
    let text_area_total_h = (border_size + bottom_extra) as f32;
    let line_gap = font_size_model * cfg.line_gap_ratio; 
    
    let text_block_h = font_size_model + line_gap + font_size_params;
    let padding_top = (text_area_total_h - text_block_h) * cfg.text_block_centering_ratio;
    
    let line1_y = (text_area_start_y + padding_top).round() as i32;
    let line2_y = (text_area_start_y + padding_top + font_size_model + line_gap).round() as i32;

    // 5. 绘制第一行：Logo + 机型文字
    if !camera_model.is_empty() {
        let base_h = font_size_model * 1.2; 

        // 使用配置参数
        let h_main = (base_h * cfg.logo_main_scale) as u32;
        let h_sub  = (base_h * cfg.logo_sub_scale) as u32;
        let s_text = base_h * cfg.model_text_scale;

        let spacing = (font_size_model * 0.3) as u32; 
        let mut total_w = 0;

        // --- A. 预处理资源 (转白 + 缩放) ---
        // 🟢 处理主Logo
        let scaled_main = if let Some(logo) = &assets.main_logo {
            let white_img = graphics::make_image_white(logo); // Arc 自动解引用
            let s = resize_image_by_height(&white_img, h_main);
            total_w += s.width() + spacing;
            Some(s)
        } else { None };

        // 🟢 处理副Logo
        let scaled_sub = if let Some(logo) = &assets.sub_logo {
            let white_img = graphics::make_image_white(logo);
            let s = resize_image_by_height(&white_img, h_sub);
            total_w += s.width() + spacing;
            Some(s)
        } else { None };

        // 🟢 处理文字
        let model_str = clean_model_name(camera_make, camera_model);
        let text_img = if !model_str.is_empty() {
            let img = graphics::generate_skewed_text_high_quality(
                &model_str, font, PxScale::from(s_text), text_color, 0.23
            );
            total_w += img.width();
            Some(img)
        } else { None };

        // --- B. 绘制元素 ---
        let mut current_x = (canvas_w as i32 - total_w as i32) / 2;
        let row_center_y = line1_y + (font_size_model as i32 / 2);

        // 1. 绘制 Main Logo
        if let Some(img) = scaled_main {
            let y = row_center_y - (img.height() as i32 / 2);
            imageops::overlay(&mut canvas, &img, current_x as i64, y as i64);
            current_x += img.width() as i32 + spacing as i32;
        }

        // 2. 绘制 Sub Logo
        let mut sub_bottom_y = 0;
        if let Some(img) = scaled_sub {
            let y = row_center_y - (img.height() as i32 / 2);
            imageops::overlay(&mut canvas, &img, current_x as i64, y as i64);
            sub_bottom_y = y + img.height() as i32;
            current_x += img.width() as i32 + spacing as i32;
        }

        // 3. 绘制机型文字
        if let Some(img) = text_img {
            // 计算基础 Y 坐标 (如果有副Logo，则与副Logo底部对齐；否则垂直居中)
            let base_y = if sub_bottom_y > 0 {
                sub_bottom_y - img.height() as i32
            } else {
                row_center_y - (img.height() as i32 / 2)
            };

            // 应用额外的垂直偏移
            let extra_offset = (border_size as f32 * cfg.model_text_y_shift_ratio) as i32;
            let final_y = base_y + extra_offset;

            // 微调 X 坐标 (减少与 Logo 的间距)
            let x = current_x - 10; 
            imageops::overlay(&mut canvas, &img, x as i64, final_y as i64);
        }
    }

    // 6. 绘制第二行：拍摄参数
    if !shooting_params.is_empty() {
        let text_w = graphics::measure_text_width(font, shooting_params, scale_params);
        let text_x = ((canvas_w as i32 - text_w as i32) / 2).max(0);
        let sub_weight = if font_weight == "ExtraBold" { "Bold" } else { font_weight };
        graphics::draw_text_high_quality(&mut canvas, sub_text_color, text_x, line2_y, scale_params, font, shooting_params, sub_weight);
    }

    println!("  - [PERF] 高斯模糊模式-绘制阶段总耗时: {:.2?}", t0.elapsed());
    DynamicImage::ImageRgba8(canvas)
}