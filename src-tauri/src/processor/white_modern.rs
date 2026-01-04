use image::{DynamicImage, Rgba, GenericImageView, RgbaImage, imageops};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{
    draw_text_mut, 
    draw_filled_rect_mut,
    draw_polygon_mut 
};
use imageproc::point::Point;
use imageproc::rect::Rect;

use std::cmp::max;
use std::f32::consts::PI; 
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*; 

use crate::graphics::shadow::ShadowProfile;
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor;

// ==========================================
// 1. 数据结构定义
// ==========================================

// ==========================================
// 策略 6: 现代白底处理器 (WhiteModern)
// ==========================================
pub struct WhiteModernProcessor {
    pub font_bold: Arc<Vec<u8>>,
    pub font_regular: Arc<Vec<u8>>,
    pub font_medium: Arc<Vec<u8>>, 
    // 🟢 1. 新增手写字体字段
    pub font_script: Arc<Vec<u8>>, 
    
}

impl FrameProcessor for WhiteModernProcessor {
    fn process(&self, img: &DynamicImage, ctx: &ParsedImageContext) -> Result<DynamicImage, String> {
        let bold = FontRef::try_from_slice(&self.font_bold).unwrap();
        let medium = FontRef::try_from_slice(&self.font_medium).unwrap();
        let regular = FontRef::try_from_slice(&self.font_regular).unwrap();
        // 🟢 2. 加载手写字体
        let script = FontRef::try_from_slice(&self.font_script)
             .map_err(|_| "WhiteModern: Birthstone 字体加载失败")?;

        let input = WhiteModernInput {
            brand: ctx.brand.to_string(),
            model: ctx.model_name.clone(),
            iso: ctx.params.iso.map(|v| v.to_string()).unwrap_or_default(),
            aperture: ctx.params.aperture.map(|v| v.to_string()).unwrap_or_default(),
            shutter: ctx.params.shutter_speed.replace("s", "").trim().to_string(),
            focal: ctx.params.focal_length.map(|v| v.to_string()).unwrap_or_default(),
        };
        
        let assets = WhiteModernResources {
            logo: None, // 不再需要 Logo 图片
        };

        // 🟢 3. 传入 script 字体
        Ok(process(img, input, &assets, &bold, &medium, &regular, &script))
    }
}



pub struct WhiteModernInput {
    pub brand: String,
    pub model: String,
    pub iso: String,
    pub aperture: String,
    pub shutter: String,
    pub focal: String,
}

#[allow(dead_code)]
pub struct WhiteModernResources {
    pub logo: Option<DynamicImage>, 
}

// ==========================================
// 2. 布局配置
// ==========================================
struct WhiteModernLayoutConfig {
    border_ratio: f32,       
    bottom_ratio: f32,       
    
    model_text_scale: f32,
    script_scale_ratio: f32, 

    param_val_scale: f32,    
    param_lbl_scale: f32,    
    
    gap_image_model: f32,    
    gap_model_params: f32,   
    
    gap_brand_model: f32,   
    
    script_y_nudge: f32,
    model_y_nudge: f32,
    
    model_x_nudge: f32,
    header_overall_y_nudge: f32,

    val_y_nudge_ratio: f32,

    badge_width_ratio: f32,  
    badge_height_ratio: f32, 
    badge_gap: f32,          
}

impl WhiteModernLayoutConfig {
    fn default() -> Self {
        Self {
            // ===========================================
            // 🟢 用户参数 (保持不变)
            // ===========================================
            border_ratio: 0.05,     
            bottom_ratio: 0.35,     
            
            model_text_scale: 0.20, 
            script_scale_ratio: 1.6,

            param_val_scale: 0.12,  
            param_lbl_scale: 0.095, 
            
            gap_image_model: 0.18,  
            gap_model_params: 0.15, 
            
            gap_brand_model: 0.1,      
            
            header_overall_y_nudge: 0.05,
            
            script_y_nudge: 0.3,
            model_y_nudge: 0.18,
            model_x_nudge: 0.0, 

            val_y_nudge_ratio: 0.28,
            
            badge_width_ratio: 1.8, 
            badge_height_ratio: 0.22,
            badge_gap: 0.40,         
        }
    }
}

// ==========================================
// 3. 辅助函数
// ==========================================

fn get_brand_script_offset(brand: &str) -> f32 {
    let b = brand.trim().to_lowercase();
    match b.as_str() {
        "sony" => 0.05, 
        "fujifilm" | "fuji" => 0.05,
        "olympus" => 0.10,
        _ => 0.0, 
    }
}

