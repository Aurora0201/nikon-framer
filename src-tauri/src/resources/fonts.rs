use std::collections::HashMap;
use std::sync::{Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use ab_glyph::FontArc;
use log::{error, info};
use once_cell::sync::Lazy;


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
    info!("✅ [Resources] 字体路径已初始化");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    InterDisplay,  // 现代无衬线
    MrDafoe,       // 手写体
    AbhayaLibre,   // 衬线体
    Birthstone,
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
            (FontFamily::Birthstone, _)                    => "Birthstone-Regular.ttf"
        }
    }
}

type FontCache = HashMap<FontKey, FontArc>;

static FONT_CACHE: Lazy<Mutex<FontCache>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// **获取字体资源 (返回解析好的字体对象)**
/// 
/// 优势：
/// 1. 缓存的是解析后的字体对象，避免重复 parse。
/// 2. 调用者拿来即用，无需再次 try_from_slice。
pub fn get_font(family: FontFamily, weight: FontWeight) -> FontArc {
    let key = FontKey { family, weight };

    // 1. 查缓存
    // 🟢 [修改点] 这里的 cache 已经是 HashMap<FontKey, FontArc>
    let mut cache = FONT_CACHE.lock().unwrap();
    if let Some(font) = cache.get(&key) {
        return font.clone(); // FontArc 克隆开销很小 (类似 Arc::clone)
    }

    // 2. 确定文件名
    let filename = key.filename();
    
    // 3. 智能路径查找策略 (保持原逻辑不变)
    let base_dir_guard = FONT_BASE_DIR.lock().unwrap();
    
    let primary_path = if let Some(base) = base_dir_guard.as_deref() {
        base.join(filename)
    } else {
        Path::new("assets/fonts").join(filename)
    };

    // 4. 路径回退检查 (保持原逻辑不变)
    let final_path = if primary_path.exists() {
        primary_path
    } else {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts")
            .join(filename);

        if source_path.exists() {
            info!("⚠️ [Resources] 首选路径缺失，回退到源码目录加载: {:?}", source_path);
            source_path
        } else {
            primary_path 
        }
    };

    info!("📦 [LazyLoad] Font: {:?} -> {:?}", key, final_path);

    // 5. 读取文件字节
    let data = fs::read(&final_path).unwrap_or_else(|e| {
        error!("❌ 严重错误: 无法读取字体文件!");
        error!("   - 尝试路径: {:?}", final_path);
        error!("   - 系统错误: {}", e);
        // 如果读不到文件，这里可以 Panic，或者返回一个内嵌的 Fallback 字体
        // 这里暂时 panic，因为没有字体后续无法工作
        panic!("无法加载核心字体资源: {:?}", final_path);
    });

    // 6. 🟢 [核心修改] 将字节解析为 FontArc
    // FontArc::try_from_vec 会接管 data 的所有权，不会发生拷贝
    let font = FontArc::try_from_vec(data).unwrap_or_else(|_| {
        error!("❌ 严重错误: 字体文件格式损坏!");
        error!("   - 路径: {:?}", final_path);
        panic!("无法解析字体文件");
    });

    // 7. 存入缓存并返回
    cache.insert(key, font.clone());
    
    font
}