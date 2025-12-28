use tauri::State;
use std::sync::{Arc, atomic::Ordering};
use crate::state::AppState;
use crate::metadata; // 引用 crate::metadata
use std::path::Path;
use std::io::Cursor;
use image::ImageFormat;
use base64::{Engine as _, engine::general_purpose};

#[tauri::command]
pub fn check_file_exif(path: String) -> bool {
    metadata::has_exif(&path)
}

#[tauri::command]
pub fn stop_batch_process(state: State<'_, Arc<AppState>>) {
    println!("🛑 收到停止指令...");
    state.should_stop.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn check_output_exists(
    file_path: String,
    style: String
) -> Option<String> { // 返回值 Option<String> 现在代表 Base64 字符串
    
    // 1. 计算目标路径 (和你之前的逻辑一样)
    let suffix = format!("_{}", style);
    let path_obj = Path::new(&file_path);
    let parent = path_obj.parent().unwrap_or(Path::new("."));
    let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let target_filename = format!("{}{}.jpg", file_stem, suffix);
    let target_path = parent.join(target_filename);

    // 2. 检查文件是否存在
    if !target_path.exists() {
        return None;
    }

    // 3. 🟢 [核心修改] 读取 -> 缩放 -> 转 Base64
    // 不直接返回路径，而是返回图片数据
    match image::open(&target_path) {
        Ok(img) => {
            // A. 缩放图片 (性能关键！只用来预览不需要全尺寸)
            // 假设预览框最大也就 1000px 宽，这样生成的字符串很小，传输极快
            let resized = img.thumbnail(1000, 1000); 

            // B. 写入内存 buffer
            let mut buffer = Vec::new();
            // 存为 JPEG 格式，质量 80，进一步减小体积
            if let Err(_) = resized.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg) {
                return None;
            }

            // C. 转 Base64
            let b64 = general_purpose::STANDARD.encode(&buffer);
            
            // D. 返回带前缀的完整 Data URL
            Some(format!("data:image/jpeg;base64,{}", b64))
        },
        Err(e) => {
            println!("读取预览图失败: {}", e);
            None
        }
    }
}