// 声明所有顶层模块
mod metadata;
mod resources;
mod processor;
mod debug;
mod graphics;
mod models;
// 新增的模块
mod state;
mod setup;
mod commands;

use std::sync::Arc;
use state::AppState;


fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // 1. 状态管理 (使用 state.rs)
        .manage(Arc::new(AppState::new()))
        // 2. 初始化设置 (使用 setup.rs)
        .setup(setup::init)
        // 3. 注册命令 (从 commands 模块导入)
        .invoke_handler(tauri::generate_handler![
            // 批处理
            commands::batch::start_batch_process_v2,
            //
            commands::common::check_output_exists,
            // 通用命令
            commands::common::stop_batch_process,
            commands::common::check_file_exif,
            // 其他遗留命令
            metadata::filter_files,
            metadata::scan_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}