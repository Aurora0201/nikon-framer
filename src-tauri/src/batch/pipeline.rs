// src/batch/pipline.rs

use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Instant;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ ImageEncoder};
use log::info;
use tauri::{Window, State, Emitter};
use rayon::prelude::*;
use serde_json::json;
use image::DynamicImage;

use crate::models::{ExportConfig, ExportImageFormat, StyleOptions};
use crate::utils::calculate_target_path_core;
use crate::{AppState};
use crate::parser::{models::ParsedImageContext};
use crate::processor::traits::FrameProcessor;
use crate::graphics::load_image_auto_rotate; // 假设你把那个函数放到了 utils 模块

// =========================================================
// 1. 上下文定义 (Context)
// =========================================================

/// 全局只读上下文：所有步骤共享，存放通用配置和状态
pub struct GlobalContext {
    pub window: Window,
    pub app_state: Arc<AppState>,
    pub options: StyleOptions,
    pub total_files: usize,
    pub completed_count: Arc<AtomicUsize>,
    // 🟢 [新增] 必须把导出配置带入全局上下文
    pub export: ExportConfig,
}

impl GlobalContext {
    pub fn calculate_target_path(&self, original_file_path: &str) -> Result<PathBuf, String> {
        // 🟢 同样调用核心函数，传入自己的字段
        // 注意：GlobalContext 必须也有 export 和 options 字段
        calculate_target_path_core(
            original_file_path, 
            &self.export, 
            &self.options
        )
    }
}

/// 任务上下文：随单个文件流动，存放中间产物
/// 使用 Option 是因为在管道初期，很多数据还没生成
pub struct TaskContext {
    pub file_path: String,
    pub image: Option<DynamicImage>,         // 加载后填充
    pub parsed_ctx: Option<ParsedImageContext>, // 解析后填充
    pub final_image: Option<DynamicImage>,   // 处理后填充
    pub output_path: Option<PathBuf>,        // 保存后填充
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

/// 步骤执行结果
pub enum StepResult {
    Continue,           // 继续下一步
    Skip(String),       // 跳过当前文件 (附带原因)
    Stop,               // 停止整个批处理 (用户取消)
}

/// 管道步骤特征
/// 要求 Send + Sync 是为了能在多线程 (Rayon) 中安全运行
pub trait PipelineStep: Send + Sync {
    fn execute(&self, global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, String>;
}


// =========================================================
// 3. 具体步骤实现
// =========================================================

/// 步骤 1: 检查是否收到停止信号
struct CheckStopStep;
impl PipelineStep for CheckStopStep {
    fn execute(&self, global: &GlobalContext, _task: &mut TaskContext) -> Result<StepResult, String> {
        if global.app_state.should_stop.load(Ordering::Relaxed) {
            return Ok(StepResult::Stop);
        }
        Ok(StepResult::Continue)
    }
}

/// 步骤 2: 检查 EXIF 是否存在 (快速过滤)
struct CheckExifStep;
impl PipelineStep for CheckExifStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, String> {
        // 假设 metadata 模块在 crate::metadata
        if !crate::metadata::has_exif(&task.file_path) {
            return Ok(StepResult::Skip("无 EXIF 数据".to_string()));
        }
        Ok(StepResult::Continue)
    }
}

/// 步骤 3: 加载图片 (使用我们优化后的 load_image_auto_rotate)
struct LoadImageStep;
impl PipelineStep for LoadImageStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, String> {
        // 🟢 使用 ? 优雅地处理错误，如果失败直接抛出 Result Err
        let img = load_image_auto_rotate(&task.file_path)?;
        task.image = Some(img);
        Ok(StepResult::Continue)
    }
}

