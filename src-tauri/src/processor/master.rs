// src-tauri/src/processor/master.rs

use image::{DynamicImage, Rgba, GenericImageView, imageops};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{draw_text_mut, draw_line_segment_mut};
use std::time::Instant; // 🟢 [新增] 引入计时器

// 布局配置中心 (保持之前的逻辑不变)
struct MasterLayoutConfig {
    border_ratio: f32,
    bottom_ratio: f32,
    column_gap_ratio: f32,
    label_bottom_margin: f32,
    row_gap: f32,
    text_scale_val: f32,
    text_scale_lbl: f32,
    separator_scale: f32,
    separator_opacity: u8,
    header_bottom_margin: f32, 
    header_script_size: f32,   
    header_small_size: f32,    
    header_gap_top: f32,       
    header_gap_bottom: f32,    
    bg_blur_radius: f32,
}

impl MasterLayoutConfig {
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
            separator_opacity: 40, 
            header_bottom_margin: 0.3,
            header_script_size: 0.12,
            header_small_size: 0.05,
            header_gap_top: -0.02,
            header_gap_bottom: 0.1,
            bg_blur_radius: 150.0,
        }
    }
}

pub fn process(
    img: &DynamicImage,
    params: &str,
    main_font: &FontRef,   
    script_font: &FontRef, 
    serif_font: &FontRef,  
) -> DynamicImage {
    // 🟢 [DEBUG] 开始计时
    let start_total = Instant::now();
    
    let cfg = MasterLayoutConfig::default();
    println!("--------------------------------------------------");
    println!("[DEBUG] Master Process Start. Params: '{}'", params);

    let (img_w, img_h) = img.dimensions();
    let is_portrait = img_h > img_w;

    // 1. 计算尺寸
    let border_size = (img_h as f32 * cfg.border_ratio) as u32;
    let bottom_height = (img_h as f32 * cfg.bottom_ratio) as u32;
    let canvas_w = img_w + (border_size * 2);
    let canvas_h = img_h + border_size + bottom_height;

    // 🟢 [DEBUG] 阶段计时：背景生成
    let start_bg = Instant::now();
    
    // 3. 生成背景 (使用优化后的高性能算法)
    let mut canvas = create_aspect_fill_bg_optimized(img, canvas_w, canvas_h, cfg.bg_blur_radius);
    canvas = canvas.brighten(-15); 
    
    println!("[PERF] 背景生成耗时: {:?}", start_bg.elapsed());

    // 🟢 [DEBUG] 阶段计时：叠加与排版
    let start_overlay = Instant::now();

    // 4. 贴入原图
    imageops::overlay(&mut canvas, img, border_size as i64, border_size as i64);

    // 5. 解析 & 清洗参数
    let (iso_raw, aperture_raw, shutter_raw, focal_raw) = parse_params_smart(params);
    let iso_val = clean_param(&iso_raw, "ISO");
    let aperture_val = clean_param(&aperture_raw, "f/");
    let focal_val = clean_param(&focal_raw, "mm");
    let shutter_val = clean_param(&shutter_raw, "s");

    // 6. 排版计算 (保持之前修正后的逻辑：仅缩小下方两行)
    let bh = bottom_height as f32;
    let center_x = canvas_w as i32 / 2;
    
    // 仅针对参数行的缩放系数 (竖构图缩小)
    let param_scale = if is_portrait { 0.6 } else { 1.0 };

    // --- A. 参数区 ---
    let val_size = bh * cfg.text_scale_val * param_scale;
    let lbl_size = bh * cfg.text_scale_lbl * param_scale;
    let margin_bottom = bh * cfg.label_bottom_margin;
    let row_gap = if is_portrait { bh * cfg.row_gap * 0.5 } else { bh * cfg.row_gap };

    let label_draw_y = (canvas_h as f32 - margin_bottom - lbl_size) as i32;
    let value_draw_y = (label_draw_y as f32 - row_gap - val_size) as i32;

    // --- B. Header 区 ---
    let params_top_y = value_draw_y as f32;
    let script_size = bh * cfg.header_script_size; 
    let small_size = bh * cfg.header_small_size;   
    let gap_top = bh * cfg.header_gap_top;
    let gap_bottom = bh * cfg.header_gap_bottom;

    let script_baseline_y = params_top_y - (bh * cfg.header_bottom_margin);
    let line2_y = script_baseline_y as i32;
    let line1_y = (script_baseline_y - (script_size * 0.5) - gap_top) as i32;
    let line3_y = (script_baseline_y + (script_size * 0.1) + gap_bottom) as i32;

    // --- C. 分隔线 ---
    let sep_top = value_draw_y as f32;
    let sep_bottom = label_draw_y as f32 + lbl_size;
    let sep_full_h = sep_bottom - sep_top;
    let sep_actual_h = sep_full_h * cfg.separator_scale;
    let sep_center_y = sep_top + (sep_full_h / 2.0);

    // 颜色定义
    let text_color = Rgba([255, 255, 255, 245]); 
    let label_color = Rgba([255, 255, 255, 160]);
    let script_color = Rgba([240, 230, 210, 250]); 
    let small_title_color = Rgba([255, 255, 255, 200]);
    let sep_color = Rgba([255, 255, 255, cfg.separator_opacity]);

    // 7. 绘制 Header
    draw_centered_text(&mut canvas, "MASTER SERIES", center_x, line1_y, serif_font, PxScale{x: small_size, y: small_size}, small_title_color);
    draw_centered_text(&mut canvas, "The decisive moment", center_x, line2_y, script_font, PxScale{x: script_size, y: script_size}, script_color);
    draw_wide_text(&mut canvas, center_x, line3_y, "PHOTOGRAPH", serif_font, small_size, small_title_color);

    // 8. 绘制参数列
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

    // 9. 绘制竖线
    draw_separator(&mut canvas, center_x - gap, sep_center_y, sep_actual_h, sep_color);
    draw_separator(&mut canvas, center_x, sep_center_y, sep_actual_h, sep_color);
    draw_separator(&mut canvas, center_x + gap, sep_center_y, sep_actual_h, sep_color);

    println!("[PERF] 排版与合成耗时: {:?}", start_overlay.elapsed());
    println!("[PERF] 总耗时: {:?}", start_total.elapsed());

    canvas
}

