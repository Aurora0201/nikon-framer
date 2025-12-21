// 声明模块
mod metadata;
mod resources;
mod processor;
mod debug;
mod graphics;

use metadata::PhotoMetadata;
use std::time::Instant; // 🟢 引入计时器

// --- Tauri Commands ---

#[tauri::command]
fn get_font_list() -> Vec<String> {
    resources::get_font_list()
}

#[tauri::command]
fn read_photo_metadata(file_path: String) -> Result<PhotoMetadata, String> {
    let start = Instant::now(); // ⏱️ 开始计时

    let (make, model, params) = metadata::get_exif_string_tuple(&file_path);
    
    let display_model = if model.to_uppercase().starts_with(&make.to_uppercase()) {
        model.clone()
    } else {
        format!("{} {}", make, model)
    };

    println!("🚀 [PERF] 元数据读取耗时: {:.2?}", start.elapsed()); // ⏱️ 打印耗时

    Ok(PhotoMetadata {
        model: display_model,
        f_number: "See Params".to_string(),
        exposure_time: "See Params".to_string(),
        iso: params, 
        focal_length: "".to_string(),
    })
}

#[tauri::command]
async fn process_single_image(
    file_path: String, 
    style: String, 
    font_filename: String,
    font_weight: String,
    shadow_intensity: f32 
) -> Result<String, String> {
    
    let total_start = Instant::now(); // ⏱️ 总任务开始
    println!("--------------------------------------------------");
    println!("🚀 [PERF] 收到处理请求: {:?}", file_path);

    // 1. 读取元数据
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

// ... debug commands 保持不变 ...
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
        .invoke_handler(tauri::generate_handler![
            process_single_image, 
            read_photo_metadata,
            get_font_list,
            debug_shadow_grid,
            debug_weight_grid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}