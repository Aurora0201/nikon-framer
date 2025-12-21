pub mod white;
pub mod blur;

use std::path::Path;
use std::io::Cursor;
use std::time::Instant;
use image::{DynamicImage, ImageBuffer, Rgba, imageops};
use base64::{Engine as _, engine::general_purpose};
use ab_glyph::FontRef; // 🟢 移除了 PxScale

use crate::resources; // 🟢 移除了 BrandLogos

// ==========================================
// 🛠️ 公共结构体与工具 (Shared Utils)
// ==========================================

/// 绘图上下文
pub struct DrawContext<'a> {
    pub canvas: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub font: &'a FontRef<'a>,
    pub font_weight: &'a str,
}

/// 辅助：按高度比例缩放图片
pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

/// 辅助：替换特殊字符 (Z -> ℤ)
pub fn format_model_text(model: &str) -> String {
    model.replace("Z", "ℤ")
}

/// 清理机型名称
pub fn clean_model_name(make: &str, model: &str) -> String {
    let make_clean = make.replace("CORPORATION", "").trim().to_string(); 
    let model_upper = model.to_uppercase();
    let make_upper = make_clean.to_uppercase();
    
    // 1. 移除厂商名
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

    // 2. 移除开头的 "Z"
    no_make = no_make.trim().to_string();
    if no_make.to_uppercase().starts_with("Z") {
        no_make = no_make[1..].trim().to_string();
    }

    no_make
}

// ==========================================
// 🚀 主入口 (Main Entry)
// ==========================================

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
    
    let total_start = Instant::now();
    println!("--------------------------------------------------");
    println!("🚀 [PERF] 开始处理: {:?}", file_path);

    let img = image::open(&file_path).map_err(|e| format!("打开图片失败: {}", e))?;

    let t_res = Instant::now();
    let font_data = resources::load_font_data(&font_filename);
    let font = FontRef::try_from_slice(&font_data).map_err(|_| "字体加载错误")?;
    
    // 加载 Logo 资源
    let logos = resources::load_brand_logos(&camera_make);
    
    println!("⏱️ [3/9] 资源加载耗时: {:.2?}", t_res.elapsed());

    // 🟢 路由分发
    let final_image = match style.as_str() {
        "BottomWhite" => white::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, &logos),
        "GaussianBlur" => blur::process(&img, &camera_make, &camera_model, &shooting_params, &font, &font_weight, shadow_intensity, &logos),
        _ => return Err("未知的样式".to_string()),
    };

    let path_obj = Path::new(&file_path);
    let file_stem = path_obj.file_stem().ok_or("无效文件名")?.to_string_lossy();
    let parent = path_obj.parent().ok_or("无效目录")?;
    let new_filename = format!("{}_{}.jpg", file_stem, style);
    let output_path = parent.join(new_filename);
    
    let rgb_final = final_image.to_rgb8();
    rgb_final.save(&output_path).map_err(|e| format!("保存失败: {}", e))?;

    let mut buffer = Cursor::new(Vec::new());
    rgb_final.write_to(&mut buffer, image::ImageFormat::Jpeg).map_err(|e| format!("预览生成失败: {}", e))?;
    let base64_str = general_purpose::STANDARD.encode(buffer.get_ref());
    
    println!("✅ 总耗时: {:.2?}", total_start.elapsed());
    println!("--------------------------------------------------");

    Ok(format!("data:image/jpeg;base64,{}", base64_str))
}