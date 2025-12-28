use tauri::{App, Manager};
use tauri::path::BaseDirectory;
use crate::resources; // 引用 crate 根目录下的 resources 模块

pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // 1. 解析资源路径
    let resource_path = handle.path()
        .resolve("assets/fonts", BaseDirectory::Resource)
        .expect("无法解析字体资源路径");

    println!("🚀 [Setup] 检测到字体资源路径: {:?}", resource_path);

    // 2. 初始化资源模块
    resources::init_font_path(resource_path);

    Ok(())
}