// src/parser/impls.rs

use crate::resources::Brand;
use crate::parser::models::{RawExifData, ParsedImageContext, ShootingParams, GeoLocation};
use crate::parser::traits::BrandParser;

// 🟢 引入刚才在 mod.rs 里定义的清洗函数
use super::clean_model_name_logic;

// ==========================================
// 1. Nikon 解析器
// ==========================================
pub struct NikonParser;
impl BrandParser for NikonParser {
    fn can_parse(&self, raw: &RawExifData) -> bool {
        raw.make.to_uppercase().contains("NIKON")
    }
    fn parse(&self, raw: &RawExifData) -> ParsedImageContext {
        // 🟢 直接调用通用清洗 -> 得到 "Z 8"
        let model_clean = clean_model_name_logic(&raw.make, &raw.model);
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
        // 🟢 直接调用通用清洗 -> 得到 "α7R V"
        // 删掉这里原来那一长串 match，逻辑已移至 mod.rs
        let model_clean = clean_model_name_logic(&raw.make, &raw.model);
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
        // 🟢 直接调用通用清洗 -> 得到 "EOS R5"
        let model_clean = clean_model_name_logic(&raw.make, &raw.model);
        build_context(Brand::Canon, model_clean, raw)
    }
}

// ... (build_context 辅助函数保持不变) ...
fn build_context(brand: Brand, model_clean: String, raw: &RawExifData) -> ParsedImageContext {
    // ... (保持原样) ...
    let clean_time = raw.datetime.replace(":", ".");
    
    // ... GPS 逻辑 ...
    let gps_data = if let (Some(lat), Some(long)) = (raw.gps_latitude, raw.gps_longitude) {
       Some(GeoLocation { latitude: lat, longitude: long })
    } else { None };

    ParsedImageContext {
        brand,
        model_name: model_clean, // 这里传入的已经是清洗完美的名字
        params: ShootingParams {
            iso: raw.iso,
            aperture: raw.aperture,
            shutter_speed: raw.shutter_speed.clone(),
            focal_length: raw.focal_length,
            lens_model: raw.lens.clone(),
            capture_time: clean_time,
        },
        artist_name: raw.artist.clone().or(raw.copyright.clone()),
        gps: gps_data,
    }
}