/// 🟢 [性能优化] 快速创建白底背景
/// 优化点：
/// 1. 避免了 `flat_map` 导致的每一行都创建一个临时 Vec 的巨大开销。
/// 2. 使用一次性内存分配。
/// 3. 使用 par_chunks_mut 并行填充内存。
fn fast_create_white_background(w: u32, h: u32) -> RgbaImage {
    let len = (w as usize) * (h as usize) * 4;
    // 一次性分配内存，避免碎片
    let mut raw_buffer = vec![0u8; len]; 
    
    // 并行填充白色 (255)
    // 4096 是一个经验值的 chunk size，避免太小的任务切换开销
    raw_buffer.par_chunks_mut(4096).for_each(|chunk| {
        chunk.fill(255);
    });

    RgbaImage::from_raw(w, h, raw_buffer).unwrap()
}
/// 高质量实心圆角矩形绘制
/// 🟢 [性能优化] 预分配 Vec 容量，避免 push 时的扩容
fn draw_rounded_rect_mut_polyfill(canvas: &mut DynamicImage, rect: Rect, radius: i32, color: Rgba<u8>) {
    let x = rect.left() as f32;
    let y = rect.top() as f32;
    let w = rect.width() as f32;
    let h = rect.height() as f32;
    
    let r = (radius as f32).min(w / 2.0).min(h / 2.0);

    if r <= 0.5 {
        draw_filled_rect_mut(canvas, rect, color);
        return;
    }

    let segments_per_corner = 16; 
    // 预分配容量：4个角 * (16段+1起点) 
    // 虽然稍微多一点点，但保证不会重分配
    let mut points: Vec<Point<i32>> = Vec::with_capacity(80); 

    let mut add_arc = |center_x: f32, center_y: f32, start_angle: f32| {
        for i in 0..=segments_per_corner {
            let angle = start_angle + (i as f32 / segments_per_corner as f32) * (PI / 2.0);
            let px = center_x + r * angle.cos();
            let py = center_y + r * angle.sin();
            points.push(Point::new(px.round() as i32, py.round() as i32));
        }
    };

    add_arc(x + w - r, y + r, -PI / 2.0);
    add_arc(x + w - r, y + h - r, 0.0);
    add_arc(x + r, y + h - r, PI / 2.0);
    add_arc(x + r, y + r, PI);

    draw_polygon_mut(canvas, &points, color);
}

fn draw_centered_text_in_rect_fixed(
    canvas: &mut DynamicImage, 
    text: &str, 
    rect: Rect, 
    font: &FontRef, 
    size: f32, 
    color: Rgba<u8>,
    nudge_ratio: f32,
    fixed_height: Option<i32> 
) {
    let scale = PxScale::from(size);
    let (w, h) = imageproc::drawing::text_size(scale, font, text);
    
    let center_x = rect.left() + (rect.width() as i32 / 2);
    let center_y = rect.top() + (rect.height() as i32 / 2);
    
    let draw_x = center_x - (w as i32 / 2);
    let h_ref = fixed_height.unwrap_or(h as i32);

    let nudge_px = (h_ref as f32 * nudge_ratio) as i32;
    let draw_y = center_y - (h_ref / 2) - nudge_px; 
    
    draw_text_mut(canvas, color, draw_x, draw_y, scale, font, text);
}

// ==========================================
// 4. 核心处理逻辑
// ==========================================

