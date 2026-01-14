use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Instant;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, DynamicImage};
use log::{info, error, debug}; // 🟢 引入标准日志宏
use tauri::{Window, State, Emitter};
use rayon::prelude::*;
use serde_json::json;

// 🟢 引入错误定义
use crate::error::AppError; 

use crate::commands::{get_exif_data, has_exif};
use crate::models::{ExportConfig, ExportImageFormat, StyleOptions};
use crate::utils::calculate_target_path_core;
use crate::AppState;
use crate::parser::{models::ParsedImageContext};
use crate::processor::traits::FrameProcessor;
use crate::graphics::load_image_auto_rotate; 

// =========================================================
// 1. 上下文定义 (Context)
// =========================================================

pub struct GlobalContext {
    pub window: Window,
    pub app_state: Arc<AppState>,
    pub options: StyleOptions,
    pub total_files: usize,
    pub completed_count: Arc<AtomicUsize>,
    pub export: ExportConfig,
}

impl GlobalContext {
    // 🔴 变更：返回值从 Result<PathBuf, String> 改为 Result<PathBuf, AppError>
    pub fn calculate_target_path(&self, original_file_path: &str) -> Result<PathBuf, AppError> {
        // 调用 core 逻辑，并将返回的 String 错误包装进 AppError::PathCalculation
        calculate_target_path_core(
            original_file_path, 
            &self.export, 
            &self.options
        ).map_err(|e| AppError::PathCalculation(e))
    }
}

pub struct TaskContext {
    pub file_path: String,
    pub image: Option<DynamicImage>,
    pub parsed_ctx: Option<ParsedImageContext>,
    pub final_image: Option<DynamicImage>,
    pub output_path: Option<PathBuf>,
}

impl TaskContext {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            image: None,
            parsed_ctx: None,
            final_image: None,
            output_path: None,
        }
    }
}

// =========================================================
// 2. 管道接口定义 (Trait)
// =========================================================

pub enum StepResult {
    Continue,
    Skip(String),
    Stop,
}

pub trait PipelineStep: Send + Sync {
    // 🔴 变更：错误类型改为 AppError
    fn execute(&self, global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, AppError>;
}


// =========================================================
// 3. 具体步骤实现
// =========================================================

/// 步骤 1: 检查是否收到停止信号
struct CheckStopStep;
impl PipelineStep for CheckStopStep {
    fn execute(&self, global: &GlobalContext, _task: &mut TaskContext) -> Result<StepResult, AppError> {
        if global.app_state.should_stop.load(Ordering::Relaxed) {
            // 这是用户主动停止，info 级别即可
            info!("🛑 [Pipeline] 用户停止处理");
            return Ok(StepResult::Stop);
        }
        Ok(StepResult::Continue)
    }
}

/// 步骤 2: 检查 EXIF 是否存在
struct CheckExifStep;
impl PipelineStep for CheckExifStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, AppError> {
        if !has_exif(&task.file_path) {
            // 跳过不是错误，不需要 error!，warn 或 debug 即可
            debug!("⚠️ [Check] 无 EXIF 跳过: {}", task.file_path);
            return Ok(StepResult::Skip("无 EXIF 数据".to_string()));
        }
        Ok(StepResult::Continue)
    }
}

/// 步骤 3: 加载图片
struct LoadImageStep;
impl PipelineStep for LoadImageStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, AppError> {
        // 🟢 load_image_auto_rotate 现在返回 AppError，直接 ? 传播
        // 如果出错，AppError 会携带 context 信息
        let img = load_image_auto_rotate(&task.file_path)?;
        task.image = Some(img);
        Ok(StepResult::Continue)
    }
}

/// 步骤 4: 核心处理
struct ProcessFrameStep {
    processor: Arc<Box<dyn FrameProcessor + Send + Sync>>,
}
impl PipelineStep for ProcessFrameStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, AppError> {
        let img = task.image.as_ref().ok_or_else(|| {
             AppError::System("逻辑错误: 步骤4执行时图片未加载".to_string())
        })?;
        