/// 步骤 4: 核心处理 (解析 + 绘图)
struct ProcessFrameStep {
    // 处理器策略作为成员变量持有
    processor: Arc<Box<dyn FrameProcessor + Send + Sync>>,
}
impl PipelineStep for ProcessFrameStep {
    fn execute(&self, _global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, String> {
        let img = task.image.as_ref().ok_or("逻辑错误: 图片未加载")?;
        
        // A. 解析数据
        let raw_exif = crate::metadata::get_exif_data(&task.file_path);
        let parsed_ctx = crate::parser::parse(raw_exif);
        
        // B. 绘制合成
        let final_img = self.processor.process(img, &parsed_ctx)
            .map_err(|e| format!("处理失败: {}", e))?;
            
        task.parsed_ctx = Some(parsed_ctx);
        task.final_image = Some(final_img);
        Ok(StepResult::Continue)
    }
}

/// 步骤 5: 保存文件 (Pro版)
struct SaveImageStep;
impl PipelineStep for SaveImageStep {
    fn execute(&self, global: &GlobalContext, task: &mut TaskContext) -> Result<StepResult, String> {
        // 1. 获取处理后的图像
        let final_img = task.final_image.as_ref()
            .ok_or_else(|| format!("💾 [Save] 严重逻辑错误: 文件 [{}] 的最终图像未生成", task.file_path))?;

        // 🟢 2. 统一路径计算 (复用逻辑)
        // GlobalContext 中包含 export 和 options，我们需要构造一个临时的 context 或者让 helper 能够拆开用
        // 这里假设我们给 GlobalContext 实现了类似的方法，或者直接用 BatchContext 的逻辑
        // 既然 GlobalContext 是从 BatchContext 转换来的，最好在 GlobalContext 上也复用 calculate_target_path
        // 这里为了演示，我们手动构造一下或者调用 helper (取决于你的架构)
        // 假设我们在 GlobalContext 上也添加了同样的方法：
        let output_path = global.calculate_target_path(&task.file_path)
             .map_err(|e| format!("💾 [Save] 路径计算失败: {}", e))?;

        info!("💾 [Save] 准备写入: {:?}", output_path);

        // 3. 自动创建父目录
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("💾 [Save] 无法创建目录 {:?}: {}", parent, e))?;
            }
        }

        // 🟢 4. 智能图像转换 (OCP: 询问格式是否支持 Alpha)
        // 使用 Cow (Copy on Write): 如果不需要转，就是引用，零开销；如果需要转，才复制内存
        let img_to_save: Cow<DynamicImage> = if !global.export.format.supports_alpha() && final_img.color().has_alpha() {
            // Log: 只有在真正发生转换时才记录，避免刷屏
            info!("  -> 检测到格式不支持透明度，正在转换为 RGB8..."); 
            Cow::Owned(DynamicImage::ImageRgb8(final_img.to_rgb8()))
        } else {
            Cow::Borrowed(final_img)
        };

        // 5. 创建文件流
        let file = File::create(&output_path)
            .map_err(|e| format!("💾 [Save] 文件创建失败 {:?}: {}", output_path, e))?;
        let mut writer = BufWriter::new(file);

        // 准备参数
        let width = img_to_save.width();
        let height = img_to_save.height();
        let color_type = img_to_save.color().into(); // 此时已经是正确的 ColorType (Rgb8 or Rgba8)

        // 🟢 6. 编码保存 (根据 Format 枚举分发)
        match global.export.format {
            ExportImageFormat::Png => {
                let encoder = PngEncoder::new(&mut writer);
                encoder.write_image(img_to_save.as_bytes(), width, height, color_type)
                    .map_err(|e| format!("💾 [Save] PNG 编码失败: {}", e))?;
            },
            ExportImageFormat::Jpg => {
                // JPG 质量从配置读取
                let encoder = JpegEncoder::new_with_quality(&mut writer, global.export.quality);
                encoder.write_image(img_to_save.as_bytes(), width, height, color_type)
                    .map_err(|e| format!("💾 [Save] JPG 编码失败: {}", e))?;
            },
            // OCP: 如果未来加了 WebP，编译器会在这里报错提示你处理
        }

        // 7. 更新上下文
        task.output_path = Some(output_path);
        
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
        let mut error_msg = None;
        let mut is_stopped = false;

        // --- 核心循环 ---
        for step in &self.steps {
            match step.execute(global, &mut task) {
                Ok(StepResult::Continue) => continue, // 继续下一步
                Ok(StepResult::Stop) => {
                    is_stopped = true;
                    break; // 停止当前任务 (外部 Rayon 会继续调度，但 CheckStopStep 会拦截)
                },
                Ok(StepResult::Skip(reason)) => {
                    skip_reason = Some(reason);
                    break; // 跳过后续步骤
                },
                Err(e) => {
                    error_msg = Some(e);
                    break; // 报错终止
                }
            }
        }

        if is_stopped { return; }

        // --- 统一的进度报告 ---
        // 无论成功、跳过还是失败，都要给前端一个交代
        let current = global.completed_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        let (status, msg) = if let Some(err) = error_msg {
            ("error", Some(err))
        } else if let Some(reason) = skip_reason {
            ("skipped", Some(reason))
        } else {
            ("processing", None) // 或 "success"
        };

        // 发送事件 (忽略发送失败，因为窗口可能已关闭)
        let _ = global.window.emit("process-progress", json!({
            "current": current,
            "total": global.total_files,
            "filepath": file_path,
            "status": status,
            "message": msg
        }));
        
        // 如果出错，可以在这里打印服务端日志
        if status == "error" {
            println!("❌ [Batch V3] Error handling {}: {:?}", file_path, msg);
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
    context: crate::models::BatchContext, // 确保这个结构体是公有的
) -> Result<String, String> {
    
    info!("🚀 [API V3] Pipeline Mode Started ({} files)", file_paths.len());

    // 1. 准备全局状态
    let state_arc = (*state).clone();
    state_arc.should_stop.store(false, Ordering::Relaxed);
    
    let total_files = file_paths.len();
    let batch_start = Instant::now();
    let completed_count = Arc::new(AtomicUsize::new(0));

    // 2. 构建全局上下文 (Arc封装以便多线程共享)
    

    let global_ctx = Arc::new(GlobalContext {
        window: window.clone(),
        app_state: state_arc.clone(),
        options: context.options.clone(),
        total_files,
        completed_count,
        export: context.export.clone()
    });

    // 3. 创建处理器策略 (Factory)
    let processor_strategy = crate::processor::create_processor(&context.options);
    let processor_arc = Arc::new(processor_strategy);

    // 4. 🔥 组装流水线 (The Assembly Line)
    // 这里体现了 OCP：如果想加功能，就在中间 insert 一个 step
    let pipeline = Arc::new(Pipeline::new()
        .add_step(CheckStopStep)
        .add_step(CheckExifStep)
        .add_step(LoadImageStep)
        .add_step(ProcessFrameStep { processor: processor_arc })
        .add_step(SaveImageStep)
    );

    // 5. 启动线程池进行并行计算
    let result = tauri::async_runtime::spawn_blocking(move || {
        file_paths.par_iter().for_each(|file_path| {
            // 所有脏活累活都委托给 pipeline.run
            pipeline.run(&global_ctx, file_path.clone());
        });
    }).await;

    // 6. 结束处理
    if let Err(e) = result {
        return Err(format!("Thread execution failed: {}", e));
    }

    let duration = batch_start.elapsed();
    
    // 检查是否是用户主动停止
    if state_arc.should_stop.load(Ordering::Relaxed) {
        window.emit("process-status", "stopped").map_err(|e| e.to_string())?;
        return Ok("Stopped by user".to_string());
    }

    info!("✨ [API V3] Batch Complete in {:.2?}", duration);
    window.emit("process-status", "finished").map_err(|e| e.to_string())?;

    Ok(format!("Done in {:.2?}", duration))
}