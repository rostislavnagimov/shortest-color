use super::converter::convert_to_color;
use super::keywords::COLOR_KEYWORDS;
use super::model::Color;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::LazyLock;

#[inline]
fn can_be_short_fast(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) == (r >> 4)
        && (g & 0x0F) == (g >> 4)
        && (b & 0x0F) == (b >> 4)
        && (a & 0x0F) == (a >> 4)
}

static COLOR_TO_NAME: LazyLock<HashMap<(u8, u8, u8), &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
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
pub fn shorten_color(color: &Color) -> String {
    let mut hex_full = String::with_capacity(9);
    let mut hex_short = String::with_capacity(5);

    if color.a == 255 {
        write!(hex_full, "#{:02x}{:02x}{:02x}", color.r, color.g, color.b).unwrap();
    } else {
        write!(
            hex_full,
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )
        .unwrap();
    }

    let has_short = if can_be_short_fast(color.r, color.g, color.b, color.a) {
        if color.a == 255 {
            write!(
                hex_short,
                "#{:x}{:x}{:x}",
                color.r / 17,
                color.g / 17,
                color.b / 17
            )
            .unwrap();
        } else {
            write!(
                hex_short,
                "#{:x}{:x}{:x}{:x}",
                color.r / 17,
                color.g / 17,
                color.b / 17,
                color.a / 17
            )
            .unwrap();
        }
        true
    } else {
        false
    };

    let color_name = if color.a == 255 {
        COLOR_TO_NAME.get(&(color.r, color.g, color.b)).copied()
    } else {
        None
    };

    let mut shortest_len = hex_full.len();
    let mut result = &hex_full;

    if has_short && hex_short.len() < shortest_len {
        shortest_len = hex_short.len();
        result = &hex_short;
    }

    if let Some(name) = color_name {
        if name.len() < shortest_len {
            return name.to_string();
        }
    }

    result.to_string()
}

#[inline(always)]
pub fn shorten_hex(hex: &str) -> String {
    if let Some(color) = convert_to_color(hex) {
        shorten_color(&color)
    } else {
        hex.to_string()
    }
}
