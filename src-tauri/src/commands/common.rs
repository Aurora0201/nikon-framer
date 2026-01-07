use tauri::State;
use std::{fs::File, io::BufReader, sync::{Arc, atomic::Ordering}};
use crate::{models::BatchContext, state::AppState};
use crate::metadata; // 引用 crate::metadata
use std::path::Path;
use std::io::Cursor;
use image::{ImageFormat, imageops::FilterType};
use base64::{Engine as _, engine::general_purpose};

#[tauri::command]
pub fn check_file_exif(path: String) -> bool {
    metadata::has_exif(&path)
}

#[tauri::command]
pub fn stop_batch_process(state: State<'_, Arc<AppState>>) {
    println!("🛑 收到停止指令...");
    state.should_stop.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn check_output_exists(
    file_path: String,
    style: String
) -> Option<String> { // 返回值 Option<String> 现在代表 Base64 字符串
    
    // 1. 计算目标路径 (和你之前的逻辑一样)
    let suffix = format!("_{}", style);
    let path_obj = Path::new(&file_path);
    let parent = path_obj.parent().unwrap_or(Path::new("."));
    let file_stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let target_filename = format!("{}{}.jpg", file_stem, suffix);
    let target_path = parent.join(target_filename);

    // 2. 检查文件是否存在
    if !target_path.exists() {
        return None;
    }

    // 3. 🟢 [核心修改] 读取 -> 缩放 -> 转 Base64
    // 不直接返回路径，而是返回图片数据
    match image::open(&target_path) {
        Ok(img) => {
            // A. 缩放图片 (性能关键！只用来预览不需要全尺寸)
            // 假设预览框最大也就 1000px 宽，这样生成的字符串很小，传输极快
            let resized = img.thumbnail(1000, 1000); 

            // B. 写入内存 buffer
            let mut buffer = Vec::new();
            // 存为 JPEG 格式，质量 80，进一步减小体积
            if let Err(_) = resized.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg) {
                return None;
            }

            // C. 转 Base64
            let b64 = general_purpose::STANDARD.encode(&buffer);
            
            // D. 返回带前缀的完整 Data URL
            Some(format!("data:image/jpeg;base64,{}", b64))
        },
        Err(e) => {
            println!("读取预览图失败: {}", e);
            None
        }
    }
}


// 🟢 新增：批量过滤未处理的文件
// 输入：所有待处理的文件路径列表 + 当前样式 ID
// 输出：仅返回那些“硬盘上还不存在结果图”的文件路径
#[tauri::command]
pub fn filter_unprocessed_files(
    paths: Vec<String>, 
    // 🟢 修改：这里接收完整的 Context JSON，而不是简单的 style string
    // 这样我们就能反序列化出 StyleOptions 枚举，从而调用 is_editable()
    context: BatchContext 
) -> Vec<String> {
    
    // 1. 检查是否为可编辑模式
    if context.options.is_editable() {
        println!("⚡ [Filter] 检测到可编辑模式 ({:?})，跳过重复检查，强制全量处理。", context.options);
        return paths; // 直接把所有路径原样返回
    }

    // 2. 如果是静态模式，执行原来的检查逻辑
    let suffix = context.options.filename_suffix();
    let mut to_process = Vec::new();

    // 🔴 修复点：使用 &paths 进行借用迭代，而不是消耗所有权
    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        let file_stem = path.file_stem().unwrap().to_string_lossy();
        let target_name = format!("{}_{}.jpg", file_stem, suffix);
        let target_path = parent.join(target_name);

        if !target_path.exists() {
            // 🟢 因为 path_str 现在只是一个借来的引用，
            // 我们需要 clone() 一份放进新的 Vec 里
            to_process.push(path_str.clone());
        }
    }

    println!("🔍 [Filter] 过滤完成: 输入 {} -> 输出 {}", paths.len(), to_process.len());
    to_process
}


// 🟢 引入 exif 库相关类型
use exif::{In, Tag, Value, Reader as ExifReader};

/// 读取本地图片，**自动矫正EXIF方向**，缩放并转换为 JPEG Blob
#[tauri::command]
pub fn read_local_image_blob(file_path: String) -> Result<Vec<u8>, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    // =================================================================
    // 🟢 阶段 1: 读取并矫正 EXIF 方向
    // =================================================================
    
    // 1.1 读取原始像素数据
    // 注意：我们要把 img 声明为可变的 (mut)，因为后面可能要旋转它
    let file_for_pixels = File::open(path).map_err(|e| format!("无法打开文件: {}", e))?;
    let reader_for_pixels = BufReader::new(file_for_pixels);
    let mut img = image::load(reader_for_pixels, ImageFormat::from_path(path).unwrap_or(ImageFormat::Jpeg))
        .map_err(|e| format!("图片解码失败: {}", e))?;

    // 1.2 读取 EXIF 方向信息
    // 我们需要单独再打开一次文件来读取 EXIF，这样最稳妥，避免不同库争抢 Cursor 位置
    let orientation_val = get_exif_orientation(path).unwrap_or(1); // 默认方向 1 (正常)

    // 1.3 根据方向值应用旋转/翻转
    // EXIF 方向定义参考：http://sylvana.net/jpegcrop/exif_orientation.html
    if orientation_val != 1 {
        img = match orientation_val {
            2 => img.fliph(),                // 水平镜像
            3 => img.rotate180(),            // 旋转 180度
            4 => img.flipv(),                // 垂直镜像
            5 => img.rotate90().fliph(),     // 旋转90度 + 水平镜像
            6 => img.rotate90(),             // 顺时针旋转 90度 (最常见的竖屏情况)
            7 => img.rotate270().fliph(),    // 旋转270度 + 水平镜像
            8 => img.rotate270(),            // 逆时针旋转 90度
            _ => img,                        // 其他情况不做处理
        };
    }

    // =================================================================
    // 阶段 2: 缩放与编码 (保持原有逻辑)
    // =================================================================

    // 此时 img 已经是方向正确的了，再进行缩放
    let resized_img = img.resize(1600, 1600, FilterType::Triangle);

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    resized_img.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|e| format!("图片编码失败: {}", e))?;

    Ok(buffer)
}


// 🟢 辅助函数：尝试提取 EXIF 方向值
fn get_exif_orientation(path: &Path) -> Option<u16> {
    let file = File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif_reader = ExifReader::new();
    
    // 尝试从容器读取 EXIF
    let exif = exif_reader.read_from_container(&mut bufreader).ok()?;
    
    // 尝试获取主目录下的 Orientation 标签
    match exif.get_field(Tag::Orientation, In::PRIMARY) {
        Some(field) => match field.value {
            // EXIF 方向通常存储为 Short (u16) 类型
            Value::Short(ref v) if !v.is_empty() => Some(v[0]),
            _ => None,
        },
        None => None,
    }
}