// 1. 声明子模块
pub mod models;
pub mod traits;
pub(crate) mod impls; // 内部实现细节，对外隐藏，对内可见

use crate::resources::Brand;
use models::{RawExifData, ParsedImageContext, ShootingParams};
use traits::BrandParser;
// 引入具体的解析器实现
use impls::{NikonParser, SonyParser, CanonParser};

// 🟢 1. 定义 Sony 映射表 (放在这里，作为通用工具)
fn map_sony_model(internal_name: &str) -> String {
    match internal_name.to_uppercase().as_str() {
        "ILCE-1"    => "α1".to_string(),
        "ILCE-9M3"  => "α9 III".to_string(),
        "ILCE-9M2"  => "α9 II".to_string(),
        "ILCE-7RM5" => "α7R V".to_string(),
        "ILCE-7RM4" => "α7R IV".to_string(),
        "ILCE-7RM3" => "α7R III".to_string(),
        "ILCE-7SM3" => "α7S III".to_string(),
        "ILCE-7SM2" => "α7S II".to_string(),
        "ILCE-7M5"  => "α7 V".to_string(),
        "ILCE-7M4"  => "α7 IV".to_string(),
        "ILCE-7M3"  => "α7 III".to_string(),
        "ILCE-7C"   => "α7C".to_string(),
        "ILCE-7CM2" => "α7C II".to_string(),
        "ILCE-7CR"  => "α7CR".to_string(),
        "ILCE-6700" => "α6700".to_string(),
        "ZV-E1"     => "ZV-E1".to_string(),
        _ => internal_name.replace("ILCE-", "α").trim().to_string(),
    }
}

// 🟢 2. 定义通用清洗逻辑 (核心大脑)
// 任何解析器都可以调用这个函数来获得干净的名字
pub(crate) fn clean_model_name_logic(make: &str, model: &str) -> String {
    let make_clean = make.replace("CORPORATION", "").trim().to_uppercase();
    let model_upper = model.to_uppercase();

    // Sony 特殊处理
    if make_clean.contains("SONY") || model_upper.starts_with("ILCE") {
        return map_sony_model(&model_upper);
    }

    // 通用处理：移除品牌前缀 (如 "Canon EOS R5" -> "EOS R5")
    let mut model_base = if let Some(idx) = model_upper.find(&make_clean) {
        let start = idx + make_clean.len();
        let rest = &model[start..];
        rest.trim().to_string()
    } else {
        model.to_string()
    };

    // Nikon 补丁 (防止 Make 是 "NIKON CORPORATION" 但 Model 是 "NIKON Z8")
    if model_base.to_uppercase().starts_with("NIKON") {
        model_base = model_base[5..].trim().to_string();
    }

    model_base
}

/// 🟢 核心入口：智能解析函数
/// 外部只需要调用这一个函数，不需要关心具体是哪个品牌的解析器在工作
pub fn parse(raw: RawExifData) -> ParsedImageContext {
    
    // A. 组建解析器团队 (注册中心)
    // 使用 Box<dyn BrandParser> 实现动态分发 (Polymorphism)
    // 如果以后想支持 Fuji，就在这里加一行 Box::new(FujiParser)
    let parsers: Vec<Box<dyn BrandParser>> = vec![
        Box::new(NikonParser),
        Box::new(SonyParser),
        Box::new(CanonParser),
    ];

    // B. 职责链模式：遍历寻找能处理的解析器
    for parser in parsers {
        if parser.can_parse(&raw) {
            return parser.parse(&raw);
        }
    }

    // C. 兜底逻辑：如果所有解析器都不认识这个品牌，使用通用逻辑
    default_parse(raw)
}

/// 默认解析逻辑 (Fallback)
/// 用于处理未适配的品牌 (如 Leica, Fuji 等尚未编写专门解析器的情况)
fn default_parse(raw: RawExifData) -> ParsedImageContext {
    // 简单的清洗逻辑：把时间里的冒号换成点
    let clean_time = raw.datetime.replace(":", ".");
    
    // 尝试简单的品牌猜测
    let make_upper = raw.make.to_uppercase();
    let brand_guess = if make_upper.contains("FUJI") {
        Brand::Fujifilm
    } else if make_upper.contains("LEICA") {
        Brand::Leica
    } else if make_upper.contains("HASSELBLAD") {
        Brand::Hasselblad
    } else {
        // 如果都不认识，归为 Other (请确保你在 Brand 枚举里加了 Other)
        Brand::Other
    };

    ParsedImageContext {
        brand: brand_guess,
        // 型号不做特殊清洗，直接去除首尾空格
        model_name: raw.model.trim().to_string(), 
        params: ShootingParams {
            iso: raw.iso,
            aperture: raw.aperture,
            shutter_speed: raw.shutter_speed,
            focal_length: raw.focal_length,
            lens_model: raw.lens,
            capture_time: clean_time,
        },
        artist_name: raw.artist.or(raw.copyright),
        gps: None, // 默认不尝试解析 GPS，除非你写了通用的 GPS 解析逻辑
    }
}