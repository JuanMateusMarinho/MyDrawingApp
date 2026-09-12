#[cfg(test)]
mod tests {
    use digital_canvas::{Color, Vec2, Rect, BlendMode, LayerId, DocumentId};

    #[test]
    fn test_color_creation() {
        let color = Color::new(1.0, 0.5, 0.0, 1.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::from_u8(255, 128, 0, 255);
        assert_eq!(color.to_hex(), "#FF8000");
    }

    #[test]
    fn test_color_hsv() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0); // Red
        let hsv = color.to_hsv();
        assert!((hsv.h - 0.0).abs() < 1.0);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_hsl() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0); // Red
        let hsl = color.to_hsl();
        assert!((hsl.h - 0.0).abs() < 1.0);
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_cmyk() {
        let color = Color::new(1.0, 0.0, 0.0, 1.0); // Red
        let cmyk = color.to_cmyk();
        assert!((cmyk.c - 0.0).abs() < 0.01);
        assert!((cmyk.m - 1.0).abs() < 0.01);
        assert!((cmyk.y - 1.0).abs() < 0.01);
        assert!((cmyk.k - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        
        let dist = a.distance(b);
        assert!((dist - 2.828).abs() < 0.01); // sqrt(8)
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(rect.contains(Vec2::new(50.0, 50.0)));
        assert!(!rect.contains(Vec2::new(150.0, 50.0)));
        assert!(!rect.contains(Vec2::new(-10.0, 50.0)));
    }

    #[test]
    fn test_blend_modes() {
        let dst = Color::new(0.5, 0.5, 0.5, 1.0);
        let src = Color::new(1.0, 0.0, 0.0, 1.0);
        
        let multiply = digital_canvas::color::blend_multiply(dst, src);
        assert!((multiply.r - 0.5).abs() < 0.01);
        assert!((multiply.g - 0.0).abs() < 0.01);
        assert!((multiply.b - 0.0).abs() < 0.01);
        
        let screen = digital_canvas::color::blend_screen(dst, src);
        assert!((screen.r - 1.0).abs() < 0.01);
        assert!((screen.g - 0.5).abs() < 0.01);
        assert!((screen.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_entity_ids() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_layer_id() {
        let layer_id = LayerId::new();
        let doc_id = DocumentId::new();
        assert_ne!(layer_id.0, doc_id.0);
    }

    #[test]
    fn test_blend_mode_str() {
        assert_eq!(BlendMode::Normal.as_str(), "Normal");
        assert_eq!(BlendMode::Multiply.as_str(), "Multiply");
        assert_eq!(BlendMode::Overlay.as_str(), "Overlay");
    }
}