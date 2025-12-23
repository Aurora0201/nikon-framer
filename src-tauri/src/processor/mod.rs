// src-tauri/src/processor/mod.rs

pub mod white;
pub mod blur;

use std::path::Path;
use std::io::Cursor;

// 🟢 修改点：引入 ImageFormat，去掉 ImageOutputFormat (为了兼容性)
use image::{DynamicImage, ImageBuffer, Rgba, imageops, ImageFormat};
use base64::{Engine as _, engine::general_purpose};
use ab_glyph::FontRef; 

// 引用 resources 模块
use crate::resources; 

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

// 🟢 修复点：添加了缺失的分号，并补全了完整逻辑
pub fn clean_model_name(make: &str, model: &str) -> String {
    let make_clean = make.replace("CORPORATION", "").trim().to_string(); 
    let model_upper = model.to_uppercase();
    let make_upper = make_clean.to_uppercase();
    
    // 提取型号主体
    let model_base = if let Some(idx) = model_upper.find(&make_upper) {
        let start = idx + make_upper.len();
        let rest = &model[start..];
        rest.trim().to_string()
    } else {
        model.to_string()
    }; // 🟢 之前报错就是这里少了这个分号！

    // 去除 NIKON 前缀
    let mut no_make = if model_base.to_uppercase().starts_with("NIKON") {
        model_base[5..].trim().to_string()
    } else {
        model_base
    };
    
    no_make = no_make.trim().to_string();
    
    // 去除 Z 前缀 (如果需要)
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
    let img = image::open(&file_path).map_err(|e| format!("打开图片失败: {}", e))?;

    // 2. 加载资源
    let font_data = resources::load_font_data(&font_filename);
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体加载错误")?;
    let logos = resources::load_brand_logos(&camera_make);

    // 3. 核心处理
    let final_image = match style.as_str() {
        "BottomWhite" => white::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, &logos),
        "GaussianBlur" => blur::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, shadow_intensity, &logos),
        _ => return Err("未知的样式".to_string()),
    };

    // 4. 保存文件
    let path_obj = Path::new(&file_path);
    let file_stem = path_obj.file_stem().ok_or("无效文件名")?.to_string_lossy();
    let parent = path_obj.parent().ok_or("无效目录")?;
    
    // 生成文件名：原名_BottomWhite.jpg
    let new_filename = format!("{}_{}.jpg", file_stem, style);
    let output_path = parent.join(new_filename);
    
    let rgb_final = final_image.to_rgb8();
    rgb_final.save(&output_path).map_err(|e| format!("保存失败: {}", e))?;
    println!("✅ 已保存: {:?}", output_path);

    // 5. 编码 Base64 预览
    let mut buffer = Cursor::new(Vec::new());
    
    // 🟢 修复点：使用 ImageFormat::Jpeg，而不是 ImageOutputFormat
    // 这样兼容性最好，使用默认质量
    rgb_final.write_to(&mut buffer, ImageFormat::Jpeg)
        .map_err(|e| format!("预览生成失败: {}", e))?;
        
    let base64_str = general_purpose::STANDARD.encode(buffer.get_ref());

    Ok(format!("data:image/jpeg;base64,{}", base64_str))
}