// src-tauri/src/processor/traits.rs
use image::DynamicImage;
use crate::parser::models::ParsedImageContext; // 🟢 引入新结构

pub trait FrameProcessor: Send + Sync {
    // 🟢 接口变了：不再接收 make/model/params 字符串，而是接收 ctx
    fn process(
        &self, 
        img: &DynamicImage, 
        ctx: &ParsedImageContext
    ) -> Result<DynamicImage, String>;
}