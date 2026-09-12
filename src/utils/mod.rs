use digital_canvas::{Color, Vec2, Rect};
use anyhow::Result;
use image::{DynamicImage, ImageFormat, GenericImageView, Rgba, ImageBuffer, ImageRgba8};
use std::sync::Arc;

type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub mod math {
    use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat};

    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }

    pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }

    pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            a.a + (b.a - a.a) * t,
        )
    }

    pub fn smooth_step(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn smoother_step(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    pub fn distance_point_to_line(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        let line_len = line_vec.length();
        if line_len == 0.0 {
            return point_vec.length();
        }
        let cross = line_vec.x * point_vec.y - line_vec.y * point_vec.x;
        cross.abs() / line_len
    }

    pub fn point_in_rect(point: Vec2, rect: Rect) -> bool {
        point.x >= rect.x && point.x <= rect.x + rect.width &&
        point.y >= rect.y && point.y <= rect.y + rect.height
    }

    pub fn rect_intersect(a: Rect, b: Rect) -> Option<Rect> {
        let x1 = a.x.max(b.x);
        let y1 = a.y.max(b.y);
        let x2 = (a.x + a.width).min(b.x + b.width);
        let y2 = (a.y + a.height).min(b.y + b.height);
        
        if x1 < x2 && y1 < y2 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    pub fn rect_union(a: Rect, b: Rect) -> Rect {
        let x1 = a.x.min(b.x);
        let y1 = a.y.min(b.y);
        let x2 = (a.x + a.width).max(b.x + b.width);
        let y2 = (a.y + a.height).max(b.y + b.height);
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
        let t2 = t * t;
        let t3 = t2 * t;
        
        0.5 * (
            (2.0 * p1) +
            (-p0 + p2) * t +
            (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 +
            (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
        )
    }

    pub fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
        let u = 1.0 - t;
        let u2 = u * u;
        let u3 = u2 * u;
        let t2 = t * t;
        let t3 = t2 * t;
        
        u3 * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t3 * p3
    }
}

pub mod color {
    use super::*;

    pub fn blend_normal(dst: Color, src: Color) -> Color {
        let a = src.a + dst.a * (1.0 - src.a);
        if a == 0.0 {
            return Color::TRANSPARENT;
        }
        Color::new(
            (src.r * src.a + dst.r * dst.a * (1.0 - src.a)) / a,
            (src.g * src.a + dst.g * dst.a * (1.0 - src.a)) / a,
            (src.b * src.a + dst.b * dst.a * (1.0 - src.a)) / a,
            a,
        )
    }

    pub fn blend_multiply(dst: Color, src: Color) -> Color {
        Color::new(
            dst.r * src.r,
            dst.g * src.g,
            dst.b * src.b,
            dst.a,
        )
    }

    pub fn blend_screen(dst: Color, src: Color) -> Color {
        Color::new(
            1.0 - (1.0 - dst.r) * (1.0 - src.r),
            1.0 - (1.0 - dst.g) * (1.0 - src.g),
            1.0 - (1.0 - dst.b) * (1.0 - src.b),
            dst.a,
        )
    }

    pub fn blend_overlay(dst: Color, src: Color) -> Color {
        let f = |d: f32, s: f32| -> f32 {
            if d < 0.5 {
                2.0 * d * s
            } else {
                1.0 - 2.0 * (1.0 - d) * (1.0 - s)
            }
        };
        Color::new(
            f(dst.r, src.r),
            f(dst.g, src.g),
            f(dst.b, src.b),
            dst.a,
        )
    }

    pub fn blend_soft_light(dst: Color, src: Color) -> Color {
        let f = |d: f32, s: f32| -> f32 {
            if s < 0.5 {
                d - (1.0 - 2.0 * s) * d * (1.0 - d)
            } else {
                d + (2.0 * s - 1.0) * (f_sqrt(d) - d)
            }
        };
        fn f_sqrt(x: f32) -> f32 {
            if x < 0.25 { ((16.0 * x - 12.0) * x + 4.0) * x } else { x.sqrt() }
        }
        Color::new(
            f(dst.r, src.r),
            f(dst.g, src.g),
            f(dst.b, src.b),
            dst.a,
        )
    }

    pub fn blend_hard_light(dst: Color, src: Color) -> Color {
        let f = |d: f32, s: f32| -> f32 {
            if s < 0.5 {
                2.0 * d * s
            } else {
                1.0 - 2.0 * (1.0 - d) * (1.0 - s)
            }
        };
        Color::new(
            f(dst.r, src.r),
            f(dst.g, src.g),
            f(dst.b, src.b),
            dst.a,
        )
    }

    pub fn blend_color_dodge(dst: Color, src: Color) -> Color {
        let f = |d: f32, s: f32| -> f32 {
            if s == 1.0 { 1.0 } else { (d / (1.0 - s)).min(1.0) }
        };
        Color::new(
            f(dst.r, src.r),
            f(dst.g, src.g),
            f(dst.b, src.b),
            dst.a,
        )
    }

    pub fn blend_color_burn(dst: Color, src: Color) -> Color {
        let f = |d: f32, s: f32| -> f32 {
            if s == 0.0 { 0.0 } else { 1.0 - (1.0 - d) / s }
        };
        Color::new(
            f(dst.r, src.r),
            f(dst.g, src.g),
            f(dst.b, src.b),
            dst.a,
        )
    }

    pub fn blend_darken(dst: Color, src: Color) -> Color {
        Color::new(
            dst.r.min(src.r),
            dst.g.min(src.g),
            dst.b.min(src.b),
            dst.a,
        )
    }

    pub fn blend_lighten(dst: Color, src: Color) -> Color {
        Color::new(
            dst.r.max(src.r),
            dst.g.max(src.g),
            dst.b.max(src.b),
            dst.a,
        )
    }

    pub fn blend_difference(dst: Color, src: Color) -> Color {
        Color::new(
            (dst.r - src.r).abs(),
            (dst.g - src.g).abs(),
            (dst.b - src.b).abs(),
            dst.a,
        )
    }

    pub fn blend_exclusion(dst: Color, src: Color) -> Color {
        Color::new(
            dst.r + src.r - 2.0 * dst.r * src.r,
            dst.g + src.g - 2.0 * dst.g * src.g,
            dst.b + src.b - 2.0 * dst.b * src.b,
            dst.a,
        )
    }

    pub fn blend_hue(dst: Color, src: Color) -> Color {
        let dst_hsl = dst.to_hsl();
        let src_hsl = src.to_hsl();
        Hsl::new(src_hsl.h, dst_hsl.s, dst_hsl.l, dst.a).to_rgb()
    }

    pub fn blend_saturation(dst: Color, src: Color) -> Color {
        let dst_hsl = dst.to_hsl();
        let src_hsl = src.to_hsl();
        Hsl::new(dst_hsl.h, src_hsl.s, dst_hsl.l, dst.a).to_rgb()
    }

    pub fn blend_color(dst: Color, src: Color) -> Color {
        let dst_hsl = dst.to_hsl();
        let src_hsl = src.to_hsl();
        Hsl::new(src_hsl.h, src_hsl.s, dst_hsl.l, dst.a).to_rgb()
    }

    pub fn blend_luminosity(dst: Color, src: Color) -> Color {
        let dst_hsl = dst.to_hsl();
        let src_hsl = src.to_hsl();
        Hsl::new(dst_hsl.h, dst_hsl.s, src_hsl.l, dst.a).to_rgb()
    }

    pub fn apply_blend_mode(dst: Color, src: Color, mode: digital_canvas::BlendMode) -> Color {
        match mode {
            digital_canvas::BlendMode::Normal => blend_normal(dst, src),
            digital_canvas::BlendMode::Multiply => blend_multiply(dst, src),
            digital_canvas::BlendMode::Screen => blend_screen(dst, src),
            digital_canvas::BlendMode::Overlay => blend_overlay(dst, src),
            digital_canvas::BlendMode::SoftLight => blend_soft_light(dst, src),
            digital_canvas::BlendMode::HardLight => blend_hard_light(dst, src),
            digital_canvas::BlendMode::ColorDodge => blend_color_dodge(dst, src),
            digital_canvas::BlendMode::ColorBurn => blend_color_burn(dst, src),
            digital_canvas::BlendMode::Darken => blend_darken(dst, src),
            digital_canvas::BlendMode::Lighten => blend_lighten(dst, src),
            digital_canvas::BlendMode::Difference => blend_difference(dst, src),
            digital_canvas::BlendMode::Exclusion => blend_exclusion(dst, src),
            digital_canvas::BlendMode::Hue => blend_hue(dst, src),
            digital_canvas::BlendMode::Saturation => blend_saturation(dst, src),
            digital_canvas::BlendMode::Color => blend_color(dst, src),
            digital_canvas::BlendMode::Luminosity => blend_luminosity(dst, src),
        }
    }
}

pub mod image {
    use super::*;

    pub fn resize_nearest(img: &RgbaImage, new_width: u32, new_height: u32) -> RgbaImage {
        let mut result = RgbaImage::new(new_width, new_height);
        let x_ratio = img.width() as f32 / new_width as f32;
        let y_ratio = img.height() as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * x_ratio) as u32;
                let src_y = (y as f32 * y_ratio) as u32;
                let pixel = img.get_pixel(src_x.min(img.width() - 1), src_y.min(img.height() - 1));
                result.put_pixel(x, y, *pixel);
            }
        }
        result
    }

    pub fn resize_bilinear(img: &RgbaImage, new_width: u32, new_height: u32) -> RgbaImage {
        let mut result = RgbaImage::new(new_width, new_height);
        let x_ratio = (img.width() - 1) as f32 / (new_width - 1) as f32;
        let y_ratio = (img.height() - 1) as f32 / (new_height - 1) as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = x as f32 * x_ratio;
                let src_y = y as f32 * y_ratio;
                
                let x0 = src_x.floor() as u32;
                let x1 = (x0 + 1).min(img.width() - 1);
                let y0 = src_y.floor() as u32;
                let y1 = (y0 + 1).min(img.height() - 1);
                
                let dx = src_x - x0 as f32;
                let dy = src_y - y0 as f32;
                
                let p00 = img.get_pixel(x0, y0);
                let p01 = img.get_pixel(x1, y0);
                let p10 = img.get_pixel(x0, y1);
                let p11 = img.get_pixel(x1, y1);
                
                let r = bilinear_interp(p00[0], p01[0], p10[0], p11[0], dx, dy);
                let g = bilinear_interp(p00[1], p01[1], p10[1], p11[1], dx, dy);
                let b = bilinear_interp(p00[2], p01[2], p10[2], p11[2], dx, dy);
                let a = bilinear_interp(p00[3], p01[3], p10[3], p11[3], dx, dy);
                
                result.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
        result
    }

    fn bilinear_interp(p00: u8, p01: u8, p10: u8, p11: u8, dx: f32, dy: f32) -> u8 {
        let v0 = p00 as f32 * (1.0 - dx) + p01 as f32 * dx;
        let v1 = p10 as f32 * (1.0 - dx) + p11 as f32 * dx;
        (v0 * (1.0 - dy) + v1 * dy).round() as u8
    }

    pub fn gaussian_blur(img: &RgbaImage, radius: f32) -> RgbaImage {
        let sigma = radius / 2.0;
        let kernel_size = (radius * 3.0).ceil() as u32 * 2 + 1;
        let kernel = gaussian_kernel(kernel_size, sigma);
        
        // Horizontal pass
        let mut temp = RgbaImage::new(img.width(), img.height());
        for y in 0..img.height() {
            for x in 0..img.width() {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                let mut a = 0.0f32;
                let mut weight_sum = 0.0f32;
                
                for (i, &weight) in kernel.iter().enumerate() {
                    let kx = x as i32 + i as i32 - kernel.len() as i32 / 2;
                    let kx = kx.clamp(0, img.width() as i32 - 1) as u32;
                    let pixel = img.get_pixel(kx, y);
                    r += pixel[0] as f32 * weight;
                    g += pixel[1] as f32 * weight;
                    b += pixel[2] as f32 * weight;
                    a += pixel[3] as f32 * weight;
                    weight_sum += weight;
                }
                
                temp.put_pixel(x, y, Rgba([
                    (r / weight_sum) as u8,
                    (g / weight_sum) as u8,
                    (b / weight_sum) as u8,
                    (a / weight_sum) as u8,
                ]));
            }
        }
        
        // Vertical pass
        let mut result = RgbaImage::new(img.width(), img.height());
        for y in 0..img.height() {
            for x in 0..img.width() {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                let mut a = 0.0f32;
                let mut weight_sum = 0.0f32;
                
                for (i, &weight) in kernel.iter().enumerate() {
                    let ky = y as i32 + i as i32 - kernel.len() as i32 / 2;
                    let ky = ky.clamp(0, img.height() as i32 - 1) as u32;
                    let pixel = temp.get_pixel(x, ky);
                    r += pixel[0] as f32 * weight;
                    g += pixel[1] as f32 * weight;
                    b += pixel[2] as f32 * weight;
                    a += pixel[3] as f32 * weight;
                    weight_sum += weight;
                }
                
                result.put_pixel(x, y, Rgba([
                    (r / weight_sum) as u8,
                    (g / weight_sum) as u8,
                    (b / weight_sum) as u8,
                    (a / weight_sum) as u8,
                ]));
            }
        }
        
        result
    }

    fn gaussian_kernel(size: u32, sigma: f32) -> Vec<f32> {
        let radius = size / 2;
        let mut kernel = Vec::with_capacity(size as usize);
        let two_sigma_sq = 2.0 * sigma * sigma;
        
        for i in 0..size {
            let x = i as i32 - radius as i32;
            let weight = (-(x * x) as f32 / two_sigma_sq).exp();
            kernel.push(weight);
        }
        
        // Normalize
        let sum: f32 = kernel.iter().sum();
        for w in &mut kernel {
            *w /= sum;
        }
        
        kernel
    }

    pub fn adjust_brightness_contrast(img: &mut RgbaImage, brightness: f32, contrast: f32) {
        let contrast_factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));
        
        for pixel in img.pixels_mut() {
            let r = ((pixel[0] as f32 - 128.0) * contrast_factor + 128.0 + brightness * 255.0).clamp(0.0, 255.0) as u8;
            let g = ((pixel[1] as f32 - 128.0) * contrast_factor + 128.0 + brightness * 255.0).clamp(0.0, 255.0) as u8;
            let b = ((pixel[2] as f32 - 128.0) * contrast_factor + 128.0 + brightness * 255.0).clamp(0.0, 255.0) as u8;
            *pixel = Rgba([r, g, b, pixel[3]]);
        }
    }

    pub fn adjust_hue_saturation(img: &mut RgbaImage, hue_shift: f32, saturation: f32) {
        for pixel in img.pixels_mut() {
            let mut color = Color::from_u8(pixel[0], pixel[1], pixel[2], pixel[3]);
            let mut hsl = color.to_hsl();
            hsl.h = (hsl.h + hue_shift) % 360.0;
            hsl.s = (hsl.s * (1.0 + saturation)).clamp(0.0, 1.0);
            color = hsl.to_rgb();
            let (r, g, b, a) = color.to_u8();
            *pixel = Rgba([r, g, b, a]);
        }
    }

    pub fn invert(img: &mut RgbaImage) {
        for pixel in img.pixels_mut() {
            pixel[0] = 255 - pixel[0];
            pixel[1] = 255 - pixel[1];
            pixel[2] = 255 - pixel[2];
        }
    }

    pub fn threshold(img: &mut RgbaImage, threshold: u8) {
        for pixel in img.pixels_mut() {
            let gray = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
            let value = if gray > threshold { 255 } else { 0 };
            pixel[0] = value;
            pixel[1] = value;
            pixel[2] = value;
        }
    }

    pub fn posterize(img: &mut RgbaImage, levels: u8) {
        let step = 256 / levels as u32;
        for pixel in img.pixels_mut() {
            pixel[0] = (pixel[0] as u32 / step * step) as u8;
            pixel[1] = (pixel[1] as u32 / step * step) as u8;
            pixel[2] = (pixel[2] as u32 / step * step) as u8;
        }
    }
}