// ---------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------

// 🟢 [高性能版] 缩图 -> 模糊 -> 放大
fn create_aspect_fill_bg_optimized(img: &DynamicImage, target_w: u32, target_h: u32, blur_radius: f32) -> DynamicImage {
    // 1. 定义缩小倍数 (Scale Factor)
    // 对于高斯模糊背景，1/10 甚至 1/20 的分辨率足以提供平滑的色块，且速度提升百倍
    // 我们限制短边至少保留 300px，防止过度马赛克
    let (src_w, src_h) = img.dimensions();
    let min_dimension = 300.0;
    
    // 计算缩放比例
    let scale_factor = (min_dimension / (src_w.min(src_h) as f64)).min(0.2); // 最多缩小到 20%
    
    let tiny_w = (src_w as f64 * scale_factor) as u32;
    let tiny_h = (src_h as f64 * scale_factor) as u32;

    // 2. 缩小原图 (使用 Nearest 即可，因为马上要模糊，不需要高质量插值)
    let tiny_img = img.resize_exact(tiny_w, tiny_h, imageops::FilterType::Nearest);

    // 3. 计算对应的 target 尺寸的缩小版
    // 我们需要先裁切出 target 的比例，但是是在 tiny 图上裁
    let ratio_target = target_w as f64 / target_h as f64;
    let ratio_tiny = tiny_w as f64 / tiny_h as f64;

    let (crop_w, crop_h) = if ratio_target > ratio_tiny {
        // 目标更宽，以宽为准
        (tiny_w, (tiny_w as f64 / ratio_target) as u32)
    } else {
        // 目标更高，以高为准
        ((tiny_h as f64 * ratio_target) as u32, tiny_h)
    };

    let crop_x = (tiny_w - crop_w) / 2;
    let crop_y = (tiny_h - crop_h) / 2;

    // 4. 在小图上裁切
    let cropped_tiny = tiny_img.crop_imm(crop_x, crop_y, crop_w, crop_h);

    // 5. 应用等效模糊半径
    // 原图模糊 150px，缩图后模糊半径 = 150 * scale_factor
    let effective_blur = blur_radius * (scale_factor as f32);
    
    // 执行模糊 (此时计算量极小)
    let blurred_tiny = cropped_tiny.blur(effective_blur);

    // 6. 放大回目标尺寸 (使用 Triangle 线性插值保证过渡平滑)
    blurred_tiny.resize_exact(target_w, target_h, imageops::FilterType::Triangle)
}

