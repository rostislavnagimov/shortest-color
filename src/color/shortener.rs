use super::converter::convert_to_color;
use super::keywords::COLOR_KEYWORDS;
use super::model::Color;
use std::collections::HashMap;
use std::sync::LazyLock;

#[inline(always)]
fn can_be_short_fast(r: u8, g: u8, b: u8, a: u8) -> bool {
    let r_match = (r & 0x0F) == (r >> 4);
    let g_match = (g & 0x0F) == (g >> 4);
    let b_match = (b & 0x0F) == (b >> 4);
    let a_match = (a & 0x0F) == (a >> 4);
    r_match & g_match & b_match & a_match
}

static COLOR_TO_NAME: LazyLock<HashMap<(u8, u8, u8), &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(COLOR_KEYWORDS.len());
    for &(name, hex_val) in COLOR_KEYWORDS {
        if let Some(color) = convert_to_color(hex_val) {
            if color.a == 255 {
                map.entry((color.r, color.g, color.b)).or_insert(name);
            }
        }
    }
    map
});

#[inline(always)]
fn format_short_hex(r: u8, g: u8, b: u8, a: u8, has_alpha: bool) -> String {
    if has_alpha {
        format!("#{:x}{:x}{:x}{:x}", r / 17, g / 17, b / 17, a / 17)
    } else {
        format!("#{:x}{:x}{:x}", r / 17, g / 17, b / 17)
    }
}

#[inline(always)]
fn format_full_hex(r: u8, g: u8, b: u8, a: u8, has_alpha: bool) -> String {
    if has_alpha {
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    } else {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

#[inline(always)]
pub fn shorten_color(color: &Color) -> String {
    let has_alpha = color.a != 255;
    let can_short = can_be_short_fast(color.r, color.g, color.b, color.a);

    let short_hex_len = if has_alpha { 5 } else { 4 };
    let full_hex_len = if has_alpha { 9 } else { 7 };

    let mut shortest = full_hex_len;
    let mut result = format_full_hex(color.r, color.g, color.b, color.a, has_alpha);

    if can_short && short_hex_len < shortest {
        shortest = short_hex_len;
        result = format_short_hex(color.r, color.g, color.b, color.a, has_alpha);
    }

    if !has_alpha {
        if let Some(name) = COLOR_TO_NAME.get(&(color.r, color.g, color.b)) {
            if name.len() < shortest {
                result = name.to_string();
            }
        }
    }

    result
}

#[inline(always)]
pub fn shorten_hex(hex: &str) -> String {
    if let Some(color) = convert_to_color(hex) {
        shorten_color(&color)
    } else {
        hex.to_string()
    }
}
