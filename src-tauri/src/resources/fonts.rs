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

    // 2. 确定文件名
    let filename = key.filename();
    
    // 3. 🟢 [核心修改] 智能路径查找策略
    // 策略 A: 优先使用 setup.rs 初始化的路径 (通常指向 target/debug/assets 或 安装后的资源目录)
    let base_dir_guard = FONT_BASE_DIR.lock().unwrap();
    
    // 构造首选路径
    let primary_path = if let Some(base) = base_dir_guard.as_deref() {
        base.join(filename)
    } else {
        // 如果未初始化，默认找相对路径
        Path::new("assets/fonts").join(filename)
    };

    // 4. 检查文件是否存在，如果不存在，尝试 "开发环境回退策略"
    let final_path = if primary_path.exists() {
        primary_path
    } else {
        // 🟢 [Dev Fallback] 如果首选路径找不到，尝试去源码目录找
        // CARGO_MANIFEST_DIR 是编译时环境变量，指向 Cargo.toml 所在的目录 (即 src-tauri)
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts")
            .join(filename);

        if source_path.exists() {
            println!("⚠️ [Resources] 首选路径缺失，回退到源码目录加载: {:?}", source_path);
            source_path
        } else {
            // 如果源码目录也没有，那就真的没了，还是报错原路径让用户检查
            primary_path 
        }
    };

    println!("📦 [LazyLoad] Font: {:?} -> {:?}", key, final_path);

    let data = fs::read(&final_path).unwrap_or_else(|e| {
        // 打印详细错误信息，帮助调试
        eprintln!("❌ 严重错误: 无法读取字体文件!");
        eprintln!("   - 尝试路径: {:?}", final_path);
        eprintln!("   - 系统错误: {}", e);
        vec![]
    });

    let arc_data = Arc::new(data);
    cache.insert(key, arc_data.clone());
    
    arc_data
}