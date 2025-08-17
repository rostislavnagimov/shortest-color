use super::model::Color;
use super::keywords::COLOR_KEYWORDS;
use super::converter::convert_to_color;

pub fn shorten_color(color: &Color) -> String {
    let mut candidates: Vec<String> = Vec::new();
    
    let full_hex = if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", color.r, color.g, color.b, color.a)
    };
    candidates.push(full_hex);
    
    if color.r % 17 == 0 && color.g % 17 == 0 && color.b % 17 == 0 && color.a % 17 == 0 {
        let short_hex = if color.a == 255 {
            format!("#{:x}{:x}{:x}", color.r / 17, color.g / 17, color.b / 17)
        } else {
            format!("#{:x}{:x}{:x}{:x}", color.r / 17, color.g / 17, color.b / 17, color.a / 17)
        };
        candidates.push(short_hex);
    }
    
    if color.a == 255 {
        for &(name, hex_val) in COLOR_KEYWORDS {
            if let Some(keyword_color) = convert_to_color(hex_val) {
                if *color == keyword_color {
                    candidates.push(name.to_string());
                    break;
                }
            }
        }
    }
    
    candidates
        .into_iter()
        .min_by_key(|c| c.len())
        .unwrap()
}

pub fn shorten_hex(hex: &str) -> String {
    if let Some(color) = convert_to_color(hex) {
        shorten_color(&color)
    } else {
        hex.to_string()
    }
}