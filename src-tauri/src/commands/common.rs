// src-tauri/src/commands/common.rs

use exif::{In, Reader, Tag, Value};
use log::{debug, error, info, warn}; // 引入日志宏
use tauri::State;
use std::{fs::{self, File}, io::BufReader, sync::{Arc, atomic::Ordering}};
use std::io::Cursor;
use image::ImageFormat;
use base64::{Engine as _, engine::general_purpose};

// 🟢 引入我们的新错误类型
use crate::{error::AppError, parser::models::RawExifData};
use crate::{
    graphics::load_image_auto_rotate, 
    models::{BatchContext, ExportConfig, StyleOptions}, 
    state::AppState, 
    utils::calculate_target_path_core,
};

// ==========================================
// 1. 无需返回错误的小命令
// ==========================================

#[tauri::command]
pub fn check_file_exif(path: String) -> bool {
    has_exif(&path)
}

#[tauri::command]
pub fn stop_batch_process(state: State<'_, Arc<AppState>>) {
    info!("🛑 收到停止指令...");
    state.should_stop.store(true, Ordering::Relaxed);
}

// ==========================================
// 2. 核心：重构内部 Helper 函数
// ==========================================

/// 🔒 内部通用函数：读取 -> 旋转 -> 缩放 -> 编码
/// 🔴 修改：返回值从 Result<Vec<u8>, String> 变为 Result<Vec<u8>, AppError>
fn load_and_resize_blob(file_path: &str, max_dimension: u32) -> Result<Vec<u8>, AppError> {
    
    // 1. 加载并旋转
    // 注意：假设 load_image_auto_rotate 暂时还返回 String 错误，
    // 我们用 AppError::System 包装它，等未来重构 graphics 模块时再改
    let img = load_image_auto_rotate(file_path)
        .map_err(|e| AppError::System(format!("加载图片失败: {}", e)))?;

    // 2. 智能缩放 (thumbnail 优化)
    let resized_img = img.thumbnail(max_dimension, max_dimension);

    // 3. 编码为 JPEG
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    // 🟢 核心优化：这里不再需要 map_err(|e| e.to_string())
    // image crate 的错误会自动转换为 AppError::Image
    resized_img.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(buffer)
}

// ==========================================
// 3. 重构 API Commands
// ==========================================

/// 读取本地图片 Blob
/// 🔴 修改：返回 AppError
#[tauri::command]
pub fn read_local_image_blob(file_path: String) -> Result<Vec<u8>, AppError> {
    // 直接调用，错误自动向上传递
    load_and_resize_blob(&file_path, 1600)
}

/// 生成缩略图 (异步)
/// 🔴 修改：返回 AppError
#[tauri::command]
pub async fn generate_thumbnail(file_path: String) -> Result<String, AppError> {
    // spawn_blocking
    let result = tauri::async_runtime::spawn_blocking(move || {
        let bytes = load_and_resize_blob(&file_path, 200)?;
        let b64 = general_purpose::STANDARD.encode(&bytes);
        // 返回成功结果
        Ok::<String, AppError>(format!("data:image/jpeg;base64,{}", b64))
    }).await;

    // 处理线程 JoinError (极为罕见，但也属于 System 错误)
    let inner_result = result.map_err(|e| AppError::System(format!("线程池异常: {}", e)))?;

    // 返回内部业务结果 (AppError 会自动序列化发给前端)
    inner_result
}

// ==========================================
// 4. 保持原有签名但增强日志的函数
// ==========================================

#[tauri::command]
pub fn check_output_exists(
    file_path: String,
    style_options: StyleOptions, 
    export_config: ExportConfig, 
) -> Option<String> {
    
    // 1. 路径计算
    let target_path = match calculate_target_path_core(&file_path, &export_config, &style_options) {
        Ok(p) => p,
        Err(e) => {
            // 🟢 使用 error! 记录
            error!("❌ [Check] 路径计算错误 [{}]: {}", file_path, e);
            return None;
        }
    };

    if !target_path.exists() {
        return None;
    }

    let target_path_str = target_path.to_str()?;

    // 2. 加载预览
    match load_and_resize_blob(target_path_str, 1000) {
        Ok(buffer) => {
            let b64 = general_purpose::STANDARD.encode(&buffer);
            let mime = export_config.format.mime_type();
            Some(format!("data:{};base64,{}", mime, b64))
        },
        Err(e) => {
            // 🟢 使用 warn! 记录 (这属于非致命错误，可能是文件损坏或占用)
            warn!("⚠️ [Check] 预览图存在但加载失败 [{}]: {:?}", target_path_str, e);
            None
        }
    }
}

// 批量过滤函数 (保持逻辑，日志已在之前步骤优化过，这里确认一下引用没问题)
#[tauri::command]
pub fn filter_unprocessed_files(
    paths: Vec<String>, 
    context: BatchContext 
) -> Vec<String> {
    info!("🔍 [Filter] 开始检查 {} 个文件...", paths.len());

    if context.options.is_editable() {
        info!("⚡ [Filter] 检测到可编辑模式，强制全量处理。");
        return paths;
    }

    let mut to_process = Vec::new();
    let mut skipped_count = 0;
    let mut error_count = 0;

    for path_str in &paths {
        match context.calculate_target_path(path_str) {
            Ok(target_path) => {
                if target_path.exists() {
                    skipped_count += 1;
                } else {
                    to_process.push(path_str.clone());
                }
            },
            Err(e) => {
                error!("⚠️ [Filter] 路径计算错误 [{}]: {}", path_str, e);
                to_process.push(path_str.clone());
                error_count += 1;
            }
        }
    }

    info!(
        "✅ [Filter] 完成: 输入 {} -> 需处理 {} (跳过 {}, 异常 {})", 
        paths.len(), to_process.len(), skipped_count, error_count
    );
    
    to_process
}


