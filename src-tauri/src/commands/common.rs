use tauri::State;
use std::{sync::{Arc, atomic::Ordering}};
use crate::{graphics::load_image_auto_rotate, models::BatchContext, state::AppState};
use crate::metadata; // 引用 crate::metadata
use std::path::Path;
use std::io::Cursor;
use image::{ImageFormat};
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
) -> Option<String> {
    
    // 1. 计算目标路径 (逻辑保持不变)
    // ---------------------------------------------------------
    // 这里有一点防御性编程：如果路径解析失败直接返回 None
    let path_obj = Path::new(&file_path);
    let parent = path_obj.parent()?;
    let file_stem = path_obj.file_stem()?.to_string_lossy();
    
    // 根据命名规则拼接目标文件名
    let suffix = format!("_{}", style);
    let target_filename = format!("{}{}.jpg", file_stem, suffix);
    let target_path = parent.join(target_filename);

    // 2. 检查文件是否存在
    // ---------------------------------------------------------
    if !target_path.exists() {
        return None;
    }

    // 3. 🟢 [复用核心] 调用通用函数获取二进制数据
    // ---------------------------------------------------------
    // 将 PathBuf 转为 &str
    let target_path_str = target_path.to_str()?;

    // 复用 load_and_resize_blob
    // 这里的 1000 是 max_dimension，用于预览图刚好合适
    match load_and_resize_blob(target_path_str, 1000) {
        Ok(buffer) => {
            // 4. 转 Base64 (前端 img 标签直接显示需要)
            // ---------------------------------------------------------
            let b64 = general_purpose::STANDARD.encode(&buffer);
            
            // 返回完整的 Data URL
            Some(format!("data:image/jpeg;base64,{}", b64))
        },
        Err(e) => {
            // 虽然文件存在，但读取或解码失败（可能是文件损坏）
            println!("⚠️ 预览图加载失败 [{}]: {}", target_path_str, e);
            None
        }
    }
}


// 🟢 新增：批量过滤未处理的文件
// 输入：所有待处理的文件路径列表 + 当前样式 ID
// 输出：仅返回那些“硬盘上还不存在结果图”的文件路径
#[tauri::command]
pub fn filter_unprocessed_files(
    paths: Vec<String>, 
    // 🟢 修改：这里接收完整的 Context JSON，而不是简单的 style string
    // 这样我们就能反序列化出 StyleOptions 枚举，从而调用 is_editable()
    context: BatchContext 
) -> Vec<String> {
    
    // 1. 检查是否为可编辑模式
    if context.options.is_editable() {
        println!("⚡ [Filter] 检测到可编辑模式 ({:?})，跳过重复检查，强制全量处理。", context.options);
        return paths; // 直接把所有路径原样返回
    }

    // 2. 如果是静态模式，执行原来的检查逻辑
    let suffix = context.options.filename_suffix();
    let mut to_process = Vec::new();

    // 🔴 修复点：使用 &paths 进行借用迭代，而不是消耗所有权
    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        let file_stem = path.file_stem().unwrap().to_string_lossy();
        let target_name = format!("{}_{}.jpg", file_stem, suffix);
        let target_path = parent.join(target_name);

        if !target_path.exists() {
            // 🟢 因为 path_str 现在只是一个借来的引用，
            // 我们需要 clone() 一份放进新的 Vec 里
            to_process.push(path_str.clone());
        }
    }

    println!("🔍 [Filter] 过滤完成: 输入 {} -> 输出 {}", paths.len(), to_process.len());
    to_process
}



/// 🔒 内部通用函数：读取 -> 旋转 -> 缩放 -> 编码
fn load_and_resize_blob(file_path: &str, max_dimension: u32) -> Result<Vec<u8>, String> {
    
    // 1. 复用之前的逻辑：加载并自动旋转
    let img = load_image_auto_rotate(file_path)?;

    // 2. 智能缩放
    // 🟢 优化点：使用 .thumbnail() 而不是 .resize()
    // thumbnail 会自动保持长宽比，并且针对"缩小"场景有极大的性能优化
    // (它内部会先进行快速降采样，然后再精细缩放，比直接用 Lanczos3 算全图快得多)
    let resized_img = img.thumbnail(max_dimension, max_dimension);

    // 3. 编码为 JPEG
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    // 缩略图质量设为默认 (约 75) 即可，足够清晰且体积小
    resized_img.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|e| format!("图片编码失败: {}", e))?;

    Ok(buffer)
}

/// 读取本地图片，**自动矫正EXIF方向**，缩放并转换为 JPEG Blob
#[tauri::command]
pub fn read_local_image_blob(file_path: String) -> Result<Vec<u8>, String> {

    // 维持原有的 1600px 逻辑
    load_and_resize_blob(&file_path, 1600)
}

/// 🖼️ 新增 API：用于"文件列表"的缩略图 (限制 200px)
/// 200px 足够支持 Retina 屏幕下的列表显示和悬停放大
#[tauri::command]
pub fn generate_thumbnail(file_path: String) -> Result<Vec<u8>, String> {
    // 200px 既能满足列表(48px)的高清显示，也能满足悬停放大(200px)的需求
    // 且生成的 Blob 大小通常只有几 KB，加载飞快
    load_and_resize_blob(&file_path, 200)
}