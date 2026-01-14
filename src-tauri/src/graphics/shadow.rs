use image::{Rgba, RgbaImage, imageops};


/// 阴影配置模板
/// 这里的参数基于 "基准尺寸 (Reference Size = 1000px)"
#[derive(Debug, Clone, Copy)]
pub struct ShadowProfile {
    pub sigma: f32,       // 基准模糊
    pub offset_x: i32,    // 基准偏移 X
    pub offset_y: i32,    // 基准偏移 Y
    pub spread: i32,      // 基准扩散
    pub color: Rgba<u8>,  // 颜色
}

#[allow(dead_code)]
impl ShadowProfile {
    // =========================================================
    // 1. 预设工厂 (Presets)
    // =========================================================

    pub fn preset_subtle() -> Self {
        Self { sigma: 10.0, offset_x: 0, offset_y: 10, spread: -2, color: Rgba([0, 0, 0, 160]) }
    }

    pub fn preset_standard() -> Self {
        Self { sigma: 15.0, offset_x: 0, offset_y: 15, spread: -5, color: Rgba([0, 0, 0, 190]) }
    }

    pub fn preset_floating() -> Self {
        Self { sigma: 25.0, offset_x: 0, offset_y: 30, spread: -8, color: Rgba([0, 0, 0, 210]) }
    }
    
    // 如果需要自定义，可以使用 new
    pub fn new(sigma: f32, offset: (i32, i32), spread: i32, color: Rgba<u8>) -> Self {
        Self { sigma, offset_x: offset.0, offset_y: offset.1, spread, color }
    }

    // =========================================================
    // 2. 核心绘制 API (只操作现有画布，不分配新内存)
    // =========================================================

    /// 🟢 [智能 API] 自适应绘制
    /// 唯一的对外公开绘制接口。
    /// 自动根据 target 画布大小缩放参数，然后调用底层高性能绘制。
    pub fn draw_adaptive_shadow_on(
        &self,
        target: &mut RgbaImage,
        src_dims: (u32, u32),
        center_pos: (i64, i64),
    ) {
        let (canvas_w, canvas_h) = target.dimensions();
        const REF_SIZE: f32 = 1000.0;
        
        let current_max_dim = canvas_w.max(canvas_h) as f32;
        let ratio = current_max_dim / REF_SIZE;

        // 临时计算运行时参数
        // 注意：这里我们不需要创建一个新的 Struct，直接传参给底层函数即可
        // 但为了代码复用，创建一个临时对象也可以，开销极小
        let effective_profile = Self {
            sigma: self.sigma * ratio,
            offset_x: (self.offset_x as f32 * ratio) as i32,
            offset_y: (self.offset_y as f32 * ratio) as i32,
            spread: (self.spread as f32 * ratio) as i32,
            color: self.color,
        };

        effective_profile.draw_raw_shadow_on(target, src_dims, center_pos.0, center_pos.1);
    }

    /// 🔒 [底层 API] 原始绘制 (Raw Drawing)
    /// 恒定时间复杂度，仅供内部调用，或者当你非常确定参数已经适配过时调用
    fn draw_raw_shadow_on(
        &self, 
        target: &mut RgbaImage, 
        src_dims: (u32, u32), 
        center_x: i64, 
        center_y: i64
    ) {
        let (src_w, src_h) = src_dims;
        
        // --- 1. 动态缩放 (恒定 500px 计算限制) ---
        const INTERNAL_LIMIT: f32 = 500.0;
        let max_dim = std::cmp::max(src_w, src_h) as f32;
        let scale_factor = if max_dim > INTERNAL_LIMIT {
            INTERNAL_LIMIT / max_dim
        } else {
            1.0
        };

        // --- 2. 参数计算 ---
        let tiny_w = (src_w as f32 * scale_factor).ceil() as u32;
        let tiny_h = (src_h as f32 * scale_factor).ceil() as u32;
        let tiny_spread = self.spread as f32 * scale_factor;
        let tiny_sigma = self.sigma * scale_factor;
        
        let shadow_rect_w = (tiny_w as f32 + tiny_spread * 2.0).max(1.0).ceil() as u32;
        let shadow_rect_h = (tiny_h as f32 + tiny_spread * 2.0).max(1.0).ceil() as u32;
        let padding = (tiny_sigma * 3.0).ceil() as u32;
        
        let canvas_w = shadow_rect_w + padding * 2;
        let canvas_h = shadow_rect_h + padding * 2;

        // --- 3. 绘制小黑块 ---
        let mut tiny_map = RgbaImage::new(canvas_w, canvas_h);
        let fill_x = padding;
        let fill_y = padding;
        let alpha = self.color[3];
        let paint_pixel = Rgba([self.color[0], self.color[1], self.color[2], alpha]);
        
        for y in fill_y..(fill_y + shadow_rect_h) {
            for x in fill_x..(fill_x + shadow_rect_w) {
                tiny_map.put_pixel(x, y, paint_pixel);
            }
        }

        // --- 4. 模糊 ---
        let blurred_tiny = imageops::blur(&tiny_map, tiny_sigma);

        // --- 5. 放大 ---
        let final_shadow_w = (canvas_w as f32 / scale_factor).ceil() as u32;
        let final_shadow_h = (canvas_h as f32 / scale_factor).ceil() as u32;

        let shadow_layer = imageops::resize(
            &blurred_tiny, 
            final_shadow_w, 
            final_shadow_h, 
            imageops::FilterType::Triangle 
        );

        // --- 6. 贴图 ---
        let paste_x = center_x as f32 + self.offset_x as f32 - (final_shadow_w as f32 / 2.0);
        let paste_y = center_y as f32 + self.offset_y as f32 - (final_shadow_h as f32 / 2.0);

        imageops::overlay(target, &shadow_layer, paste_x.round() as i64, paste_y.round() as i64);
    }
}