/// 读取文件 EXIF 并填充 RawExifData
/// 
/// 🟢 变更：返回值从 RawExifData 改为 Result<RawExifData, AppError>
/// 这样调用者可以区分是“文件不存在”还是“单纯没有EXIF”
pub fn get_exif_data(path: &str) -> Result<RawExifData, AppError> {
    // 1. 尝试打开文件 (IO 错误应该抛出)
    let file = File::open(path).map_err(|e| {
        error!("❌ [Metadata] 无法打开文件 [{}]: {}", path, e);
        AppError::Io(e)
    })?;

    // 2. 读取 EXIF
    let mut reader = BufReader::new(file);
    
    // 🟢 策略调整：如果读取 EXIF 失败（比如是 PNG 或 纯文本文件），
    // 这不算系统错误，而是“无数据”。所以我们记录警告，但返回默认空数据。
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(e) => e,
        Err(e) => {
            // debug! 级别即可，因为很多图片确实没有 EXIF，不需要刷屏 error
            debug!("ℹ️ [Metadata] 未找到 EXIF 信息 [{}]: {}", path, e);
            return Ok(RawExifData::default());
        }
    };

    // --- 辅助闭包：获取字符串值 (逻辑保持不变，但增加健壮性) ---
    let get_text = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .map(|f| f.display_value().with_unit(&exif).to_string())
            .unwrap_or_default()
            .replace("\"", "")
            .trim()
            .to_string()
    };

    // --- 辅助闭包：获取 u32 ---
    let get_u32 = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0))
    };

    // --- 辅助闭包：获取 f32 ---
    let get_f32 = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .and_then(|f| match &f.value {
                Value::Rational(v) if !v.is_empty() => {
                    let r = &v[0];
                    if r.denom == 0 { None } else { Some(r.num as f32 / r.denom as f32) }
                },
                Value::SRational(v) if !v.is_empty() => {
                    let r = &v[0];
                    if r.denom == 0 { None } else { Some(r.num as f32 / r.denom as f32) }
                },
                Value::Float(v) if !v.is_empty() => Some(v[0]),
                Value::Double(v) if !v.is_empty() => Some(v[0] as f32),
                _ => None
            })
    };

    // GPS 预留位置
    let lat = None;
    let long = None;

    let data = RawExifData {
        make: get_text(Tag::Make),
        model: get_text(Tag::Model),
        lens: get_text(Tag::LensModel),
        iso: get_u32(Tag::PhotographicSensitivity),
        aperture: get_f32(Tag::FNumber),
        shutter_speed: get_text(Tag::ExposureTime),
        focal_length: get_u32(Tag::FocalLengthIn35mmFilm)
            .or_else(|| get_u32(Tag::FocalLength)),
        datetime: get_text(Tag::DateTimeOriginal),
        artist: Some(get_text(Tag::Artist)),
        copyright: Some(get_text(Tag::Copyright)),
        gps_latitude: lat,
        gps_longitude: long,
    };

    // 成功日志（可选，防止刷屏可以用 debug!）
    // debug!("✅ [Metadata] 读取成功: {}", path);
    Ok(data)
}

/// 快速检查是否存在 EXIF
pub fn has_exif(path: &str) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            // 这里我们只关心“有没有”，所以打开失败也算 false，但记录一下 debug 日志
            debug!("⚠️ [HasExif] 文件无法打开 [{}]: {}", path, e);
            return false;
        }
    };
    let mut bufreader = BufReader::new(&file);
    exif::Reader::new().read_from_container(&mut bufreader).is_ok()
}

// 🟢 [Command] 批量过滤：只保留文件
#[tauri::command]
pub fn filter_files(paths: Vec<String>) -> Vec<String> {
    let original_count = paths.len();
    
    let filtered: Vec<String> = paths.into_iter()
        .filter(|path| {
            match fs::metadata(path) {
                Ok(meta) => meta.is_file(),
                Err(e) => {
                    warn!("⚠️ [Filter] 无法读取元数据，跳过 [{}]: {}", path, e);
                    false
                }
            }
        })
        .collect();

    if original_count != filtered.len() {
        debug!("🔍 [Filter] 过滤结果: {} -> {} (移除了文件夹或无效路径)", original_count, filtered.len());
    }
    
    filtered
}

// 🟢 [Command] 扫描文件夹
// 🟢 变更：返回 Result<Vec<String>, AppError> 以便前端捕获“文件夹无权限”等错误
#[tauri::command]
pub fn scan_folder(folder_path: String) -> Result<Vec<String>, AppError> {
    let allowed_exts = ["jpg", "jpeg", "png", "nef", "arw", "dng", "tif", "tiff", "webp"];
    let mut image_paths = Vec::new();

    // read_dir 可能会失败（权限不足、路径不存在），这里应该用 ? 抛出
    let entries = fs::read_dir(&folder_path).map_err(|e| {
        error!("❌ [Scan] 无法读取目录 [{}]: {}", folder_path, e);
        AppError::Io(e)
    })?;

    for entry in entries {
        // 单个文件读取失败不应该打断整个流程，记录日志并继续
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("⚠️ [Scan] 目录条目读取失败: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if allowed_exts.contains(&ext_str.to_lowercase().as_str()) {
                        if let Some(path_str) = path.to_str() {
                            image_paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    debug!("📂 [Scan] 扫描目录 [{}] 完成，找到 {} 张图片", folder_path, image_paths.len());
    Ok(image_paths)
}