        // A. 解析数据 (get_exif_data 现在返回 Result<RawExifData, AppError>)
        // 如果这里出错（比如 IO 错误），直接传播中断
        let raw_exif = get_exif_data(&task.file_path)?;
        let parsed_ctx = crate::parser::parse(raw_exif);
        
        // B. 绘制合成
        // processor.process 目前可能还返回 String 错误，我们需要包装一下
        let final_img = self.processor.process(img, &parsed_ctx)
            .map_err(|e| {
                error!("❌ [Process] 绘图算法失败 [{}]: {}", task.file_path, e);
                AppError::Image(image::ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
                // 或者用 AppError::System(format!("绘图失败: {}", e))
            })?;
            
        task.parsed_ctx = Some(parsed_ctx);
        task.final_image = Some(final_img);
        Ok(StepResult::Continue)
    }
}

/// 步骤 5: 保存文件 (Pro版 & OCP & Structured Error)
struct SaveImageStep;
impl PipelineStep for SaveImageStep {
    fn execute(&self, global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, AppError> {
        let final_img = task.final_image.as_ref()
            .ok_or_else(|| AppError::System("逻辑错误: 最终图未生成".to_string()))?;

        // 1. 路径计算 (已封装在 GlobalContext，返回 AppError)
        let output_path = global.calculate_target_path(&task.file_path)?;

        debug!("💾 [Save] 准备写入: {:?}", output_path);

        // 2. 自动创建父目录
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    error!("❌ [Save] 创建目录失败 {:?}: {}", parent, e);
                    AppError::Io(e) // 🟢 保持原始 IO 错误类型
                })?;
            }
        }

        // 3. 智能图像转换 (处理 JPG 不支持 Alpha 的问题)
        let img_to_save: Cow<DynamicImage> = if !global.export.format.supports_alpha() && final_img.color().has_alpha() {
            debug!("  -> 格式不支持透明度，正在转换为 RGB8..."); 
            Cow::Owned(DynamicImage::ImageRgb8(final_img.to_rgb8()))
        } else {
            Cow::Borrowed(final_img)
        };

        // 4. 创建文件流
        let file = File::create(&output_path).map_err(|e| {
            error!("❌ [Save] 创建文件句柄失败 {:?}: {}", output_path, e);
            AppError::Io(e)
        })?;
        let mut writer = BufWriter::new(file);

        // 5. 编码保存
        // 🟢 map_err 模式：先记录日志，再抛出 AppError
        let width = img_to_save.width();
        let height = img_to_save.height();
        let color_type = img_to_save.color().into();

        match global.export.format {
            ExportImageFormat::Png => {
                let encoder = PngEncoder::new(&mut writer);
                encoder.write_image(img_to_save.as_bytes(), width, height, color_type)
                    .map_err(|e| {
                        error!("❌ [Save] PNG 编码失败: {}", e);
                        AppError::Image(e) // 自动转换 ImageError
                    })?;
            },
            ExportImageFormat::Jpg => {
                let encoder = JpegEncoder::new_with_quality(&mut writer, global.export.quality);
                encoder.write_image(img_to_save.as_bytes(), width, height, color_type)
                    .map_err(|e| {
                        error!("❌ [Save] JPG 编码失败: {}", e);
                        AppError::Image(e)
                    })?;
            },
        }

        task.output_path = Some(output_path);
        
        // 成功日志 (info 级别，证明这张图搞定了)
        info!("✅ [Save] 已保存: {:?}", task.file_path);
        Ok(StepResult::Continue)
    }
}


// =========================================================
// 4. 管道执行器 (Runner)
// =========================================================

struct Pipeline {
    steps: Vec<Box<dyn PipelineStep>>,
}

impl Pipeline {
    fn new() -> Self {
        Self { steps: Vec::new() }
    }

