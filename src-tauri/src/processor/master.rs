// src-tauri/src/processor/master.rs

use image::{DynamicImage, Rgba, GenericImageView, imageops};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::{draw_text_mut, draw_line_segment_mut};

// 🟢 1. 布局配置中心 (已更新你的微调参数)
struct MasterLayoutConfig {
    // --- 宏观布局 ---
    border_ratio: f32,
    bottom_ratio: f32,

    // --- 水平间距 ---
    column_gap_ratio: f32,

    // --- 垂直排版 (参数区) ---
    label_bottom_margin: f32,
    row_gap: f32,

    // --- 字号控制 ---
    text_scale_val: f32,
    text_scale_lbl: f32,

    // --- 分隔线控制 ---
    separator_scale: f32,
    separator_opacity: u8,

    // --- Header (顶部三行字) 配置 ---
    header_bottom_margin: f32, 
    header_script_size: f32,   
    header_small_size: f32,    
    header_gap_top: f32,       
    header_gap_bottom: f32,    
}

impl MasterLayoutConfig {
    fn default() -> Self {
        Self {
            // 宏观
            border_ratio: 0.03,
            bottom_ratio: 0.4,

            // 水平
            column_gap_ratio: 0.18,

            // 垂直定位
            label_bottom_margin: 0.18,
            row_gap: 0.001,

            // 字号
            text_scale_val: 0.13,
            text_scale_lbl: 0.07,

            // 分隔线
            separator_scale: 0.75,
            separator_opacity: 40, 

            // --- Header 配置 (已更新) ---
            header_bottom_margin: 0.3, // 整体上移，给下面参数留更多空
            header_script_size: 0.12,
            header_small_size: 0.05,
            
            // 间距控制
            header_gap_top: -0.02,  // 负值让第一行紧贴手写体
            header_gap_bottom: 0.1, // 增加手写体与第三行的距离
        }
    }
}

/// **Process Master Style Image**
pub fn process(
    img: &DynamicImage,
    params: &str,
    main_font: &FontRef,   
    script_font: &FontRef, 
    serif_font: &FontRef,  
) -> DynamicImage {
    let cfg = MasterLayoutConfig::default();

    println!("--------------------------------------------------");
    println!("[DEBUG] Master Process Start. Params: '{}'", params);

    let (img_w, img_h) = img.dimensions();

    // 1. 计算尺寸
    let border_size = (img_h as f32 * cfg.border_ratio) as u32;
    let bottom_height = (img_h as f32 * cfg.bottom_ratio) as u32;

    // 2. 画布总尺寸
    let canvas_w = img_w + (border_size * 2);
    let canvas_h = img_h + border_size + bottom_height;

    // 3. 生成背景
    let mut canvas = create_aspect_fill_bg(img, canvas_w, canvas_h, 50.0);
    canvas = canvas.brighten(-15); 

    // 4. 贴入原图
    imageops::overlay(&mut canvas, img, border_size as i64, border_size as i64);

    // 5. 解析 & 清洗参数
    let (iso_raw, aperture_raw, shutter_raw, focal_raw) = parse_params_smart(params);
    let iso_val = clean_param(&iso_raw, "ISO");
    let aperture_val = clean_param(&aperture_raw, "f/");
    let focal_val = clean_param(&focal_raw, "mm");
    let shutter_val = clean_param(&shutter_raw, "s");

    // ---------------------------------------------------------
    // 6. 排版计算
    // ---------------------------------------------------------
    
    let bh = bottom_height as f32;
    let center_x = canvas_w as i32 / 2;
    
    // --- A. 参数区 (Bottom Section) ---
    let val_size = bh * cfg.text_scale_val;
    let lbl_size = bh * cfg.text_scale_lbl;
    let margin_bottom = bh * cfg.label_bottom_margin;
    let row_gap = bh * cfg.row_gap;

    let label_draw_y = (canvas_h as f32 - margin_bottom - lbl_size) as i32;
    let value_draw_y = (label_draw_y as f32 - row_gap - val_size) as i32;

    // --- B. Header 区 (Top Section) ---
    
    // 锚点：参数行数值的顶部
    let params_top_y = value_draw_y as f32;
    
    let script_size = bh * cfg.header_script_size;
    let small_size = bh * cfg.header_small_size;
    
    // 间距像素值
    let gap_top = bh * cfg.header_gap_top;
    let gap_bottom = bh * cfg.header_gap_bottom;

    // Line 2 (Middle): Script 基准线
    // 以参数区顶部为起点，向上偏移 header_bottom_margin
    let script_baseline_y = params_top_y - (bh * cfg.header_bottom_margin);
    let line2_y = script_baseline_y as i32;

    // Line 1 (Top): "MASTER SERIES"
    // 位置 = Script基线 - (0.5 * script大小) - 上间距
    let line1_y = (script_baseline_y - (script_size * 0.5) - gap_top) as i32;

    // Line 3 (Bottom): "PHOTOGRAPH"
    // 位置 = Script基线 + (0.1 * script大小) + 下间距
    let line3_y = (script_baseline_y + (script_size * 0.1) + gap_bottom) as i32;

    // --- C. 分隔线计算 ---
    let sep_top = value_draw_y as f32;
    let sep_bottom = label_draw_y as f32 + lbl_size;
    let sep_full_h = sep_bottom - sep_top;
    let sep_actual_h = sep_full_h * cfg.separator_scale;
    let sep_center_y = sep_top + (sep_full_h / 2.0);

    // ---------------------------------------------------------
    // 颜色定义
    let text_color = Rgba([255, 255, 255, 245]); 
    let label_color = Rgba([255, 255, 255, 160]);
    let script_color = Rgba([240, 230, 210, 250]); 
    let small_title_color = Rgba([255, 255, 255, 200]);
    let sep_color = Rgba([255, 255, 255, cfg.separator_opacity]);

    // 🟢 7. 绘制 Header
    
    // Line 1: "MASTER SERIES" -> 🟢 改为 draw_centered_text (窄间距/标准间距)
    draw_centered_text(&mut canvas, "MASTER SERIES", center_x, line1_y, serif_font, PxScale{x: small_size, y: small_size}, small_title_color);
    
    // Line 2: "The decisive moment" (手写体)
    draw_centered_text(&mut canvas, "The decisive moment", center_x, line2_y, script_font, PxScale{x: script_size, y: script_size}, script_color);
    
    // Line 3: "PHOTOGRAPH" -> 保持 draw_wide_text (宽间距)，增加层次感
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

    canvas
}

