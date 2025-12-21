use serde::Serialize;
use std::fs;
use std::io::BufReader;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMetadata {
    pub model: String,
    pub f_number: String,
    pub exposure_time: String,
    pub iso: String,
    pub focal_length: String,
}

// 🟢 修改返回值：(Make, Model, Params)
pub fn get_exif_string_tuple(path: &str) -> (String, String, String) {
    let default = ("".to_string(), "".to_string(), "".to_string());
    
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return default,
    };
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif_data = match exifreader.read_from_container(&mut bufreader) {
        Ok(d) => d,
        Err(_) => return default,
    };

    let get = |tag| match exif_data.get_field(tag, exif::In::PRIMARY) {
        Some(f) => f.display_value().with_unit(&exif_data).to_string().replace("\"", "").trim().to_string(),
        None => "".to_string(),
    };

    // 1. 厂商 (用于匹配 Logo)
    let make = get(exif::Tag::Make);

    // 2. 型号 (用于显示)
    let model = get(exif::Tag::Model);
    
    // 3. 参数拼接
    let mut params = Vec::new();
    
    let fl = get(exif::Tag::FocalLength);
    if !fl.is_empty() { params.push(fl); }
    
    let f = get(exif::Tag::FNumber);
    if !f.is_empty() { params.push(f); }

    let t = get(exif::Tag::ExposureTime);
    if !t.is_empty() { params.push(t); }

    let iso = get(exif::Tag::PhotographicSensitivity);
    if !iso.is_empty() { params.push(format!("ISO {}", iso)); }

    (make, model, params.join("  "))
}