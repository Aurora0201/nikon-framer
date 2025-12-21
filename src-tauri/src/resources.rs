use std::fs;
use std::path::Path;
use image::{DynamicImage, ImageFormat};

// 🟢 [修改] 结构体新增 z_symbol
pub struct BrandLogos {
    pub icon: Option<DynamicImage>, // 金色方块
    pub word: Option<DynamicImage>, // Nikon 文字
    pub z_symbol: Option<DynamicImage>, // 🟢 新增：Z 字符图片
}

pub fn get_font_list() -> Vec<String> {
    let font_dir = "fonts"; 
    let mut fonts = Vec::new();

    if let Ok(entries) = fs::read_dir(font_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ext_str == "ttf" || ext_str == "otf" {
                            if let Some(name) = path.file_name() {
                                fonts.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    fonts
}

pub fn load_font_data(font_filename: &str) -> Vec<u8> {
    if font_filename == "default" || font_filename.is_empty() {
        #[cfg(target_os = "windows")]
        {
            let sys_font = Path::new("C:\\Windows\\Fonts\\arial.ttf");
            if sys_font.exists() {
                if let Ok(bytes) = fs::read(sys_font) {
                    return bytes;
                }
            }
        }
        include_bytes!("../assets/fonts/default.ttf").to_vec()
    } else {
        let custom_path = Path::new("fonts").join(font_filename);
        match fs::read(&custom_path) {
            Ok(bytes) => bytes,
            Err(_) => include_bytes!("../assets/fonts/default.ttf").to_vec(),
        }
    }
}

// 🟢 [修改] 加载逻辑，包含 Z.png
pub fn load_brand_logos(make: &str) -> BrandLogos {
    let make_upper = make.to_uppercase();
    
    if make_upper.contains("NIKON") {
        let icon_data = include_bytes!("../assets/logos/Nikon.png");
        let word_data = include_bytes!("../assets/logos/Nikon-word.png");
        
        // 🟢 尝试加载 Z.png
        // 请确保 src-tauri/assets/logos/Z.png 存在
        let z_data_res = std::panic::catch_unwind(|| {
            include_bytes!("../assets/logos/Z.png")
        });
        
        let z_img = match z_data_res {
            Ok(data) => image::load_from_memory_with_format(data, ImageFormat::Png).ok(),
            Err(_) => None, // 如果文件不存在，防止崩溃
        };

        BrandLogos {
            icon: image::load_from_memory_with_format(icon_data, ImageFormat::Png).ok(),
            word: image::load_from_memory_with_format(word_data, ImageFormat::Png).ok(),
            z_symbol: z_img,
        }
    } else {
        BrandLogos { icon: None, word: None, z_symbol: None }
    }
}