pub fn process(
    img: &DynamicImage,
    input: WhiteModernInput,
    _assets: &WhiteModernResources,
    font_bold: &FontRef,    
    font_medium: &FontRef, 
    font_regular: &FontRef, 
    font_script: &FontRef,  
) -> DynamicImage {
    let start_total = Instant::now();
    let cfg = WhiteModernLayoutConfig::default();
    
    let (src_w, src_h) = img.dimensions();
    
    // 竖构图优化逻辑
    let is_portrait = src_h > src_w;
    let portrait_scale = if is_portrait { 0.55 } else { 1.0 }; 

    // 尺寸计算 (应用缩放)
    let border_size = (src_h as f32 * cfg.border_ratio * portrait_scale) as u32;
    let bottom_height = (src_h as f32 * cfg.bottom_ratio * portrait_scale) as u32;
    
    let canvas_w = src_w + border_size * 2;
    let canvas_h = src_h + border_size + bottom_height;
    
    // 1. 背景创建 (已优化)
    let canvas_buffer = fast_create_white_background(canvas_w, canvas_h);
    let mut canvas = DynamicImage::ImageRgba8(canvas_buffer);

    let img_x = border_size as i64;
    let img_y = border_size as i64;
    let img_center_x = img_x + (src_w / 2) as i64;
    let img_center_y = img_y + (src_h / 2) as i64;
    
    ShadowProfile::preset_standard()
        .draw_adaptive_shadow_on(
            canvas.as_mut_rgba8().unwrap(), 
            (src_w, src_h), 
            (img_center_x, img_center_y)
        );

    imageops::overlay(&mut canvas, img, img_x, img_y);
    
    // =========================================
    // 5. Header 排版
    // =========================================
    let bh = bottom_height as f32;
    let center_x = (canvas_w / 2) as i32;
    let content_start_y = (border_size + src_h) as i32;
    
    let base_size = bh * cfg.model_text_scale;
    let script_size = base_size * cfg.script_scale_ratio; 
    let model_size = base_size;

    let script_scale = PxScale::from(script_size);
    let model_scale = PxScale::from(model_size);

    // 🟢 [优化] 移除 clone，直接使用引用
    let brand_text = &input.brand; 
    
    let (brand_w, brand_h) = imageproc::drawing::text_size(script_scale, font_script, brand_text);
    let (model_w, model_h) = imageproc::drawing::text_size(model_scale, font_medium, &input.model);
    
    // 布局计算
    let gap_px = (bh * cfg.gap_brand_model) as i32;
    let model_x_offset_px = (model_size * cfg.model_x_nudge) as i32;
    
    let header_total_w = brand_w as i32 + gap_px + model_w as i32 + model_x_offset_px;
    let start_x = center_x - (header_total_w / 2);
    
    let header_base_y = content_start_y + (bh * cfg.gap_image_model) as i32;
    let overall_y_offset = (bh * cfg.header_overall_y_nudge) as i32;
    let header_y = header_base_y + overall_y_offset;
    
    let header_center_y_line = header_y + (model_h as i32 / 2);

    let color_black = Rgba([20, 20, 20, 255]); 
    let color_pen_blue = Rgba([35, 65, 140, 255]); 

    // --- A. 品牌 (Script) ---
    let brand_fix_ratio = get_brand_script_offset(brand_text);
    let brand_fix_px = (script_size * brand_fix_ratio) as i32;

    let script_draw_x = start_x;
    let script_y_start = header_center_y_line - (brand_h as i32 / 2);
    let script_final_y = script_y_start - (script_size * cfg.script_y_nudge) as i32 + brand_fix_px;
    
    draw_text_mut(&mut canvas, color_pen_blue, script_draw_x, script_final_y, script_scale, font_script, brand_text);

    // --- B. 机型 (Medium) ---
    let model_draw_x = start_x + brand_w as i32 + gap_px + model_x_offset_px;
    let model_final_y = header_y - (model_size * cfg.model_y_nudge) as i32;
    
    draw_text_mut(&mut canvas, color_pen_blue, model_draw_x, model_final_y, model_scale, font_regular, &input.model);

    // =========================================
    // 6. 底部胶囊排版
    // =========================================
    let badge_h = (bh * cfg.badge_height_ratio) as u32;
    let badge_w = (badge_h as f32 * cfg.badge_width_ratio) as u32; 
    let badge_stroke = max(4, (src_w as f32 * 0.0030) as u32);
    let badge_radius = (badge_h / 3) as i32;
    
    let val_size = bh * cfg.param_val_scale;
    let lbl_size = bh * cfg.param_lbl_scale;
    
    let (_, standard_val_h) = imageproc::drawing::text_size(PxScale::from(val_size), font_bold, "0");

    let params = vec![
        (input.shutter, "S"),
        (input.iso, "ISO"),
        (input.focal, "mm"),
        (input.aperture, "F"),
    ];
    
    let gap_badge = (badge_w as f32 * cfg.badge_gap) as i32;
    let total_badges_w = (badge_w as i32 * 4) + (gap_badge * 3);
    let mut current_badge_x = center_x - (total_badges_w / 2);
    
    let badges_y = header_y + model_h as i32 + (bh * cfg.gap_model_params) as i32;
    
    let border_color = Rgba([180, 180, 180, 255]); 
    let bg_color = Rgba([255, 255, 255, 255]);     
    let lbl_color = Rgba([100, 100, 100, 255]);    

    for (val, lbl) in params {
        let rect_outer = Rect::at(current_badge_x, badges_y).of_size(badge_w, badge_h);
        draw_rounded_rect_mut_polyfill(&mut canvas, rect_outer, badge_radius, border_color);
        
        let rect_inner = Rect::at(
            current_badge_x + badge_stroke as i32, 
            badges_y + badge_stroke as i32
        ).of_size(
            badge_w - badge_stroke * 2, 
            badge_h - badge_stroke * 2
        );
        let inner_radius = max(0, badge_radius - badge_stroke as i32);
        draw_rounded_rect_mut_polyfill(&mut canvas, rect_inner, inner_radius, bg_color);
        
        let rect_text = Rect::at(current_badge_x, badges_y).of_size(badge_w, badge_h);
        draw_centered_text_in_rect_fixed(
            &mut canvas, 
            &val, 
            rect_text, 
            font_bold, 
            val_size, 
            color_black,
            cfg.val_y_nudge_ratio,
            Some(standard_val_h as i32)
        );
        
        let lbl_y = badges_y + badge_h as i32 + (bh * 0.08) as i32;
        let (lbl_w, _) = imageproc::drawing::text_size(PxScale::from(lbl_size), font_medium, lbl);
        let lbl_x = current_badge_x + (badge_w as i32 / 2) - (lbl_w as i32 / 2);
        
        draw_text_mut(&mut canvas, lbl_color, lbl_x, lbl_y, PxScale::from(lbl_size), font_medium, lbl);
        
        current_badge_x += badge_w as i32 + gap_badge;
    }

    println!("[PERF] WhiteModern Total: {:?}", start_total.elapsed());
    canvas
}