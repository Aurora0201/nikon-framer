use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{error, info};
use once_cell::sync::Lazy;
use image::{DynamicImage};
use std::fmt; // 引入格式化库

// =========================================================
// 🟢 Logo 资源管理系统 (Brand & Logo Assets)
// =========================================================

// 1. 品牌枚举
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Nikon,
    Sony,
    Canon,
    Fujifilm,
    Leica,
    Hasselblad,
    Other
    // ...
}

// 🟢 核心：实现 Display 特征
impl fmt::Display for Brand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 这里定义你希望转换成的字符串样子
        // 通常建议用首字母大写的标准写法
        let s = match self {
            Brand::Nikon => "Nikon",
            Brand::Sony => "Sony",
            Brand::Canon => "Canon",
            Brand::Fujifilm => "Fujifilm",
            Brand::Leica => "Leica",
            Brand::Hasselblad => "Hasselblad",
            Brand::Other => "Unkonwn", // 或者是 "Unknown"
        };
        write!(f, "{}", s)
    }
}

// 2. Logo 具体描述符
#[allow(dead_code)]
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
            (Brand::Nikon, LogoType::Wordmark)      => Some(include_bytes!("../../assets/logos/nikon-wordmark.png")),
            (Brand::Nikon, LogoType::SymbolZ)       => Some(include_bytes!("../../assets/logos/nikon-symbol-z.png")),
            (Brand::Nikon, LogoType::IconYellowBox) => Some(include_bytes!("../../assets/logos/nikon-icon-yellow-box.png")),

            // === Sony (暂未添加文件，注释以防报错) ===
            (Brand::Sony, LogoType::Wordmark)    => Some(include_bytes!("../../assets/logos/sony-wordmark.png")),
            // (Brand::Sony, LogoType::SymbolAlpha) => Some(include_bytes!("../assets/logos/Alpha.png")),

            // === Leica (暂未添加文件) ===
            // (Brand::Leica, LogoType::Wordmark)   => Some(include_bytes!("../assets/logos/Leica-Word.png")),
            // (Brand::Leica, LogoType::IconRedDot) => Some(include_bytes!("../assets/logos/Leica-Red.png")),

            // === Canon (暂未添加文件) ===
            (Brand::Canon, LogoType::Wordmark)   => Some(include_bytes!("../../assets/logos/canon-wordmark.png")),

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
        info!("📦 [Resources] 首次加载 Logo: {:?} - {:?}", brand, l_type);
        
        // 解码图片 (支持 png, jpg 等格式)
        if let Ok(img) = image::load_from_memory(data) {
            let arc_img = Arc::new(img);
            
            // C. 第三步：写入缓存 (写锁)
            let mut cache = LOGO_CACHE.lock().unwrap();
            cache.insert(key, arc_img.clone());
            
            return Some(arc_img);
        } else {
            error!("❌ [Resources] 图片解码失败: {:?} - {:?}", brand, l_type);
        }
    } else {
        // 如果 load_data 返回 None (说明该品牌该类型没有定义资源)
        // 可以在这里打印日志方便调试
        info!("⚠️ [Resources] 未定义的 Logo 资源: {:?} - {:?}", brand, l_type);
    }

    None
}