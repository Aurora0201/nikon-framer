use image::{DynamicImage, GenericImageView, Rgba, RgbaImage, imageops};

/// 阴影配置描述文件 (Shadow Profile)
/// 将所有控制阴影外观的参数封装在此，符合 Builder 模式
#[derive(Debug, Clone, Copy)]
pub struct ShadowProfile {
    pub sigma: f32,       // 模糊半径 (Blur)
    pub offset_x: i32,    // X 轴偏移
    pub offset_y: i32,    // Y 轴偏移
    pub spread: i32,      // 扩散/收缩 (Spread, 负值表示收缩)
    pub color: Rgba<u8>,  // 阴影颜色
}

impl ShadowProfile {
    // =========================================================
    // 1. 预设工厂 (Presets)
    // =========================================================

    /// 预设：精致版 (Subtle)
    /// 适用于：小图标、按钮、轻微隆起的元素
    pub fn preset_subtle() -> Self {
        Self {
            sigma: 10.0,
            offset_x: 0,
            offset_y: 10,
            spread: -2,
            color: Rgba([0, 0, 0, 160]),
        }
    }

    /// 预设：标准版 (Standard / HTML Match)
    /// 适用于：卡片、预览框、大多数 UI 容器
    /// 特点：类似于 Apple/Nikon 风格，黑且实
    pub fn preset_standard() -> Self {
        Self {
            sigma: 15.0,
            offset_x: 0,
            offset_y: 15,
            spread: -5,
            color: Rgba([0, 0, 0, 190]),
        }
    }

    /// 预设：悬浮版 (Floating)
    /// 适用于：弹窗、被选中的元素
    pub fn preset_floating() -> Self {
        Self {
            sigma: 25.0,
            offset_x: 0,
            offset_y: 30,
            spread: -8,
            color: Rgba([0, 0, 0, 210]),
        }
    }

    /// 自定义构造器
    pub fn new(sigma: f32, offset: (i32, i32), spread: i32, color: Rgba<u8>) -> Self {
        Self {
            sigma,
            offset_x: offset.0,
            offset_y: offset.1,
            spread,
            color,
        }
    }

    // =========================================================
    // 2. 链式修改器 (Modifiers)
    // =========================================================
    
    pub fn with_color(mut self, color: Rgba<u8>) -> Self {
        self.color = color;
        self
    }

    pub fn with_offset(mut self, x: i32, y: i32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    pub fn with_sigma(mut self, sigma: f32) -> Self {
        self.sigma = sigma;
        self
    }

    // =========================================================
    // 3. 核心生成逻辑 (Action)
    // =========================================================

    /// 将当前配置应用到图片上，返回【原图+阴影】的合成图
    pub fn apply_to(&self, img: &DynamicImage) -> DynamicImage {
        let (src_w, src_h) = img.dimensions();
        let sigma = self.sigma;
        let spread_px = self.spread;
        let shadow_color = self.color;

        // 1. 智能降采样 (提升 60MP 图片处理性能的关键)
        let scale_factor = if sigma < 2.0 { 1.0 } 
            else if sigma < 10.0 { 0.5 } 
            else if sigma < 30.0 { 0.25 } 
            else { 0.125 };

        // 2. 计算小图基准尺寸
        let base_tiny_w = (src_w as f32 * scale_factor).ceil();
        let base_tiny_h = (src_h as f32 * scale_factor).ceil();

        // 2.5 应用 Spread (扩散/收缩)
        let tiny_spread = spread_px as f32 * scale_factor;
        // 保证尺寸至少为 1x1
        let tiny_shadow_w = (base_tiny_w + tiny_spread * 2.0).max(1.0).ceil() as u32;
        let tiny_shadow_h = (base_tiny_h + tiny_spread * 2.0).max(1.0).ceil() as u32;

        // 3. 计算模糊 Padding
        let tiny_sigma = sigma * scale_factor;
        let tiny_padding = (tiny_sigma * 3.0).ceil() as u32;

        // 4. 创建小画布
        let tiny_canvas_w = tiny_shadow_w + tiny_padding * 2;
        let tiny_canvas_h = tiny_shadow_h + tiny_padding * 2;
        let mut tiny_map = RgbaImage::new(tiny_canvas_w, tiny_canvas_h);

        // 5. 绘制并染色
        // 注意：这里 resize 到的是包含 spread 的尺寸
        let resized_content = img.resize_exact(tiny_shadow_w, tiny_shadow_h, imageops::FilterType::Nearest);
        
        for (x, y, pixel) in resized_content.pixels() {
            let alpha = pixel[3];
            if alpha > 0 {
                let final_alpha = ((alpha as f32 / 255.0) * (shadow_color[3] as f32 / 255.0) * 255.0) as u8;
                tiny_map.put_pixel(
                    x + tiny_padding, 
                    y + tiny_padding, 
                    Rgba([shadow_color[0], shadow_color[1], shadow_color[2], final_alpha])
                );
            }
        }

        // 6. 极速模糊
        let blurred_tiny = imageops::blur(&tiny_map, tiny_sigma);

        // 7. 放大回原尺寸
        let final_padding = (tiny_padding as f32 / scale_factor).ceil() as u32;
        
        let upscaled_shadow_w = tiny_shadow_w as f32 / scale_factor;
        let upscaled_shadow_h = tiny_shadow_h as f32 / scale_factor;
        
        let final_shadow_w = (upscaled_shadow_w + final_padding as f32 * 2.0).ceil() as u32;
        let final_shadow_h = (upscaled_shadow_h + final_padding as f32 * 2.0).ceil() as u32;

        let shadow_layer = imageops::resize(&blurred_tiny, final_shadow_w, final_shadow_h, imageops::FilterType::Triangle);

        // 8. 坐标计算与合成
        // 计算 shadow_layer 相对于 src_img 的偏移
        let relative_shadow_x = self.offset_x as f32 - (final_shadow_w as f32 - src_w as f32) / 2.0;
        let relative_shadow_y = self.offset_y as f32 - (final_shadow_h as f32 - src_h as f32) / 2.0;

        let abs_shadow_left = relative_shadow_x.round() as i32;
        let abs_shadow_top = relative_shadow_y.round() as i32;
        
        let abs_shadow_right = abs_shadow_left + final_shadow_w as i32;
        let abs_shadow_bottom = abs_shadow_top + final_shadow_h as i32;

        // 计算总画布包围盒
        let canvas_min_x = 0.min(abs_shadow_left);
        let canvas_min_y = 0.min(abs_shadow_top);
        let canvas_max_x = (src_w as i32).max(abs_shadow_right);
        let canvas_max_y = (src_h as i32).max(abs_shadow_bottom);

        let final_w = (canvas_max_x - canvas_min_x) as u32;
        let final_h = (canvas_max_y - canvas_min_y) as u32;

        let mut canvas = RgbaImage::from_pixel(final_w, final_h, Rgba([0, 0, 0, 0]));

        // 贴入阴影
        let paste_shadow_x = abs_shadow_left - canvas_min_x;
        let paste_shadow_y = abs_shadow_top - canvas_min_y;
        imageops::overlay(&mut canvas, &shadow_layer, paste_shadow_x as i64, paste_shadow_y as i64);

        // 贴入原图
        let paste_img_x = 0 - canvas_min_x;
        let paste_img_y = 0 - canvas_min_y;
        imageops::overlay(&mut canvas, img, paste_img_x as i64, paste_img_y as i64);

        DynamicImage::ImageRgba8(canvas)
    }
}