use super::converter::convert_to_color;
use super::keywords::COLOR_KEYWORDS;
use super::model::Color;
use std::collections::HashMap;
use std::sync::LazyLock;

#[inline(always)]
fn can_shorten_hex(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) == (r >> 4)
        && (g & 0x0F) == (g >> 4)
        && (b & 0x0F) == (b >> 4)
        && (a & 0x0F) == (a >> 4)
}

static COLOR_TO_NAME: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(COLOR_KEYWORDS.len());
    for &(name, hex_val) in COLOR_KEYWORDS {
        if let Some(color) = convert_to_color(hex_val) {
            if color.a == 255 {
                let rgb = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
                map.entry(rgb).or_insert(name);
            }
        }
    }
    map
});

#[inline(always)]
fn color_to_u32(color: &Color) -> u32 {
    ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
}

pub fn shorten_color(color: &Color) -> String {
    let has_alpha = color.a != 255;
    let can_short = can_shorten_hex(color.r, color.g, color.b, color.a);

    if has_alpha {
        if can_short {
            format!(
                "#{:x}{:x}{:x}{:x}",
                color.r >> 4,
                color.g >> 4,
                color.b >> 4,
                color.a >> 4
            )
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                color.r, color.g, color.b, color.a
            )
        }
    } else {
        let short_hex_len = 4;
        let full_hex_len = 7;
        let mut shortest = full_hex_len;
        let mut result = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);

        if can_short && short_hex_len < shortest {
            shortest = short_hex_len;
            result = format!("#{:x}{:x}{:x}", color.r >> 4, color.g >> 4, color.b >> 4);
        }

        let rgb = color_to_u32(color);
        if let Some(&name) = COLOR_TO_NAME.get(&rgb) {
            if name.len() < shortest {
                result = name.to_string();
            }
        }

        result
    }
}

#[inline(always)]
pub fn shorten_hex(hex: &str) -> String {
    if let Some(color) = convert_to_color(hex) {
        shorten_color(&color)
    } else {
        hex.to_string()
    }
}