pub mod stroke {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Stroke {
        pub points: Vec<StrokePoint>,
        pub brush_id: u64,
        pub color: Color,
        pub blend_mode: digital_canvas::BlendMode,
    }

    #[derive(Debug, Clone)]
    pub struct StrokePoint {
        pub position: Vec2,
        pub pressure: f32,
        pub tilt: Vec2,
        pub rotation: f32,
        pub timestamp: f64,
    }

    impl Stroke {
        pub fn new(brush_id: u64, color: Color, blend_mode: digital_canvas::BlendMode) -> Self {
            Self {
                points: Vec::new(),
                brush_id,
                color,
                blend_mode,
            }
        }

        pub fn add_point(&mut self, point: StrokePoint) {
            self.points.push(point);
        }

        pub fn length(&self) -> f32 {
            if self.points.len() < 2 {
                return 0.0;
            }
            let mut len = 0.0;
            for i in 1..self.points.len() {
                len += self.points[i].position.distance(self.points[i-1].position);
            }
            len
        }

        pub fn smooth(&mut self, smoothing: f32) {
            if self.points.len() < 3 {
                return;
            }
            
            let mut smoothed = Vec::with_capacity(self.points.len());
            smoothed.push(self.points[0].clone());
            
            for i in 1..self.points.len() - 1 {
                let prev = self.points[i - 1].position;
                let curr = self.points[i].position;
                let next = self.points[i + 1].position;
                
                let smoothed_pos = prev * (1.0 - smoothing) + next * smoothing;
                smoothed.push(StrokePoint {
                    position: curr * (1.0 - smoothing) + smoothed_pos * smoothing,
                    pressure: self.points[i].pressure,
                    tilt: self.points[i].tilt,
                    rotation: self.points[i].rotation,
                    timestamp: self.points[i].timestamp,
                });
            }
            
            smoothed.push(self.points[self.points.len() - 1].clone());
            self.points = smoothed;
        }

