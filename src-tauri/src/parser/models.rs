// src/parser/models.rs
use serde::{Serialize, Deserialize}; // 🟢 引入这个
use crate::resources::Brand;

// 🟢 1. 原始数据 (从文件读取的脏数据)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawExifData {
    pub make: String,
    pub model: String,
    pub lens: String,
    
    // 拍摄参数
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: String,
    pub focal_length: Option<u32>,
    
    // 时间与作者
    pub datetime: String,
    pub artist: Option<String>,
    pub copyright: Option<String>,

    // 🟢 新增：GPS 原始数据
    // EXIF 库通常能直接给出 f64 (十进制) 的经纬度，
    // 如果库给的是度分秒(Rational)，我们需要在 metadata 层就转好，或者在这里存原始值
    // 假设 kamadak-exif 或类似库已经帮我们处理了一部分，或者我们读取 lat/long 的 f64 值
    pub gps_latitude: Option<f64>,  // e.g. 35.6895
    pub gps_longitude: Option<f64>, // e.g. 139.6917
}


// 🟢 2. 拍摄参数 (纯物理数据)
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShootingParams {
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: String,
    pub focal_length: Option<u32>,
    pub lens_model: String,
    
    pub capture_time: String, // "2023.12.30 14:00"
}

impl ShootingParams {
    /// 辅助函数：生成标准的参数字符串 (e.g. "50mm f/1.8 1/800s ISO 100")
    /// 供那些不需要自定义排版的相框直接使用
    pub fn format_standard(&self) -> String {
        let mut parts = Vec::new();

        // 焦距
        if let Some(f) = self.focal_length {
            parts.push(format!("{}mm", f));
        }

        // 光圈
        if let Some(a) = self.aperture {
            parts.push(format!("f/{}", a));
        }

        // 快门 (直接用字符串，因为已经是清洗过的)
        if !self.shutter_speed.is_empty() {
            parts.push(self.shutter_speed.clone());
        }

        // ISO
        if let Some(iso) = self.iso {
            parts.push(format!("ISO {}", iso));
        }

        parts.join("  ") // 用双空格分隔，视觉上更清晰
    }
}


// 🟢 3. GPS 信息结构体
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    // 未来可扩展：
    // pub location_name: Option<String>, // "Tokyo, Japan" (如果做了逆地理编码)
}

#[allow(dead_code)]
impl GeoLocation {
    // 辅助方法：格式化为字符串 "35°41'N 139°41'E"
    pub fn format_dms(&self) -> String {
        // 这里可以实现一个简单的算法把小数转度分秒
        format!("{:.4}, {:.4}", self.latitude, self.longitude) // 暂时简单返回
    }
}

// 🟢 4. 最终上下文 (The Clean Context)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedImageContext {
    pub brand: Brand,
    pub model_name: String,      // "Z 8"
    pub params: ShootingParams,
    
    pub artist_name: Option<String>, 
    
    // 🟢 新增 GPS (Option，因为很多照片没开定位)
    pub gps: Option<GeoLocation>,
}

