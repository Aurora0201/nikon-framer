// 声明模块
mod metadata;
mod resources;
mod processor;
mod debug;
mod graphics;

use metadata::PhotoMetadata;

// --- Tauri Commands ---

#[tauri::command]
fn get_font_list() -> Vec<String> {
    resources::get_font_list()
}

#[tauri::command]
fn read_photo_metadata(file_path: String) -> Result<PhotoMetadata, String> {
    // 🟢 更新：接收 3 个返回值 (Make, Model, Params)
    let (make, model, params) = metadata::get_exif_string_tuple(&file_path);
    
    // 为了前端显示，如果 Model 字符串里不包含 Make，我们把它们拼起来显示
    // 例如 Make="Nikon", Model="Z 8" -> 显示 "Nikon Z 8"
    // 如果 Model 本身就是 "Nikon Z 8"，就不重复拼了
    let display_model = if model.to_uppercase().starts_with(&make.to_uppercase()) {
        model.clone()
    } else {
        format!("{} {}", make, model)
    };

    Ok(PhotoMetadata {
        model: display_model,
        f_number: "See Params".to_string(),
        exposure_time: "See Params".to_string(),
        iso: params, // 这里的 params 已经是拼接好的光圈/快门/ISO字符串
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
    
    // 🟢 1. 在主线程读取元数据 (Make, Model, Params)
    // 这是为了获取厂商名 (make) 以便加载 Logo，以及获取准确的型号 (model) 进行排版
    let (make, model, params) = metadata::get_exif_string_tuple(&file_path);

    let result = tauri::async_runtime::spawn_blocking(move || {
        // 🟢 2. 将分离的元数据传入 Processor
        // 注意参数顺序必须与 processor::run 定义的一致
        processor::run(
            file_path, 
            style, 
            font_filename, 
            font_weight, 
            shadow_intensity, 
            make,   // 新增
            model,  // 新增
            params  // 新增
        )
    }).await;

    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(format!("任务执行失败: {}", e)),
    }
}

#[tauri::command]
async fn debug_shadow_grid() -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        debug::generate_shadow_grid()
    }).await;

    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(format!("Debug 任务失败: {}", e)),
    }
}

#[tauri::command]
async fn debug_weight_grid() -> Result<String, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        debug::generate_weight_grid()
    }).await;

    match result {
        Ok(inner_result) => inner_result,
        Err(e) => Err(format!("Debug 任务失败: {}", e)),
    }
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