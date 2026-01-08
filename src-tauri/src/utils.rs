use std::path::{Path, PathBuf};

use crate::models::{ExportConfig, StyleOptions};

// 🟢 这是一个独立的、无状态的辅助函数
// 它不依赖具体的 Context 结构体，只依赖它需要的数据
pub fn calculate_target_path_core(
    original_file_path: &str,
    export_config: &ExportConfig,
    style_options: &StyleOptions,
) -> Result<PathBuf, String> {
    let path_obj = Path::new(original_file_path);
    
    // 1. 获取文件名 (Stem)
    let file_stem = path_obj.file_stem()
        .ok_or_else(|| format!("无法解析文件名: {}", original_file_path))?
        .to_string_lossy();
    
    // 2. 确定父目录 (使用 export_config)
    let parent = if let Some(ref custom) = export_config.target_dir {
        PathBuf::from(custom)
    } else {
        path_obj.parent()
            .ok_or_else(|| format!("无法获取父目录: {}", original_file_path))?
            .to_path_buf()
    };

    // 3. 确定后缀 (使用 style_options)
    let suffix = style_options.filename_suffix();

    // 4. 确定扩展名 (使用 export_config 的 Enum)
    let ext = export_config.format.extension();

    // 5. 拼接
    let filename = format!("{}_{}.{}", file_stem, suffix, ext);
    Ok(parent.join(filename))
}