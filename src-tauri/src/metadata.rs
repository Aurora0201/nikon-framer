use std::fs;
use std::io::BufReader;
use std::fs::File;
use exif::{In, Reader, Tag, Value};
use crate::parser::models::RawExifData; // 引入我们定义的数据结构


/// 读取文件 EXIF 并填充 RawExifData
pub fn get_exif_data(path: &str) -> RawExifData {
    // 1. 尝试打开文件
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return RawExifData::default(),
    };

    // 2. 读取 EXIF
    let mut reader = BufReader::new(file);
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(e) => e,
        Err(_) => return RawExifData::default(),
    };

    // --- 辅助闭包：获取字符串值 ---
    let get_text = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .map(|f| f.display_value().with_unit(&exif).to_string())
            .unwrap_or_default()
            .replace("\"", "") // 去掉可能存在的引号
            .trim()
            .to_string()
    };

    // --- 辅助闭包：获取 u32 (ISO, 焦距) ---
    let get_u32 = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
    };

    // --- 辅助闭包：获取 f32 (光圈) ---
    let get_f32 = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| match &f.value {
                // 1. 无符号分数 (Type 5: Rational) -> num/denom 都是 u32
                Value::Rational(v) if !v.is_empty() => {
                    let r = &v[0];
                    if r.denom == 0 {
                        None
                    } else {
                        Some(r.num as f32 / r.denom as f32)
                    }
                },
                
                // 2. 有符号分数 (Type 10: SRational) -> num/denom 都是 i32
                // 🟢 之前报错是因为写成了 UnsignedRational，实际上应该是 SRational
                Value::SRational(v) if !v.is_empty() => {
                    let r = &v[0];
                    if r.denom == 0 {
                        None
                    } else {
                        Some(r.num as f32 / r.denom as f32)
                    }
                },

                // 3. 浮点数 (Type 11: Float)
                Value::Float(v) if !v.is_empty() => Some(v[0]),
                
                // 4. 双精度浮点 (Type 12: Double) - 为了保险起见加上
                Value::Double(v) if !v.is_empty() => Some(v[0] as f32),
                
                _ => None
            })
    };

    // --- 辅助闭包：解析 GPS ---
    // 这是一个简化实现，如果需要高精度转换，需要把度分秒转十进制
    // 这里暂时留空或者返回 None，视你引用的 exif 库版本支持情况而定
    // 为了不报错，我们暂时返回 None，稍后可以专门加一个 GPS 转换函数
    let lat = None; 
    let long = None;

    RawExifData {
        make: get_text(Tag::Make),
        model: get_text(Tag::Model),
        lens: get_text(Tag::LensModel),
        
        iso: get_u32(Tag::PhotographicSensitivity), // ISO
        aperture: get_f32(Tag::FNumber),            // 光圈
        shutter_speed: get_text(Tag::ExposureTime), // 快门 (保留字符串，因为 1/8000 比小数直观)
        focal_length: get_u32(Tag::FocalLengthIn35mmFilm) // 优先用等效焦距
            .or_else(|| get_u32(Tag::FocalLength)),       // 没有就用物理焦距
            
        datetime: get_text(Tag::DateTimeOriginal),
        artist: Some(get_text(Tag::Artist)),
        copyright: Some(get_text(Tag::Copyright)),
        
        gps_latitude: lat,
        gps_longitude: long,
    }
}


// 🟢 [新增] 快速检查是否存在 EXIF
pub fn has_exif(path: &str) -> bool {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    // 只要能读到 header 就算成功，不需要解析具体字段
    exifreader.read_from_container(&mut bufreader).is_ok()
}

// 🟢 [新增] 批量过滤：只保留文件，剔除文件夹
#[tauri::command]
pub fn filter_files(paths: Vec<String>) -> Vec<String> {
    paths.into_iter()
        .filter(|path| {
            // 获取元数据，检查 is_file()
            match fs::metadata(path) {
                Ok(meta) => meta.is_file(),
                Err(_) => false, // 无法读取的文件也过滤掉
            }
        })
        .collect()
}

#[tauri::command]
pub fn scan_folder(folder_path: String) -> Vec<String> {
    let allowed_exts = vec!["jpg", "jpeg", "png", "nef", "arw", "dng"];
    let mut image_paths = Vec::new();

    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // 只处理文件，忽略子文件夹（如果不希望递归的话）
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        // 转小写进行比对
                        if allowed_exts.contains(&ext_str.to_lowercase().as_str()) {
                            if let Some(path_str) = path.to_str() {
                                image_paths.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    image_paths
}