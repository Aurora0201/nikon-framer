use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
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