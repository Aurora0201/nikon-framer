use tauri::{State, Window, Emitter};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}}; // 🟢 新增 AtomicUsize
use std::time::Instant;
use std::path::Path;
use std::fs::File; // 🟢 需要引入
use std::io::BufReader; // 🟢 需要引入
use crate::models::BatchContext;
use crate::state::AppState;
use crate::{processor, metadata}; 
use rayon::prelude::*; // 🟢 必须引入
use crate::parser;
use image::{self, DynamicImage, imageops}; // 🟢 引入 imageops

// =========================================================
// 🟢 新增：优雅的加载函数 (Private Helper)
// 职责单一：打开图片，如果有EXIF方向标记，就自动旋转摆正
// =========================================================
fn load_image_auto_rotate(path: &str) -> Result<DynamicImage, String> {
    // 1. 先尝试标准打开
    let mut img = image::open(path).map_err(|e| e.to_string())?;

    // 2. 偷看一眼 EXIF 方向
    if let Ok(file) = File::open(path) {
        let mut bufreader = BufReader::new(&file);
        let exifreader = exif::Reader::new();
        
        if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
            if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                if let Some(orientation) = field.value.get_uint(0) {
                    // 🟢 修复：将 ImageBuffer 包装回 DynamicImage
                    img = match orientation {
                        3 => DynamicImage::ImageRgba8(imageops::rotate180(&img)),
                        6 => DynamicImage::ImageRgba8(imageops::rotate90(&img)),
                        8 => DynamicImage::ImageRgba8(imageops::rotate270(&img)),
                        _ => img, // 这个本身就是 DynamicImage，无需包装
                    };
                }
            }
        }
    }

    Ok(img)
}

#[tauri::command]
pub async fn start_batch_process_v2(
    window: Window,
    state: State<'_, Arc<AppState>>,
    file_paths: Vec<String>,
    context: BatchContext,
) -> Result<String, String> {
    
    println!("🚀 [API V2] 启动并行批处理 ({} 个文件)", file_paths.len());
    
    // 1. 获取主线程用的 Arc
    let state_arc = state.inner().clone();
    state_arc.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();

    // 克隆给线程用的变量
    let state_for_thread = state_arc.clone();
    let window_for_thread = window.clone();
    
    let suffix = context.options.filename_suffix(); 
    let suffix_arc = Arc::new(suffix.to_string());

    // 创建处理器 (此时创建的是支持 ctx 的新版处理器)
    let processor_strategy = processor::create_processor(&context.options);
    let processor_arc = Arc::new(processor_strategy);

    let completed_count = Arc::new(AtomicUsize::new(0));

    // 放入线程池
    let result = tauri::async_runtime::spawn_blocking(move || {
        
        file_paths.par_iter().for_each(|file_path| {
            
            // 🛑 检查停止标志
            if state_for_thread.should_stop.load(Ordering::Relaxed) {
                return;
            }

            // 1. EXIF 预检查 (快速过滤)
            if !metadata::has_exif(file_path) {
                let current = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
                let _ = window_for_thread.emit("process-progress", serde_json::json!({
                    "current": current, "total": total_files, "filepath": file_path, "status": "skipped"
                }));
                return;
            }

           // =========================================================
            // 🟢 修改点：使用新函数替代 image::open
            // =========================================================
            let img = match load_image_auto_rotate(file_path) {
                Ok(i) => i,
                Err(e) => {
                    println!("❌ 无法打开: {} -> {}", file_path, e);
                    return; 
                }
            };
            // =========================================================
            
            // =========================================================
            // 🟢 核心重构区域 START
            // =========================================================
            
            // A. 读取原始数据 (Raw Data)
            let raw_exif = metadata::get_exif_data(file_path);

            // B. 智能解析与清洗 (Parsing)
            // 这里会处理 "NIKON Z 8" -> "Z 8"，以及 "2023:12:30" -> "2023.12.30"
            let parsed_ctx = parser::parse(raw_exif);

            // C. 绘图处理 (Drawing)
            // 将清洗好的 ctx 传给处理器
            let processor_ref = &processor_arc; 
            let final_image = match processor_ref.process(&img, &parsed_ctx) {
                Ok(img) => img,
                Err(e) => {
                    println!("❌ 处理失败: {} -> {}", file_path, e);
                    return;
                }
            };

            // =========================================================
            // 🟢 核心重构区域 END
            // =========================================================

            // 3. 保存文件
            let suffix_ref = &suffix_arc;
            let path_obj = Path::new(file_path);
            let parent = path_obj.parent().unwrap_or(Path::new("."));
            let file_stem = path_obj.file_stem().unwrap().to_string_lossy();
            
            let new_filename = format!("{}_{}.jpg", file_stem, suffix_ref);
            let output_path = parent.join(new_filename);

            if let Err(e) = final_image.save(&output_path) {
                println!("❌ 保存失败: {}", e);
                return;
            }

            // 4. 发送进度
            let current = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = window_for_thread.emit("process-progress", serde_json::json!({
                "current": current,
                "total": total_files,
                "filepath": file_path,
                "status": "processing"
            }));
        });
    }).await;

    // 错误处理与结束状态
    if let Err(e) = result {
        return Err(format!("Thread pool error: {}", e));
    }

    let duration = batch_start.elapsed();
    
    if state_arc.should_stop.load(Ordering::Relaxed) {
        window.emit("process-status", "stopped").map_err(|e| e.to_string())?;
        return Ok("Stopped by user".to_string());
    }

    println!("✨ [API V2] 并行批处理全部完成，耗时: {:.2?}", duration);
    window.emit("process-status", "finished").map_err(|e| e.to_string())?;

    Ok(format!("Batch processing complete in {:.2?}", duration))
}