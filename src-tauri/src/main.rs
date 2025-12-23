// 声明模块
mod metadata;
mod resources;
mod processor;
mod debug;
mod graphics;

use metadata::PhotoMetadata;
use std::time::Instant; // 🟢 移除了未使用的 Duration
use std::sync::{Arc, atomic::{AtomicBool, Ordering}}; 
// 🟢 下面这行是关键修复：引入了 Emitter，移除了未使用的 Manager
use tauri::{State, Window, Emitter}; 

// --- State Management ---
struct AppState {
    should_stop: AtomicBool,
}

// --- Tauri Commands ---

#[tauri::command]
fn get_font_list() -> Vec<String> {
    resources::get_font_list()
}

#[tauri::command]
fn check_file_exif(path: String) -> bool {
    metadata::has_exif(&path)
}

#[tauri::command]
fn read_photo_metadata(file_path: String) -> Result<PhotoMetadata, String> {
    let start = Instant::now();

    let (make, model, params) = metadata::get_exif_string_tuple(&file_path);
    
    let display_model = if model.to_uppercase().starts_with(&make.to_uppercase()) {
        model.clone()
    } else {
        format!("{} {}", make, model)
    };

    println!("🚀 [PERF] 元数据读取耗时: {:.2?}", start.elapsed());

    Ok(PhotoMetadata {
        model: display_model,
        f_number: "See Params".to_string(),
        exposure_time: "See Params".to_string(),
        iso: params, 
        focal_length: "".to_string(),
    })
}

// --- 停止批处理命令 ---
#[tauri::command]
fn stop_batch_process(state: State<'_, Arc<AppState>>) {
    println!("🛑 收到停止指令...");
    state.should_stop.store(true, Ordering::Relaxed);
}

// --- 开始批处理命令 ---
#[tauri::command]
async fn start_batch_process(
    window: Window, 
    state: State<'_, Arc<AppState>>, 
    file_paths: Vec<String>, 
    style: String,
    font_filename: String,
    font_weight: String,
    shadow_intensity: f32
) -> Result<String, String> {
    
    // 1. 重置停止标志
    state.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();

    println!("================ 批处理开始 (总数: {}) ================", total_files);

    for (index, file_path) in file_paths.iter().enumerate() {
        // 2. 检查是否收到停止信号
        if state.should_stop.load(Ordering::Relaxed) {
            // 修复点：引入 Emitter 后，这里的 emit 方法就能找到了
            window.emit("process-status", "stopped").map_err(|e| e.to_string())?;
            return Ok("Batch processing stopped by user".to_string());
        }

        // 3. 过滤无 EXIF 文件
        if !metadata::has_exif(file_path) {
            println!("⚠️ 跳过无EXIF文件: {}", file_path);
            window.emit("process-progress", serde_json::json!({
                "current": index + 1,
                "total": total_files,
                "filepath": file_path,
                "status": "skipped"
            })).map_err(|e| e.to_string())?;
            continue;
        }

        // 4. 执行处理核心逻辑
        let path_clone = file_path.clone();
        let style_clone = style.clone();
        let font_clone = font_filename.clone();
        let weight_clone = font_weight.clone();
        
        let (make, model, params) = metadata::get_exif_string_tuple(&path_clone);

        // 放到 blocking 线程池处理图片
        let result = tauri::async_runtime::spawn_blocking(move || {
            processor::run(
                path_clone, 
                style_clone, 
                font_clone, 
                weight_clone, 
                shadow_intensity, 
                make,   
                model,  
                params  
            )
        }).await;

        match result {
            Ok(Ok(_)) => {
                println!("✅ 完成: {}", file_path);
            },
            Ok(Err(e)) => {
                println!("❌ 处理失败 {}: {}", file_path, e);
            },
            Err(e) => {
                println!("❌ 线程错误: {}", e);
            }
        }

        // 5. 发送进度条事件
        window.emit("process-progress", serde_json::json!({
            "current": index + 1,
            "total": total_files,
            "filepath": file_path,
            "status": "processing"
        })).map_err(|e| e.to_string())?;
    }

    println!("================ 批处理完成 (耗时: {:.2?}) ================", batch_start.elapsed());
    
    // 发送完成状态
    window.emit("process-status", "finished").map_err(|e| e.to_string())?;

    Ok("Batch processing complete".to_string())
}

#[tauri::command]
async fn process_single_image(
    file_path: String, 
    style: String, 
    font_filename: String,
    font_weight: String,
    shadow_intensity: f32 
) -> Result<String, String> {
    
    let total_start = Instant::now(); 
    println!("--------------------------------------------------");
    println!("🚀 [PERF] 收到单张处理请求: {:?}", file_path);

    let t_meta = Instant::now();
    let (make, model, params) = metadata::get_exif_string_tuple(&file_path);
    println!("⏱️ [PERF] Main线程-元数据提取: {:.2?}", t_meta.elapsed());

    let result = tauri::async_runtime::spawn_blocking(move || {
        processor::run(
            file_path, 
            style, 
            font_filename, 
            font_weight, 
            shadow_intensity, 
            make,   
            model,  
            params  
        )
    }).await;

    println!("✅ [PERF] 任务总耗时 (含线程调度): {:.2?}", total_start.elapsed());
    println!("--------------------------------------------------");

    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(format!("任务执行失败: {}", e)),
    }
}

// ... debug commands ...
#[tauri::command]
async fn debug_shadow_grid() -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        debug::generate_shadow_grid()
    }).await;
    match result { Ok(r) => r, Err(e) => Err(e.to_string()) }
}

#[tauri::command]
async fn debug_weight_grid() -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        debug::generate_weight_grid()
    }).await;
    match result { Ok(r) => r, Err(e) => Err(e.to_string()) }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(AppState { should_stop: AtomicBool::new(false) }))
        .invoke_handler(tauri::generate_handler![
            process_single_image,
            start_batch_process,
            stop_batch_process,
            read_photo_metadata,
            get_font_list,
            check_file_exif,
            debug_shadow_grid,
            debug_weight_grid,
            metadata::filter_files,
            metadata::scan_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}