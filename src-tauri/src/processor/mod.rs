pub mod white;
pub mod blur;

use std::path::Path;
use std::io::Cursor;
use std::time::Instant;
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use base64::{Engine as _, engine::general_purpose};
use ab_glyph::FontRef; 

use crate::resources; 

// ... (DrawContext, resize_image_by_height, format_model_text, clean_model_name 保持不变) ...
// 请保留这些辅助函数，这里为了节省篇幅省略，记得不要删掉它们！
pub struct DrawContext<'a> {
    pub canvas: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub font: &'a FontRef<'a>,
    pub font_weight: &'a str,
}

pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

pub fn format_model_text(model: &str) -> String {
    model.replace("Z", "ℤ")
}

pub fn clean_model_name(make: &str, model: &str) -> String {
    let make_clean = make.replace("CORPORATION", "").trim().to_string(); 
    let model_upper = model.to_uppercase();
    let make_upper = make_clean.to_uppercase();
    let model_base = if let Some(idx) = model_upper.find(&make_upper) {
        let start = idx + make_upper.len();
        let rest = &model[start..];
        rest.trim().to_string()
    } else {
        model.to_string()
    };
    let mut no_make = if model_base.to_uppercase().starts_with("NIKON") {
        model_base[5..].trim().to_string()
    } else {
        model_base
    };
    no_make = no_make.trim().to_string();
    if no_make.to_uppercase().starts_with("Z") {
        no_make = no_make[1..].trim().to_string();
    }
    no_make
}

// 🚀 主入口
pub fn run(
    file_path: String, 
    style: String, 
    font_filename: String, 
    font_weight: String, 
    shadow_intensity: f32,
    camera_make: String,
    camera_model: String,
    
    shooting_params: String
) -> Result<String, String> {
    
    // 1. 打开图片
    let t_open = Instant::now();
    let img = image::open(&file_path).map_err(|e| format!("打开图片失败: {}", e))?;
    println!("⏱️ [PERF] 原图加载耗时: {:.2?}", t_open.elapsed());

    // 2. 加载资源
    let t_res = Instant::now();
    let font_data = resources::load_font_data(&font_filename);
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体加载错误")?;
    let logos = resources::load_brand_logos(&camera_make);
    println!("⏱️ [PERF] 字体与Logo加载耗时: {:.2?}", t_res.elapsed());

    // 3. 核心处理 (白底/高斯)
    let t_process = Instant::now();
    let final_image = match style.as_str() {
        "BottomWhite" => white::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, &logos),
        "GaussianBlur" => blur::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, shadow_intensity, &logos),
        _ => return Err("未知的样式".to_string()),
    };
    println!("⏱️ [PERF] 核心绘图逻辑耗时: {:.2?}", t_process.elapsed());

    // 4. 保存文件
    let t_save = Instant::now();
    let path_obj = Path::new(&file_path);
    let file_stem = path_obj.file_stem().ok_or("无效文件名")?.to_string_lossy();
    let parent = path_obj.parent().ok_or("无效目录")?;
    let new_filename = format!("{}_{}.jpg", file_stem, style);
    let output_path = parent.join(new_filename);
    
    let rgb_final = final_image.to_rgb8();
    rgb_final.save(&output_path).map_err(|e| format!("保存失败: {}", e))?;
    println!("⏱️ [PERF] 结果保存到磁盘耗时: {:.2?}", t_save.elapsed());

    // 5. 编码 Base64 预览
    let t_encode = Instant::now();
    let mut buffer = Cursor::new(Vec::new());
    // 使用 Jpeg 格式且质量稍微降低一点以加快预览传输，或者保持原样
    rgb_final.write_to(&mut buffer, image::ImageFormat::Jpeg).map_err(|e| format!("预览生成失败: {}", e))?;
    let base64_str = general_purpose::STANDARD.encode(buffer.get_ref());
    println!("⏱️ [PERF] Base64编码耗时: {:.2?}", t_encode.elapsed());

    Ok(format!("data:image/jpeg;base64,{}", base64_str))
}