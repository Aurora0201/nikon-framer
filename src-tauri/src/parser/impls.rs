use crate::resources::Brand;
use crate::parser::models::{RawExifData, ParsedImageContext, ShootingParams, GeoLocation};
use crate::parser::traits::BrandParser;

// ==========================================
// 1. Nikon 解析器
// ==========================================
pub struct NikonParser;

impl BrandParser for NikonParser {
    fn can_parse(&self, raw: &RawExifData) -> bool {
        raw.make.to_uppercase().contains("NIKON")
    }

    fn parse(&self, raw: &RawExifData) -> ParsedImageContext {
        // 清洗: "NIKON Z 8" -> "Z 8"
        let model_clean = raw.model.replace("NIKON", "").trim().to_string();
        
        // 组装通用数据 (调用底部的辅助函数减少重复代码)
        build_context(Brand::Nikon, model_clean, raw)
    }
}

// ==========================================
// 2. Sony 解析器
// ==========================================
pub struct SonyParser;

impl BrandParser for SonyParser {
    fn can_parse(&self, raw: &RawExifData) -> bool {
        raw.make.to_uppercase().contains("SONY")
    }

    fn parse(&self, raw: &RawExifData) -> ParsedImageContext {
        let model_upper = raw.model.to_uppercase();
        
        // 🟢 修复点：让每个分支都直接返回 String (拥有所有权的值)
        // 这样就不存在引用的生命周期问题了
        let model_clean = match model_upper.as_str() {
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
            
            // 兜底逻辑：直接生成 String 并返回
            _ => raw.model.replace("ILCE-", "").trim().to_string(),
        }; 
        // ⬆️ 注意：这里不需要再 .to_string() 了，因为 match 内部已经全部转成 String 了

        build_context(Brand::Sony, model_clean, raw)
    }
}

// ==========================================
// 3. Canon 解析器
// ==========================================
pub struct CanonParser;

impl BrandParser for CanonParser {
    fn can_parse(&self, raw: &RawExifData) -> bool {
        raw.make.to_uppercase().contains("CANON")
    }

    fn parse(&self, raw: &RawExifData) -> ParsedImageContext {
        // 清洗: "Canon EOS R5" -> "EOS R5"
        let model_clean = raw.model.replace("Canon", "").trim().to_string();

        build_context(Brand::Canon, model_clean, raw)
    }
}

// ==========================================
// 🛠️ 私有辅助函数 (减少重复代码)
// ==========================================
fn build_context(brand: Brand, model_clean: String, raw: &RawExifData) -> ParsedImageContext {
    // 1. 作者策略: Artist > Copyright
    let final_artist = raw.artist.clone()
        .or_else(|| raw.copyright.clone())
        .filter(|s| !s.trim().is_empty());

    // 2. 时间格式: 2023:10:01 -> 2023.10.01
    let clean_time = raw.datetime.replace(":", ".");

    // 3. GPS 转换
    let gps_data = if let (Some(lat), Some(long)) = (raw.gps_latitude, raw.gps_longitude) {
        Some(GeoLocation { latitude: lat, longitude: long })
    } else {
        None
    };

    ParsedImageContext {
        brand,
        model_name: model_clean,
        params: ShootingParams {
            iso: raw.iso,
            aperture: raw.aperture,
            shutter_speed: raw.shutter_speed.clone(),
            focal_length: raw.focal_length,
            lens_model: raw.lens.clone(),
            capture_time: clean_time,
        },
        artist_name: final_artist,
        gps: gps_data,
    }
}