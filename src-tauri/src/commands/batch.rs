use tauri::{State, Window, Emitter};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}}; // 🟢 新增 AtomicUsize
use std::time::Instant;
use std::path::Path;
use crate::models::BatchContext;
use crate::state::AppState;
use crate::{processor, metadata}; 
use rayon::prelude::*; // 🟢 必须引入

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
    // 重置停止标志
    state_arc.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();

    // 🟢 关键修正点 1：专门克隆一份给后台线程用 (命名为 _thread)
    // 这样原始的 state_arc 就不会被 move 走，函数最后还能用
    let state_for_thread = state_arc.clone();
    let window_for_thread = window.clone();

    // 准备其他共享数据
    let suffix = context.options.filename_suffix(); 
    let suffix_arc = Arc::new(suffix.to_string());

    let processor_strategy = processor::create_processor(&context.options);
    let processor_arc = Arc::new(processor_strategy);

    let completed_count = Arc::new(AtomicUsize::new(0));

    // 放入线程池
    let result = tauri::async_runtime::spawn_blocking(move || {
        // 🟢 关键修正点 2：闭包里只使用 _for_thread 版本的变量
        
        file_paths.par_iter().for_each(|file_path| {
            
            // 使用 state_for_thread
            if state_for_thread.should_stop.load(Ordering::Relaxed) {
                return;
            }

            // EXIF 预检查
            if !metadata::has_exif(file_path) {
                let current = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
                // 使用 window_for_thread
                let _ = window_for_thread.emit("process-progress", serde_json::json!({
                    "current": current,
                    "total": total_files,
                    "filepath": file_path,
                    "status": "skipped"
                }));
                return;
            }

            // ... (中间的处理逻辑保持不变) ...
            let processor_ref = &processor_arc; 
            let suffix_ref = &suffix_arc;

            let img = match image::open(file_path) {
                Ok(i) => i,
                Err(e) => {
                    println!("❌ 无法打开: {} -> {}", file_path, e);
                    return; 
                }
            };
            
            let (make, model, params) = metadata::get_exif_string_tuple(file_path);

            let final_image = match processor_ref.process(&img, &make, &model, &params) {
                Ok(img) => img,
                Err(e) => {
                    println!("❌ 处理失败: {} -> {}", file_path, e);
                    return;
                }
            };

            let path_obj = Path::new(file_path);
            let parent = path_obj.parent().unwrap_or(Path::new("."));
            let file_stem = path_obj.file_stem().unwrap().to_string_lossy();
            
            let new_filename = format!("{}_{}.jpg", file_stem, suffix_ref);
            let output_path = parent.join(new_filename);

            if let Err(e) = final_image.save(&output_path) {
                println!("❌ 保存失败: {}", e);
                return;
            }

            // 发送成功进度 (使用 window_for_thread)
            let current = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
            
            let _ = window_for_thread.emit("process-progress", serde_json::json!({
                "current": current,
                "total": total_files,
                "filepath": file_path,
                "status": "processing"
            }));
        });
    }).await;

    // 检查线程池结果
    if let Err(e) = result {
        println!("❌ 线程池异常: {}", e);
        return Err(format!("Thread pool error: {}", e));
    }

    let duration = batch_start.elapsed();
    
    // 🟢 关键修正点 3：这里现在可以使用 state_arc 了
    // 因为移动进闭包的是 state_for_thread，state_arc 依然在当前作用域有效
    if state_arc.should_stop.load(Ordering::Relaxed) {
        window.emit("process-status", "stopped").map_err(|e| e.to_string())?;
        return Ok("Stopped by user".to_string());
    }

    println!("✨ [API V2] 并行批处理全部完成，耗时: {:.2?}", duration);
    window.emit("process-status", "finished").map_err(|e| e.to_string())?;

    Ok(format!("Batch processing complete in {:.2?}", duration))
}