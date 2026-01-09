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
mod parser;
mod batch;
mod utils;


use std::sync::Arc;
use state::AppState;
use tauri_plugin_log::{Target, TargetKind};


fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // 可选配置：设置日志轮转 (防止日志无限大)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll) 
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .targets([
                    Target::new(TargetKind::Stdout), // 让控制台显示
                    Target::new(TargetKind::LogDir { file_name: None }), // 让文件保存
                    Target::new(TargetKind::Webview), // (可选) 让前端 F12 console 也能看到 Rust 日志
                ])
                // 🟢 2. 配置日志级别 (Level)
                // Debug: 开发时用，显示最详细的信息
                // Info: 生产时用，显示关键流程
                // 如果你不设置，默认可能是 Info 或 Error，导致 debug! 看不到
                .level(log::LevelFilter::Debug)
                .build()
        )
        // 1. 状态管理 (使用 state.rs)
        .manage(Arc::new(AppState::new()))
        // 2. 初始化设置 (使用 setup.rs)
        .setup(setup::init)
        // 3. 注册命令 (从 commands 模块导入)
        .invoke_handler(tauri::generate_handler![
            // 批处理
            batch::start_batch_process_v3,
            //
            commands::check_output_exists,
            // 🟢 注册新命令
            commands::filter_unprocessed_files,
            // 通用命令
            commands::stop_batch_process,
            commands::check_file_exif,
            // 其他遗留命令
            commands::read_local_image_blob,
            commands::generate_thumbnail,
            metadata::filter_files,
            metadata::scan_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}