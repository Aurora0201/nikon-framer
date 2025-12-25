use std::fs;
use std::io::BufReader;



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