// ---------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------

fn draw_wide_text(
    canvas: &mut DynamicImage, 
    center_x: i32, 
    y: i32, 
    text: &str, 
    font: &FontRef, 
    size: f32, 
    color: Rgba<u8>
) {
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

fn draw_column_absolute(
    canvas: &mut DynamicImage, 
    x: i32, 
    val_y: i32,
    lbl_y: i32,
    value: &str, 
    label: &str, 
    font: &FontRef, 
    val_size: f32,
    lbl_size: f32,
    val_color: Rgba<u8>,
    lbl_color: Rgba<u8>
) {
    draw_centered_text(canvas, value, x, val_y, font, PxScale { x: val_size, y: val_size }, val_color);
    draw_centered_text(canvas, label, x, lbl_y, font, PxScale { x: lbl_size, y: lbl_size }, lbl_color);
}

fn draw_separator(canvas: &mut DynamicImage, x: i32, center_y: f32, height: f32, color: Rgba<u8>) {
    let start_y = center_y - (height / 2.0);
    let end_y = center_y + (height / 2.0);
    draw_line_segment_mut(canvas, (x as f32, start_y), (x as f32, end_y), color);
}

fn draw_centered_text(
    canvas: &mut DynamicImage, 
    text: &str, 
    x: i32, 
    y: i32, 
    font: &FontRef, 
    scale: PxScale, 
    color: Rgba<u8>
) {
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

fn create_aspect_fill_bg(img: &DynamicImage, target_w: u32, target_h: u32, blur_radius: f32) -> DynamicImage {
    let (src_w, src_h) = img.dimensions();
    let ratio_w = target_w as f64 / src_w as f64;
    let ratio_h = target_h as f64 / src_h as f64;
    let scale = ratio_w.max(ratio_h);
    let new_w = (src_w as f64 * scale).ceil() as u32;
    let new_h = (src_h as f64 * scale).ceil() as u32;
    let mut resized = img.resize(new_w, new_h, imageops::FilterType::Triangle);
    let crop_x = (new_w.saturating_sub(target_w)) / 2;
    let crop_y = (new_h.saturating_sub(target_h)) / 2;
    let cropped = resized.crop(crop_x, crop_y, target_w, target_h);
    cropped.blur(blur_radius)
}