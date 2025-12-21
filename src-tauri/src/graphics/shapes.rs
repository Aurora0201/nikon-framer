use image::{ImageBuffer, Rgba};
use imageproc::drawing::{draw_filled_rect_mut, draw_filled_circle_mut};
use imageproc::rect::Rect;

// 🟢 绘制实心圆角矩形
pub fn draw_rounded_rect_mut(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    rect: Rect,
    radius: i32,
    color: Rgba<u8>
) {
    let (x, y) = (rect.left(), rect.top());
    let (w, h) = (rect.width(), rect.height());
    let r = radius.min((w as i32) / 2).min((h as i32) / 2);

    let rect_h = Rect::at(x + r, y).of_size(w - (2 * r as u32), h);
    let rect_v = Rect::at(x, y + r).of_size(w, h - (2 * r as u32));
    draw_filled_rect_mut(image, rect_h, color);
    draw_filled_rect_mut(image, rect_v, color);

    draw_filled_circle_mut(image, (x + r, y + r), r, color);
    draw_filled_circle_mut(image, (x + (w as i32) - r - 1, y + r), r, color);
    draw_filled_circle_mut(image, (x + r, y + (h as i32) - r - 1), r, color);
    draw_filled_circle_mut(image, (x + (w as i32) - r - 1, y + (h as i32) - r - 1), r, color);
}