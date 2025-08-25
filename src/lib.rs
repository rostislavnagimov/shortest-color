mod color;

pub use color::{parse, shorten, Color};

pub fn shorten_css_color(color_str: &str) -> String {
    let trimmed = color_str.trim().to_ascii_lowercase();

    if trimmed.len() < 5 {
        return if trimmed == "#f00" {
            "red".to_string()
        } else {
            trimmed
        };
    }

    match parse(&trimmed) {
        Some(color) => shorten(&color),
        None => color_str.to_string(),
    }
}
