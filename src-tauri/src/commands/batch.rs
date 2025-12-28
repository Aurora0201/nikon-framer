use tauri::{State, Window, Emitter};
use std::sync::{Arc, atomic::Ordering};
use std::time::Instant;
use std::path::Path;
use crate::models::BatchContext;
use crate::state::AppState;
use crate::{processor, metadata}; // 引用根模块

#[tauri::command]
pub async fn start_batch_process_v2(
    window: Window,
    state: State<'_, Arc<AppState>>,
    file_paths: Vec<String>,
    context: BatchContext,
) -> Result<String, String> {
    
    // 1. 初始化状态
    println!("🚀 [API V2] 启动批处理 ({} 个文件)", file_paths.len());
    state.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();

    // 2. 创建处理器 (策略模式)
    // 🟢 关键：使用 Arc 包裹 Box，以便在循环的线程中共享引用
    let processor_strategy = processor::create_processor(&context.options);
    let processor_arc = Arc::new(processor_strategy);

    for (index, file_path) in file_paths.iter().enumerate() {
        // --- A. 检查停止信号 ---
        if state.should_stop.load(Ordering::Relaxed) {
            window.emit("process-status", "stopped").map_err(|e| e.to_string())?;
            return Ok("Stopped by user".to_string());
        }

        // --- B. EXIF 预检查 ---
        if !metadata::has_exif(file_path) {
            // 发送跳过事件
            window.emit("process-progress", serde_json::json!({
                "current": index + 1,
                "total": total_files,
                "filepath": file_path,
                "status": "skipped"
            })).ok(); // 忽略发送失败
            continue;
        }

        // --- C. 准备线程所需数据 ---
        let path_clone = file_path.clone();
        // 克隆 Arc 引用计数，开销极小
        let processor_ref = processor_arc.clone(); 

        // --- D. 放入线程池执行 (Heavy Lifting) ---
        let result = tauri::async_runtime::spawn_blocking(move || {
            // 1. 打开图片
            let img = image::open(&path_clone).map_err(|e| format!("无法打开图片: {}", e))?;
            
            // 2. 获取元数据
            let (make, model, params) = metadata::get_exif_string_tuple(&path_clone);

            // 3. 调用多态接口处理图片
            // processor_ref 会自动根据之前的 create_processor 逻辑调用对应实现
            let final_image = processor_ref.process(&img, &make, &model, &params)?;

            // 4. 保存图片逻辑
            let path_obj = Path::new(&path_clone);
            let parent = path_obj.parent().unwrap_or(Path::new("."));
            let file_stem = path_obj.file_stem().unwrap().to_string_lossy();
            
            // 这里可以做一个简单的优化：根据不同的 style 生成不同的后缀
            // 但因为 processor_ref 是 dyn Trait，获取 style 名字比较麻烦，
            // 简单起见，可以暂时统一后缀，或者在 Trait 里加一个 get_suffix() 方法。
            // 这里我们简单使用 "_framed.jpg"
            let new_filename = format!("{}_framed.jpg", file_stem);
            let output_path = parent.join(new_filename);

            final_image.save(&output_path).map_err(|e| format!("保存失败: {}", e))?;

            Ok::<String, String>(output_path.to_string_lossy().to_string())
        }).await;

        // --- E. 处理结果与 UI 反馈 ---
        match result {
            Ok(Ok(saved_path)) => {
                println!("✅ 完成: {}", saved_path);
                // 发送成功进度
                window.emit("process-progress", serde_json::json!({
                    "current": index + 1,
                    "total": total_files,
                    "filepath": file_path,
                    "status": "processing"
                })).map_err(|e| e.to_string())?;
            },
            Ok(Err(e)) => {
                println!("❌ 处理错误: {}", e);
                // 可以选择发送错误事件，或者仅打印日志
            },
            Err(e) => println!("❌ 线程崩溃: {}", e),
        }
    }

    let duration = batch_start.elapsed();
    println!("✨ [API V2] 批处理全部完成，耗时: {:.2?}", duration);
    
    // 发送完成信号
    window.emit("process-status", "finished").map_err(|e| e.to_string())?;

    Ok(format!("Batch processing complete in {:.2?}", duration))
}