use std::fs;
use std::path::Path;
use image::{DynamicImage, ImageFormat};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

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



// 🟢 1. 定义字体家族 (对应你实际拥有的字体系列)
// 以后加新字体，就在这里加名字，不用管它用来做什么
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    InterDisplay,  // 现代无衬线
    MrDafoe,       // 手写体
    AbhayaLibre,   // 衬线体
}

// 🟢 2. 定义字重
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Regular,
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontKey {
    family: FontFamily,
    weight: FontWeight,
}

// 🟢 3. 文件名映射逻辑 (核心配置中心)
// 根据 Family + Weight -> 找到对应的文件名
impl FontKey {
    fn filename(&self) -> &'static str {
        match (self.family, self.weight) {
            // --- Inter Display (OTF) ---
            (FontFamily::InterDisplay, FontWeight::Bold)   => "InterDisplay-Bold.otf",
            (FontFamily::InterDisplay, FontWeight::Medium) => "InterDisplay-Medium.otf",
            // Inter 的 fallback: 如果要 Regular 或者其他未定义的，都用 Regular
            (FontFamily::InterDisplay, _)                  => "InterDisplay-Regular.otf",

            // --- MrDafoe (TTF) ---
            // 手写体通常只有一种字重，无论要什么都给 Regular
            (FontFamily::MrDafoe, _) => "MrDafoe-Regular.ttf",

            // --- AbhayaLibre (TTF) ---
            // 你只有 Medium，所以无论要什么都给 Medium
            (FontFamily::AbhayaLibre, _) => "AbhayaLibre-Medium.ttf",
        }
    }
}

type FontCache = HashMap<FontKey, Arc<Vec<u8>>>;

static FONT_CACHE: Lazy<Mutex<FontCache>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// **获取字体资源**
/// 
/// 用法: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold)
pub fn get_font(family: FontFamily, weight: FontWeight) -> Arc<Vec<u8>> {
    let key = FontKey { family, weight };

    // 1. 查缓存
    let mut cache = FONT_CACHE.lock().unwrap();
    if let Some(data) = cache.get(&key) {
        return data.clone();
    }

    // 2. 加载文件
    let filename = key.filename();
    // 假设你的字体都在 src-tauri/assets/fonts/ 下 (根据你的截图调整路径)
    // ⚠️ 注意：根据你的截图，文件夹是 `assets/fonts`，请确认路径
    let path = Path::new("assets/fonts").join(filename);
    
    println!("📦 [LazyLoad] Font: {:?} -> {:?}", key, path);

    let data = fs::read(&path).unwrap_or_else(|_| {
        eprintln!("❌ 严重错误: 字体文件缺失 {:?}，加载空数据", path);
        vec![]
    });

    let arc_data = Arc::new(data);
    cache.insert(key, arc_data.clone());
    
    arc_data
}