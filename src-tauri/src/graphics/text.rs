// 🟢 4. 修复 text_size 的 Option 报错
// (你可以把这个放在 src/graphics/mod.rs 里，或者暂时放在这里)
use ab_glyph::{Font, FontArc, PxScale, ScaleFont}; // 需要引入 Font trait

pub fn text_size(text: &str, scale: PxScale, font: &FontArc) -> (u32, u32) {
    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0;
    let mut height = 0.0;

    for c in text.chars() {
        // 🟢 修复：with_scale_and_position 直接返回 Glyph，不需要 if let Some
        let glyph = scaled_font.glyph_id(c).with_scale_and_position(scale, ab_glyph::point(0.0, 0.0));
        
        // outline_glyph 才会返回 Option，这里需要 if let Some
        if let Some(outlined) = scaled_font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            width += scaled_font.h_advance(outlined.glyph().id);
            if bounds.height() > height {
                height = bounds.height();
            }
        }
    }
    
    if text.is_empty() {
        return (0, 0);
    }

    (width.ceil() as u32, height.ceil() as u32)
}


/// 计算经过 DPI 校准后的字体大小 (物理像素)
///
/// 前端 (Web) 通常基于 96 DPI，而后端图形库常基于 72 DPI。
/// 为了实现"所见即所得"，需要引入校准系数。
/// 
/// - `image_width`: 图片宽度
/// - `font_scale`: 前端传入的字体比例 (0.0 ~ 1.0)
pub fn calculate_corrected_font_size(image_width: u32, font_scale: f32) -> f32 {
    // 🟢 用户最终微调确认的校准系数
    const DPI_CORRECTION_FACTOR: f32 = 1.21;
    
    let raw_font_size = image_width as f32 * font_scale;
    raw_font_size * DPI_CORRECTION_FACTOR
}


/// 计算模拟浏览器基线的垂直偏移量 (像素)
///
/// Web 浏览器 (CSS) 在渲染 line-height: 1 的文字时，基线位置往往比
/// 字体 metrics 中的标准 Baseline 要高。为了实现"所见即所得"，
/// 需要引入这个手动微调的偏移量。
///
/// - `font_size`: 经过 DPI 校准后的字号
pub fn calculate_browser_baseline_offset(font_size: f32) -> f32 {
    // 调试确认的最佳偏移比例 (针对 Inter Display 字体)
    // 0.121 代表向上偏移字号的 12.1%
    const BROWSER_BASELINE_RATIO: f32 = 0.121;
    
    font_size * BROWSER_BASELINE_RATIO
}