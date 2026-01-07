// 1. 声明子模块
pub mod white_classic;
pub mod transparent_classic;
pub mod traits;
pub mod transparent_master;
pub mod white_polaroid;
pub mod white_master;
pub mod white_modern; // 🟢
pub mod signature;

use ab_glyph::FontArc;
use image::{DynamicImage, imageops};


// 3. 引入项目内部模块
use crate::models::StyleOptions;
use crate::processor::signature::SignatureProcessor;
use crate::processor::traits::FrameProcessor; 

use crate::processor::transparent_master::TransparentMasterProcessor;
use crate::processor::white_classic::WhiteClassicProcessor;
use crate::processor::white_master::WhiteMasterProcessor;
// 引入资源管理
use crate::resources::{self, FontFamily, FontWeight};

// 引入各处理器的特定结构体 (Input & Resources)
use crate::processor::transparent_classic::TransparentClassicProcessor;
use crate::processor::white_polaroid::WhitePolaroidProcessor;
use crate::processor::white_modern::WhiteModernProcessor;


// --- 公共辅助函数 ---

/// 根据高度调整图片大小 (保持长宽比)
pub fn resize_image_by_height(img: &DynamicImage, target_height: u32) -> DynamicImage {
    img.resize(target_height * 10, target_height, imageops::FilterType::Lanczos3)
}

// ==========================================
// 工厂函数: 核心装配车间
// ==========================================
pub fn create_processor(options: &StyleOptions) -> Box<dyn FrameProcessor + Send + Sync> {
    match options {
        
        // 1. 极简白底模式
        StyleOptions::WhiteClassic => {
            Box::new(WhiteClassicProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold) 
            })
        },

        // 2. 高斯模糊模式
        StyleOptions::TransparentClassic => {
            Box::new(TransparentClassicProcessor { 
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
            })
        },

        // 3. 大师透明模式
        StyleOptions::TransparentMaster => {
            Box::new(TransparentMasterProcessor {
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },

        // 4. 拍立得模式
        StyleOptions::WhitePolaroid => {
            Box::new(WhitePolaroidProcessor {
                font_data: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
            })
        },

        // 5. 大师白底模式 (🟢 新增)
        StyleOptions::WhiteMaster => {
            Box::new(WhiteMasterProcessor {
                main_font: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                script_font: resources::get_font(FontFamily::MrDafoe, FontWeight::Regular),
                serif_font: resources::get_font(FontFamily::AbhayaLibre, FontWeight::Medium),
            })
        },

        StyleOptions::WhiteModern => {
            Box::new(WhiteModernProcessor {
                // Modern 风格建议搭配无衬线字体
                font_bold: resources::get_font(FontFamily::InterDisplay, FontWeight::Bold),
                font_medium: resources::get_font(FontFamily::InterDisplay, FontWeight::Medium),
                font_script: resources::get_font(FontFamily::Birthstone, FontWeight::Regular),
                font_regular: resources::get_font(FontFamily::InterDisplay, FontWeight::Regular),
            })
        },
        // 🟢 修复 Signature 模式的初始化逻辑
        StyleOptions::Signature { text, font_scale, bottom_ratio } => {
            
            // 1. 从资源管理器获取原始数据 (Arc<Vec<u8>>)
            let font_data_arc = resources::get_font(FontFamily::InterDisplay, FontWeight::Medium);
            
            // 2. 🟢 核心修复：手动转换为 FontArc
            // 因为我们要维持现有架构，这里进行一次内存复制 (to_vec) 是最稳妥的
            // 这解决了 "expected FontRef found Arc" 的问题
            let font = FontArc::try_from_vec(font_data_arc.to_vec())
                .expect("Failed to parse font data");

            Box::new(SignatureProcessor {
                font, // 现在这里是 FontArc 类型了，匹配结构体定义
                text: text.clone(),
                font_scale: *font_scale,
                bottom_ratio: *bottom_ratio,
            })
        },

    }
}