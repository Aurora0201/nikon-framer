// src-tauri/src/processor/traits.rs
use image::DynamicImage;

// 🟢 加上 Send + Sync，让 trait object 可以在多线程间安全移动
pub trait FrameProcessor: Send + Sync {
    fn process(
        &self, 
        img: &DynamicImage, 
        make: &str, 
        model: &str, 
        params: &str
    ) -> Result<DynamicImage, String>;
}