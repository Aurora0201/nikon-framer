// 1. 声明子模块
pub mod models;
pub mod traits;
pub(crate) mod impls; // 内部实现细节，对外隐藏，对内可见

use crate::resources::Brand;
use models::{RawExifData, ParsedImageContext, ShootingParams};
use traits::BrandParser;
// 引入具体的解析器实现
use impls::{NikonParser, SonyParser, CanonParser};

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