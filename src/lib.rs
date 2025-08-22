pub mod color;
pub use color::{convert_to_color, is_valid_color, shorten_color};

pub fn shorten_css_color(color_str: &str) -> String {
    let trimmed = color_str.trim().to_ascii_lowercase();

    if trimmed.len() < 5 {
        return if trimmed == "#f00" {
            "red".to_string()
        } else {
            trimmed
        };
    }

    if !is_valid_color(&trimmed) {
        return color_str.to_string();
    }

    if let Some(color) = convert_to_color(&trimmed) {
        shorten_color(&color)
    } else {
        trimmed
    }
}
