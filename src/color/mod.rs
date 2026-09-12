use digital_canvas::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
    pub const GRAY: Color = Color::new(0.5, 0.5, 0.5, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };
        Some(Self::from_u8(r, g, b, a))
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    pub fn to_u8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    pub fn to_hex(&self) -> String {
        let (r, g, b, a) = self.to_u8();
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    pub fn to_hsv(&self) -> Hsv {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        Hsv::new(h, s, v, self.a)
    }

    pub fn to_hsl(&self) -> Hsl {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let l = (max + min) / 2.0;
        let s = if delta == 0.0 {
            0.0
        } else if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        Hsl::new(h, s, l, self.a)
    }

    pub fn to_cmyk(&self) -> Cmyk {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let k = 1.0 - r.max(g).max(b);
        if k == 1.0 {
            return Cmyk::new(0.0, 0.0, 0.0, 1.0, self.a);
        }

        let c = (1.0 - r - k) / (1.0 - k);
        let m = (1.0 - g - k) / (1.0 - k);
        let y = (1.0 - b - k) / (1.0 - k);

        Cmyk::new(c, m, y, k, self.a)
    }

    pub fn lerp(&self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    pub fn blend(&self, other: Color, mode: digital_canvas::BlendMode) -> Color {
        match mode {
            digital_canvas::BlendMode::Normal => other,
            digital_canvas::BlendMode::Multiply => Color::new(
                self.r * other.r,
                self.g * other.g,
                self.b * other.b,
                self.a * other.a + other.a * (1.0 - self.a),
            ),
            digital_canvas::BlendMode::Screen => Color::new(
                1.0 - (1.0 - self.r) * (1.0 - other.r),
                1.0 - (1.0 - self.g) * (1.0 - other.g),
                1.0 - (1.0 - self.b) * (1.0 - other.b),
                self.a + other.a - self.a * other.a,
            ),
            digital_canvas::BlendMode::Overlay => {
                let mut result = Color::TRANSPARENT;
                for (src, dst) in [(self.r, other.r), (self.g, other.g), (self.b, other.b)] {
                    let val = if dst < 0.5 {
                        2.0 * src * dst
                    } else {
                        1.0 - 2.0 * (1.0 - src) * (1.0 - dst)
                    };
                    // This is simplified - would need proper component-wise
                }
                // Simplified implementation
                other
            }
            _ => other, // Simplified - implement all blend modes
        }
    }

    pub fn premultiply(&self) -> Color {
        Color::new(
            self.r * self.a,
            self.g * self.a,
            self.b * self.a,
            self.a,
        )
    }

    pub fn unpremultiply(&self) -> Color {
        if self.a == 0.0 {
            return *self;
        }
        Color::new(
            self.r / self.a,
            self.g / self.a,
            self.b / self.a,
            self.a,
        )
    }

    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    pub fn is_transparent(&self) -> bool {
        self.a <= 0.0
    }

    pub fn is_opaque(&self) -> bool {
        self.a >= 1.0
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z, 1.0)
    }
}

impl From<Vec4> for Color {
    fn from(v: Vec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self {
            h: h % 360.0,
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let h = self.h / 60.0;
        let c = self.v * self.s;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = self.v - c;

        let (r, g, b) = if h < 1.0 {
            (c, x, 0.0)
        } else if h < 2.0 {
            (x, c, 0.0)
        } else if h < 3.0 {
            (0.0, c, x)
        } else if h < 4.0 {
            (0.0, x, c)
        } else if h < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color::new(r + m, g + m, b + m, self.a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

impl Hsl {
    pub fn new(h: f32, s: f32, l: f32, a: f32) -> Self {
        Self {
            h: h % 360.0,
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let h = self.h / 360.0;
        let s = self.s;
        let l = self.l;

        let hue_to_rgb = |p: f32, q: f32, t: f32| -> f32 {
            let t = if t < 0.0 { t + 1.0 } else if t > 1.0 { t - 1.0 } else { t };
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };

        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        Color::new(r, g, b, self.a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cmyk {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub a: f32,
}

impl Cmyk {
    pub fn new(c: f32, m: f32, y: f32, k: f32, a: f32) -> Self {
        Self {
            c: c.clamp(0.0, 1.0),
            m: m.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            k: k.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn to_rgb(&self) -> Color {
        let r = (1.0 - self.c) * (1.0 - self.k);
        let g = (1.0 - self.m) * (1.0 - self.k);
        let b = (1.0 - self.y) * (1.0 - self.k);
        Color::new(r, g, b, self.a)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub name: String,
    pub colors: Vec<Color>,
}

impl ColorPalette {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            colors: Vec::new(),
        }
    }

    pub fn add_color(&mut self, color: Color) {
        self.colors.push(color);
    }

    pub fn remove_color(&mut self, index: usize) -> Option<Color> {
        if index < self.colors.len() {
            Some(self.colors.remove(index))
        } else {
            None
        }
    }

    pub fn get_color(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorHistory {
    colors: Vec<Color>,
    max_size: usize,
}

impl ColorHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            colors: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, color: Color) {
        self.colors.retain(|c| *c != color);
        self.colors.insert(0, color);
        if self.colors.len() > self.max_size {
            self.colors.truncate(self.max_size);
        }
    }

    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    pub fn clear(&mut self) {
        self.colors.clear();
    }
}
