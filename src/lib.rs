pub mod color;
pub use color::{try_convert_color, is_valid_color, shorten_color};

pub fn shorten_css_color(color_str: &str) -> String {
    let trimmed = color_str.trim().to_ascii_lowercase();

    if trimmed.len() < 5 {
        if trimmed == "#f00" {
            return "red".to_string();
        }
        return trimmed;
    }

    match try_convert_color(&trimmed) {
        Some(color) => shorten_color(&color),
        None => color_str.to_string(),
    }
}