// ⬇️ 其他辅助函数保持不变
fn draw_wide_text(canvas: &mut DynamicImage, center_x: i32, y: i32, text: &str, font: &FontRef, size: f32, color: Rgba<u8>) {
    let scale = PxScale { x: size, y: size };
    let tracking = size * 0.4; 
    let mut total_width = 0.0;
    let char_widths: Vec<f32> = text.chars().map(|c| {
        let (w, _) = imageproc::drawing::text_size(scale, font, &c.to_string());
        total_width += w as f32 + tracking;
        w as f32
    }).collect();
    if total_width > 0.0 { total_width -= tracking; }
    let mut current_x = center_x as f32 - (total_width / 2.0);
    for (i, c) in text.chars().enumerate() {
        draw_text_mut(canvas, color, current_x as i32, y, scale, font, &c.to_string());
        current_x += char_widths[i] + tracking;
    }
}

fn draw_column_absolute(canvas: &mut DynamicImage, x: i32, val_y: i32, lbl_y: i32, value: &str, label: &str, font: &FontRef, val_size: f32, lbl_size: f32, val_color: Rgba<u8>, lbl_color: Rgba<u8>) {
    draw_centered_text(canvas, value, x, val_y, font, PxScale { x: val_size, y: val_size }, val_color);
    draw_centered_text(canvas, label, x, lbl_y, font, PxScale { x: lbl_size, y: lbl_size }, lbl_color);
}

fn draw_separator(canvas: &mut DynamicImage, x: i32, center_y: f32, height: f32, color: Rgba<u8>) {
    let start_y = center_y - (height / 2.0);
    let end_y = center_y + (height / 2.0);
    draw_line_segment_mut(canvas, (x as f32, start_y), (x as f32, end_y), color);
}

fn draw_centered_text(canvas: &mut DynamicImage, text: &str, x: i32, y: i32, font: &FontRef, scale: PxScale, color: Rgba<u8>) {
    let (text_w, _text_h) = imageproc::drawing::text_size(scale, font, text);
    let draw_x = x - (text_w as i32 / 2);
    draw_text_mut(canvas, color, draw_x, y, scale, font, text);
}

fn parse_params_smart(params: &str) -> (String, String, String, String) {
    let parts: Vec<&str> = params.split_whitespace().collect();
    let mut iso = String::from("");
    let mut aperture = String::from("");
    let mut shutter = String::from("");
    let mut focal = String::from("");
    for (i, part) in parts.iter().enumerate() {
        let p = part.to_lowercase();
        if p == "mm" { if i > 0 { focal = parts[i-1].to_string(); } } 
        else if p.ends_with("mm") { focal = part.to_string(); } 
        else if p.starts_with("f/") || (p.starts_with("f") && p.len() > 1 && p.chars().nth(1).unwrap().is_numeric()) { aperture = part.to_string(); }
        else if p == "s" { if i > 0 { shutter = parts[i-1].to_string(); } }
        else if p.ends_with("s") && !p.contains("iso") { shutter = part.to_string(); }
        else if p.contains("1/") { shutter = part.to_string(); }
        else if p == "iso" { if i + 1 < parts.len() { iso = format!("ISO {}", parts[i+1]); } }
        else if p.starts_with("iso") { let val = p.replace("iso", ""); if !val.is_empty() { iso = format!("ISO {}", val); } }
        else if part.chars().all(|c| c.is_numeric()) { iso = format!("ISO {}", part); }
    }
    (iso, aperture, shutter, focal)
}

fn clean_param(raw: &str, remove: &str) -> String {
    raw.to_uppercase()
        .replace(&remove.to_uppercase(), "")
        .replace(&remove.to_lowercase(), "")
        .trim()
        .to_string()
}