    fn add_step<S: PipelineStep + 'static>(mut self, step: S) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    /// 运行单张图片的完整流程
    fn run(&self, global: &GlobalContext, file_path: String) {
        let mut task = TaskContext::new(file_path.clone());
        let mut skip_reason = None;
        let mut error_obj: Option<AppError> = None; // 🔴 变更：存储 AppError
        let mut is_stopped = false;

        // --- 核心循环 ---
        for step in &self.steps {
            match step.execute(global, &mut task) {
                Ok(StepResult::Continue) => continue,
                Ok(StepResult::Stop) => {
                    is_stopped = true;
                    break;
                },
                Ok(StepResult::Skip(reason)) => {
                    skip_reason = Some(reason);
                    break;
                },
                Err(e) => {
                    // 🟢 捕获结构化错误
                    error_obj = Some(e);
                    break;
                }
            }
        }

        if is_stopped { return; }

        // --- 统一的进度报告 ---
        let current = global.completed_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        let (status, msg_payload) = if let Some(err) = error_obj {
            // 🟢 错误时，status="error"，message 是序列化后的 AppError 对象
            // 前端可以通过 msg_payload.code 判断错误类型
            ("error", json!(err)) 
        } else if let Some(reason) = skip_reason {
            ("skipped", json!(reason))
        } else {
            ("processing", json!(null)) // 成功
        };

        // 发送事件
        let _ = global.window.emit("process-progress", json!({
            "current": current,
            "total": global.total_files,
            "filepath": file_path,
            "status": status,
            "message": msg_payload // 这里的 message 可能是一个字符串，也可能是一个 Error 对象
        }));
        
        // 服务端最后一道日志防线
        if status == "error" {
            // 这里的 err 已经在各个 step 里由 log::error 记录过了，所以这里 debug 即可
            debug!("❌ [Pipeline] 任务终止: {}", file_path);
        }
    }
}

// =========================================================
// 5. API 入口函数
// =========================================================

#[tauri::command]
pub async fn start_batch_process_v3(
    window: Window,
    state: State<'_, Arc<AppState>>,
    file_paths: Vec<String>,
    context: crate::models::BatchContext,
) -> Result<String, AppError> { // 🔴 变更：返回 AppError
    
    info!("🚀 [API V3] Pipeline Mode Started ({} files)", file_paths.len());

    let state_arc = (*state).clone();
    state_arc.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();
    let completed_count = Arc::new(AtomicUsize::new(0));

    // 构建全局上下文
    let global_ctx = Arc::new(GlobalContext {
        window: window.clone(),
        app_state: state_arc.clone(),
        options: context.options.clone(),
        total_files,
        completed_count,
        export: context.export.clone()
    });

    let processor_strategy = crate::processor::create_processor(&context.options);
    let processor_arc = Arc::new(processor_strategy);

    // 组装流水线
    let pipeline = Arc::new(Pipeline::new()
        .add_step(CheckStopStep)
        .add_step(CheckExifStep)
        .add_step(LoadImageStep)
        .add_step(ProcessFrameStep { processor: processor_arc })
        .add_step(SaveImageStep)
    );

    // 启动线程池
    let result = tauri::async_runtime::spawn_blocking(move || {
        file_paths.par_iter().for_each(|file_path| {
            pipeline.run(&global_ctx, file_path.clone());
        });
    }).await;

    // 处理 spawn_blocking 的 JoinError
    result.map_err(|e| AppError::System(format!("线程池异常: {}", e)))?;

    let duration = batch_start.elapsed();
    
    if state_arc.should_stop.load(Ordering::Relaxed) {
        window.emit("process-status", "stopped").map_err(|e| AppError::System(e.to_string()))?;
        return Ok("Stopped by user".to_string());
    }

    info!("✨ [API V3] Batch Complete in {:.2?}", duration);
    window.emit("process-status", "finished").map_err(|e| AppError::System(e.to_string()))?;

    Ok(format!("Done in {:.2?}", duration))
}