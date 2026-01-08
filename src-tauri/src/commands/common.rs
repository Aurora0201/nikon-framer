use tauri::State;
use std::{ sync::{Arc, atomic::Ordering}};
use crate::{graphics::load_image_auto_rotate, models::{BatchContext, ExportConfig, StyleOptions}, state::AppState, utils::calculate_target_path_core};
use crate::metadata; // 引用 crate::metadata
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
    style_options: StyleOptions, 
    export_config: ExportConfig, 
) -> Option<String> {

    // 1. 🟢 核心升级：调用统一的路径计算逻辑 (Single Source of Truth)
    // ---------------------------------------------------------
    // 不再手动拼接 target_parent, suffix, ext
    // 直接问 models: "根据这些配置，目标文件应该在哪？"
    let target_path = match calculate_target_path_core(&file_path, &export_config, &style_options) {
        Ok(p) => p,
        Err(e) => {
            // 如果路径都算不出来（比如文件名非法），那文件肯定不存在
            println!("路径计算错误: {}", e);
            return None;
        }
    };

    // 2. 检查文件是否存在
    // ---------------------------------------------------------
    if !target_path.exists() {
        return None;
    }

    // 3. 读取 -> 缩放 -> 转 Base64
    // ---------------------------------------------------------
    let target_path_str = target_path.to_str()?;

    // 复用 load_and_resize_blob
    match load_and_resize_blob(target_path_str, 1000) {
        Ok(buffer) => {
            let b64 = general_purpose::STANDARD.encode(&buffer);
            
            // 🟢 核心升级：从 export_config.format 获取 MIME 类型
            // 不再写死 if ext == "png" ...
            let mime = export_config.format.mime_type();
            
            Some(format!("data:{};base64,{}", mime, b64))
        },
        Err(e) => {
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
    context: BatchContext 
) -> Vec<String> {
    println!("🔍 [Filter] 开始检查 {} 个文件...", paths.len());

    // 1. OCP: 检查可编辑模式
    if context.options.is_editable() {
        println!("⚡ [Filter] 检测到可编辑模式 ({:?})，强制全量处理。", context.options);
        return paths;
    }

    let mut to_process = Vec::new();
    let mut skipped_count = 0;
    let mut error_count = 0;

    for path_str in &paths {
        // 🟢 2. 调用统一路径计算逻辑
        match context.calculate_target_path(path_str) {
            Ok(target_path) => {
                if target_path.exists() {
                    // 文件存在，跳过
                    skipped_count += 1;
                    // 可选：如果需要调试，可以打印跳过了谁
                    // println!("  -> 跳过已存在: {:?}", target_path);
                } else {
                    // 文件不存在，加入待处理列表
                    to_process.push(path_str.clone());
                }
            },
            Err(e) => {
                // 🔴 错误处理：路径计算失败（极少发生），但也需要记录
                eprintln!("⚠️ [Filter] 路径计算错误 [{}]: {}", path_str, e);
                // 策略：如果算不出目标路径，为了保险起见，建议加入待处理列表，或者跳过
                // 这里选择加入，让 pipeline 去处理并报错，避免静默失败
                to_process.push(path_str.clone());
                error_count += 1;
            }
        }
    }

    println!(
        "✅ [Filter] 完成: 输入 {} -> 需处理 {} (跳过 {}, 异常 {})", 
        paths.len(), to_process.len(), skipped_count, error_count
    );
    
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
pub async fn generate_thumbnail(file_path: String) -> Result<String, String> {
    // 🟢 使用 spawn_blocking 将计算密集型任务扔到专用线程池，防止阻塞 Tauri 主循环
    let result = tauri::async_runtime::spawn_blocking(move || {
        // 这里放所有的重型操作：读取、解码、缩放、Base64编码
        let bytes = load_and_resize_blob(&file_path, 200)?;
        let b64 = general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:image/jpeg;base64,{}", b64))
    }).await;

    // 处理 Result<Result<...>> 的嵌套解包
    result.map_err(|e| e.to_string())?
}