        pub fn stabilize(&mut self, stabilization: f32) {
            // More aggressive smoothing for stabilization
            self.smooth(stabilization * 0.8);
        }

        pub fn resample(&mut self, spacing: f32) {
            if self.points.len() < 2 || spacing <= 0.0 {
                return;
            }
            
            let mut resampled = Vec::new();
            resampled.push(self.points[0].clone());
            
            let mut accumulated = 0.0;
            for i in 1..self.points.len() {
                let dist = self.points[i].position.distance(self.points[i-1].position);
                accumulated += dist;
                
                while accumulated >= spacing {
                    let t = (dist - accumulated + spacing) / dist;
                    let pos = math::lerp_vec2(self.points[i-1].position, self.points[i].position, t);
                    let pressure = math::lerp(self.points[i-1].pressure, self.points[i].pressure, t);
                    
                    resampled.push(StrokePoint {
                        position: pos,
                        pressure,
                        tilt: self.points[i-1].tilt,
                        rotation: self.points[i-1].rotation,
                        timestamp: self.points[i-1].timestamp,
                    });
                    
                    accumulated -= spacing;
                }
            }
            
            resampled.push(self.points[self.points.len() - 1].clone());
            self.points = resampled;
        }
    }
}

pub mod serialization {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct SerializableColor {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    impl From<Color> for SerializableColor {
        fn from(c: Color) -> Self {
            Self { r: c.r, g: c.g, b: c.b, a: c.a }
        }
    }

