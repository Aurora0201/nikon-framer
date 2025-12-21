use std::fs;
use std::path::Path;
use image::{DynamicImage, ImageFormat};

// 🟢 1. 在这里定义所有内置字体的显示名称
// 只要文件名传回来是这个，我们就加载 include_bytes! 里的数据
const BUILTIN_FONT_NAME: &str = "Nikon-Default.ttf";

// 扫描字体列表 (内置 + 用户目录)
pub fn get_font_list() -> Vec<String> {
    let mut fonts = Vec::new();

    // 🟢 步骤 A: 添加内置字体到列表最前面
    fonts.push(BUILTIN_FONT_NAME.to_string());

    // 🟢 步骤 B: 扫描用户 "fonts" 文件夹
    let font_dir = "fonts"; 
    // 确保目录存在，不存在则创建，避免报错
    if !Path::new(font_dir).exists() {
        let _ = fs::create_dir(font_dir);
    }

    if let Ok(entries) = fs::read_dir(font_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ext_str == "ttf" || ext_str == "otf" {
                            if let Some(name) = path.file_name() {
                                let name_str = name.to_string_lossy().to_string();
                                // 防止用户文件夹里也有一个叫这个名字的文件导致重复显示
                                if name_str != BUILTIN_FONT_NAME {
                                    fonts.push(name_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fonts
}

// 加载字体数据 (根据名称分流)
pub fn load_font_data(font_filename: &str) -> Vec<u8> {
    // 🟢 判断 1: 如果是内置字体名，或者是空的 (第一次启动)，或者是 "default" (旧版兼容)
    if font_filename == BUILTIN_FONT_NAME || font_filename == "default" || font_filename.is_empty() {
        // 直接返回编译进二进制的字体数据
        return include_bytes!("../assets/fonts/InterDisplay-Bold.otf").to_vec();
    } 
    
    // 🟢 判断 2: 否则去读取用户文件夹
    let custom_path = Path::new("fonts").join(font_filename);
    match fs::read(&custom_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("⚠️ 无法读取用户字体: {:?}，回退到内置字体。", custom_path);
            include_bytes!("../assets/fonts/InterDisplay-Bold.otf").to_vec()
        },
    }
}

// ... (load_logo_image 和 load_brand_logos 保持不变) ...
pub fn load_logo_image(make: &str) -> Option<DynamicImage> {
    let make_upper = make.to_uppercase();
    let logo_data = if make_upper.contains("NIKON") {
        Some(include_bytes!("../assets/logos/Nikon.png") as &[u8])
    } else {
        None
    };

    if let Some(data) = logo_data {
        image::load_from_memory_with_format(data, ImageFormat::Png).ok()
    } else {
        None
    }
}

pub struct BrandLogos {
    pub icon: Option<DynamicImage>,
    pub word: Option<DynamicImage>,
    pub z_symbol: Option<DynamicImage>,
}

pub fn load_brand_logos(make: &str) -> BrandLogos {
    let make_upper = make.to_uppercase();
    
    if make_upper.contains("NIKON") {
        let icon_data = include_bytes!("../assets/logos/Nikon.png");
        let word_data = include_bytes!("../assets/logos/Nikon-word.png");
        
        let z_data_res = std::panic::catch_unwind(|| {
            include_bytes!("../assets/logos/Z.png")
        });
        
        let z_img = match z_data_res {
            Ok(data) => image::load_from_memory_with_format(data, ImageFormat::Png).ok(),
            Err(_) => None,
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