use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use crate::parser::models::ParsedImageContext;
use crate::processor::traits::FrameProcessor;
use crate::graphics::{self, calculate_browser_baseline_offset, calculate_corrected_font_size};

pub struct SignatureProcessor {
    pub font: FontArc,
    pub text: String,
    pub font_scale: f32,
    pub bottom_ratio: f32,
}

impl FrameProcessor for SignatureProcessor {
    fn process(
        &self,
        img: &DynamicImage,
        _ctx: &ParsedImageContext
    ) -> Result<DynamicImage, String> {
        
        let mut canvas = img.clone();
        let width = canvas.width();
        let height = canvas.height();

        // 1. 字体准备
        // -------------------------------------------------------------
        // 使用通用函数获取修正后的字号 (含 DPI 校准)
        let font_size = calculate_corrected_font_size(width, self.font_scale);
        
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        // 2. X轴计算 (水平居中)
        let (text_w, _text_h) = graphics::text_size(&self.text, scale, &self.font);
        let x = (width as i32 - text_w as i32) / 2;

        // 3. Y轴计算 (基线对齐)
        // -------------------------------------------------------------
        let target_line_y = height as f32 * (1.0 - self.bottom_ratio);
        let ascent = scaled_font.ascent();

        // 🟢 使用通用函数获取基线偏移量 (模拟浏览器渲染行为)
        let vertical_offset_px = calculate_browser_baseline_offset(font_size);

        // 最终公式：目标线 - 基线高度 - 浏览器模拟偏移
        let y = (target_line_y - ascent - vertical_offset_px) as i32;
        
        // 4. 绘制文字
        // -------------------------------------------------------------
        let white = Rgba([255, 255, 255, 240]); 
        
        draw_text_mut(
            &mut canvas,
            white,
            x,
            y,
            scale,
            &self.font,
            &self.text,
        );

        Ok(canvas)
    }
}