use tauri::State;
use std::sync::{Arc, atomic::Ordering};
use crate::state::AppState;
use crate::metadata; // 引用 crate::metadata

#[tauri::command]
pub fn check_file_exif(path: String) -> bool {
    metadata::has_exif(&path)
}

#[tauri::command]
pub fn stop_batch_process(state: State<'_, Arc<AppState>>) {
    println!("🛑 收到停止指令...");
    state.should_stop.store(true, Ordering::Relaxed);
}