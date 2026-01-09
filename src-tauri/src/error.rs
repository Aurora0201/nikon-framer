// src-tauri/src/error.rs

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("图片处理错误: {0}")]
    Image(#[from] image::ImageError),

    // 针对那些还是 String 类型的旧代码，提供一个过渡容器
    #[error("系统错误: {0}")]
    System(String),
    
    #[error("路径计算失败: {0}")]
    PathCalculation(String),
}

// 核心：实现 Serialize，让前端接收到的是 JSON 对象而不是报错字符串
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        // 1. 错误码 (用于前端判断类型)
        state.serialize_field("code", match self {
            AppError::Io(_) => "IO_ERROR",
            AppError::Image(_) => "IMAGE_ERROR",
            AppError::System(_) => "SYSTEM_ERROR",
            AppError::PathCalculation(_) => "PATH_ERROR",
        })?;
        // 2. 错误信息 (用于展示)
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}