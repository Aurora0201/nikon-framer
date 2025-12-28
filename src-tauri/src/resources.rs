use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use once_cell::sync::Lazy;
use image::{DynamicImage, ImageFormat};

// =========================================================
// 🟢 Logo 资源管理系统 (Brand & Logo Assets)
// =========================================================

// 1. 品牌枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Nikon,
    Sony,
    Canon,
    Fujifilm,
    Leica,
    Hasselblad,
    // ...
}

// 2. Logo 具体描述符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogoType {
    // --- 通用型 ---
    Wordmark,         // 标准字标 (如 "Nikon", "Sony")
    WordmarkVertical, // 竖排字标

    // --- 尼康专属 ---
    IconYellowBox,    // 尼康小黄块
    SymbolZ,          // Z 系列 Logo
    
    // --- 索尼专属 ---
    SymbolAlpha,      // α (Alpha) Logo
    SymbolGMaster,    // G Master Logo
    
    // --- 徕卡专属 ---
    IconRedDot,       // 可乐标 (红)
    IconBlackDot,     // 黑标
    
    // --- 富士专属 ---
    SymbolGFX,        // GFX 系统标
    SymbolX,          // X 系统标
}

// 3. 组合键 (用于 Map 索引)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LogoKey {
    brand: Brand,
    l_type: LogoType,
}

impl LogoKey {
    // 🟢 加载逻辑：精确匹配 品牌 + 类型
    // 注意：目前仅开启 Nikon，其他品牌注释掉以防编译时找不到文件报错
    fn load_data(&self) -> Option<&'static [u8]> {
        match (self.brand, self.l_type) {
            // === Nikon ===
            (Brand::Nikon, LogoType::Wordmark)      => Some(include_bytes!("../assets/logos/Nikon-word.png")),
            (Brand::Nikon, LogoType::SymbolZ)       => Some(include_bytes!("../assets/logos/Z.png")),
            (Brand::Nikon, LogoType::IconYellowBox) => Some(include_bytes!("../assets/logos/Nikon.png")),

            // === Sony (暂未添加文件，注释以防报错) ===
            // (Brand::Sony, LogoType::Wordmark)    => Some(include_bytes!("../assets/logos/Sony.png")),
            // (Brand::Sony, LogoType::SymbolAlpha) => Some(include_bytes!("../assets/logos/Alpha.png")),

            // === Leica (暂未添加文件) ===
            // (Brand::Leica, LogoType::Wordmark)   => Some(include_bytes!("../assets/logos/Leica-Word.png")),
            // (Brand::Leica, LogoType::IconRedDot) => Some(include_bytes!("../assets/logos/Leica-Red.png")),

            // === Canon (暂未添加文件) ===
            // (Brand::Canon, LogoType::Wordmark)   => Some(include_bytes!("../assets/logos/Canon.png")),

            // 其他未定义的组合返回 None
            _ => None,
        }
    }
}

// 4. Logo 缓存池定义
// Key: 品牌+类型, Value: 线程安全的图片引用
type LogoCache = HashMap<LogoKey, Arc<DynamicImage>>;

static LOGO_CACHE: Lazy<Mutex<LogoCache>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// **获取 Logo 资源 (懒加载实现)**
/// 
/// 用法: resources::get_logo(Brand::Nikon, LogoType::Wordmark)
pub fn get_logo(brand: Brand, l_type: LogoType) -> Option<Arc<DynamicImage>> {
    let key = LogoKey { brand, l_type };

    // A. 第一步：查缓存 (读锁)
    // 如果缓存里有，直接返回，速度极快
    {
        let cache = LOGO_CACHE.lock().unwrap();
        if let Some(img) = cache.get(&key) {
            return Some(img.clone());
        }
    }

    // B. 第二步：缓存未命中，执行加载
    // 这一步涉及文件解码，相对耗时
    if let Some(data) = key.load_data() {
        println!("📦 [Resources] 首次加载 Logo: {:?} - {:?}", brand, l_type);
        
        // 解码图片 (支持 png, jpg 等格式)
        if let Ok(img) = image::load_from_memory(data) {
            let arc_img = Arc::new(img);
            
            // C. 第三步：写入缓存 (写锁)
            let mut cache = LOGO_CACHE.lock().unwrap();
            cache.insert(key, arc_img.clone());
            
            return Some(arc_img);
        } else {
            eprintln!("❌ [Resources] 图片解码失败: {:?} - {:?}", brand, l_type);
        }
    } else {
        // 如果 load_data 返回 None (说明该品牌该类型没有定义资源)
        // 可以在这里打印日志方便调试
        // println!("⚠️ [Resources] 未定义的 Logo 资源: {:?} - {:?}", brand, l_type);
    }

    None
}

// =========================================================
// 🟢 字体资源管理系统 (Font Assets) - 保持不变以维持功能
// =========================================================

// 用于存储真实的资源绝对路径 (由 setup.rs 初始化)
static FONT_BASE_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| {
    Mutex::new(None)
});

// 初始化函数
pub fn init_font_path(path: PathBuf) {
    let mut dir = FONT_BASE_DIR.lock().unwrap();
    *dir = Some(path);
    println!("✅ [Resources] 字体路径已初始化");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    InterDisplay,  // 现代无衬线
    MrDafoe,       // 手写体
    AbhayaLibre,   // 衬线体
}

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

impl FontKey {
    fn filename(&self) -> &'static str {
        match (self.family, self.weight) {
            (FontFamily::InterDisplay, FontWeight::Bold)   => "InterDisplay-Bold.otf",
            (FontFamily::InterDisplay, FontWeight::Medium) => "InterDisplay-Medium.otf",
            (FontFamily::InterDisplay, _)                  => "InterDisplay-Regular.otf",
            (FontFamily::MrDafoe, _)                       => "MrDafoe-Regular.ttf",
            (FontFamily::AbhayaLibre, _)                   => "AbhayaLibre-Medium.ttf",
        }
    }
}

type FontCache = HashMap<FontKey, Arc<Vec<u8>>>;

static FONT_CACHE: Lazy<Mutex<FontCache>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// **获取字体资源**
pub fn get_font(family: FontFamily, weight: FontWeight) -> Arc<Vec<u8>> {
    let key = FontKey { family, weight };

    // 1. 查缓存
    let mut cache = FONT_CACHE.lock().unwrap();
    if let Some(data) = cache.get(&key) {
        return data.clone();
    }

    // 2. 加载文件
    let filename = key.filename();
    
    // 使用全局初始化的路径
    let base_dir_guard = FONT_BASE_DIR.lock().unwrap();
    // 兜底逻辑：如果未初始化(如测试环境)，尝试相对路径
    let folder = base_dir_guard.as_deref().unwrap_or(Path::new("assets/fonts"));
    let path = folder.join(filename);
    
    println!("📦 [LazyLoad] Font: {:?} -> {:?}", key, path);

    let data = fs::read(&path).unwrap_or_else(|_| {
        eprintln!("❌ 严重错误: 字体文件缺失 {:?}，加载空数据", path);
        vec![]
    });

    let arc_data = Arc::new(data);
    cache.insert(key, arc_data.clone());
    
    arc_data
}