    impl From<SerializableColor> for Color {
        fn from(c: SerializableColor) -> Self {
            Color::new(c.r, c.g, c.b, c.a)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct SerializableVec2 {
        pub x: f32,
        pub y: f32,
    }

    impl From<Vec2> for SerializableVec2 {
        fn from(v: Vec2) -> Self {
            Self { x: v.x, y: v.y }
        }
    }

    impl From<SerializableVec2> for Vec2 {
        fn from(v: SerializableVec2) -> Self {
            Vec2::new(v.x, v.y)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct SerializableRect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    impl From<Rect> for SerializableRect {
        fn from(r: Rect) -> Self {
            Self { x: r.x, y: r.y, width: r.width, height: r.height }
        }
    }

    impl From<SerializableRect> for Rect {
        fn from(r: SerializableRect) -> Self {
            Rect::new(r.x, r.y, r.width, r.height)
        }
    }
}

pub mod performance {
    use std::time::Instant;

    pub struct FrameTimer {
        frame_times: Vec<f32>,
        max_samples: usize,
        last_frame: Instant,
    }

    impl FrameTimer {
        pub fn new(max_samples: usize) -> Self {
            Self {
                frame_times: Vec::with_capacity(max_samples),
                max_samples,
                last_frame: Instant::now(),
            }
        }

        pub fn tick(&mut self) -> f32 {
            let now = Instant::now();
            let dt = now.duration_since(self.last_frame).as_secs_f32();
            self.last_frame = now;
            
            self.frame_times.push(dt);
            if self.frame_times.len() > self.max_samples {
                self.frame_times.remove(0);
            }
            
            dt
        }

        pub fn average_fps(&self) -> f32 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            1.0 / avg_frame_time
        }

        pub fn average_frame_time_ms(&self) -> f32 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32 * 1000.0
        }

        pub fn min_frame_time_ms(&self) -> f32 {
            self.frame_times.iter().fold(f32::INFINITY, |a, &b| a.min(b)) * 1000.0
        }

        pub fn max_frame_time_ms(&self) -> f32 {
            self.frame_times.iter().fold(0.0f32, |a, &b| a.max(b)) * 1000.0
        }
    }

    pub struct Profiler {
        sections: std::collections::HashMap<String, SectionTiming>,
    }

    #[derive(Default)]
    struct SectionTiming {
        total_time: f32,
        count: u32,
        min_time: f32,
        max_time: f32,
    }

    impl Profiler {
        pub fn new() -> Self {
            Self {
                sections: std::collections::HashMap::new(),
            }
        }

        pub fn begin_section(&mut self, name: &str) -> SectionGuard {
            SectionGuard {
                profiler: self,
                name: name.to_string(),
                start: Instant::now(),
            }
        }

        fn end_section(&mut self, name: &str, elapsed: f32) {
            let section = self.sections.entry(name.to_string()).or_default();
            section.total_time += elapsed;
            section.count += 1;
            section.min_time = section.min_time.min(elapsed);
            section.max_time = section.max_time.max(elapsed);
        }

        pub fn report(&self) -> String {
            let mut report = String::new();
            report.push_str("Profiler Report:\n");
            report.push_str("----------------\n");
            
            let mut sections: Vec<_> = self.sections.iter().collect();
            sections.sort_by(|a, b| b.1.total_time.partial_cmp(&a.1.total_time).unwrap());
            
            for (name, timing) in sections {
                let avg = timing.total_time / timing.count as f32;
                report.push_str(&format!(
                    "  {}: {:.2}ms avg ({:.2}ms min, {:.2}ms max) x{}\n",
                    name,
                    avg * 1000.0,
                    timing.min_time * 1000.0,
                    timing.max_time * 1000.0,
                    timing.count
                ));
            }
            
            report
        }
    }

    pub struct SectionGuard<'a> {
        profiler: &'a mut Profiler,
        name: String,
        start: Instant,
    }

    impl Drop for SectionGuard<'_> {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed().as_secs_f32();
            self.profiler.end_section(&self.name, elapsed);
        }
    }
}