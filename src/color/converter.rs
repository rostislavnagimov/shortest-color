use super::keywords::COLOR_KEYWORDS;
use super::model::Color;
use std::collections::HashMap;
use std::sync::LazyLock;

const HEX_MAP: [u8; 256] = {
    let mut map = [255; 256];
    let mut i = b'0';
    while i <= b'9' {
        map[i as usize] = i - b'0';
        i += 1;
    }
    i = b'A';
    while i <= b'F' {
        map[i as usize] = i - b'A' + 10;
        i += 1;
    }
    i = b'a';
    while i <= b'f' {
        map[i as usize] = i - b'a' + 10;
        i += 1;
    }
    map
};

#[inline(always)]
fn hex_val(b: u8) -> Option<u8> {
    let val = unsafe { *HEX_MAP.get_unchecked(b as usize) };
    if val > 15 { None } else { Some(val) }
}

#[inline(always)]
fn parse_hex2(bytes: &[u8], i: usize) -> Option<u8> {
    let high = hex_val(bytes[i])?;
    let low = hex_val(bytes[i + 1])?;
    Some((high << 4) | low)
}

#[inline(always)]
fn parse_hex1(b: u8) -> Option<u8> {
    let val = hex_val(b)?;
    Some((val << 4) | val)
}

#[inline(always)]
fn parse_rgb_value(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() || bytes.len() > 6 {
        return None;
    }

    // Negative numbers not allowed for RGB
    if bytes[0] == b'-' {
        return None;
    }

    let mut result = 0.0f32;
    let mut decimal_places = 0u32;

    for &b in bytes {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 4 { break; } // limit precision
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                    if result > 255.9 { return None; }
                }
            }
            b'.' => {
                if decimal_places > 0 { return None; }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    Some((result + 0.5) as u8) // round to nearest
}

#[inline(always)]
fn parse_percentage(bytes: &[u8]) -> Option<f32> {
    if bytes.is_empty() || bytes.len() > 6 || !bytes.ends_with(b"%") {
        return None;
    }

    let num_bytes = &bytes[..bytes.len() - 1];
    if num_bytes.is_empty() {
        return None;
    }

    let mut result = 0.0f32;
    let mut decimal_places = 0u32;
    let mut pos = 0;
    let negative = num_bytes[0] == b'-';

    if negative { 
        pos = 1;
        if num_bytes.len() == 1 { return None; }
    }

    for &b in &num_bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 3 { break; }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                }
            }
            b'.' => {
                if decimal_places > 0 { return None; }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    let final_result = if negative { -result } else { result };
    if final_result < 0.0 || final_result > 100.0 {
        return None;
    }
    
    Some(final_result)
}

#[inline(always)]
fn parse_alpha_value(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }

    if bytes.ends_with(b"%") {
        let percentage = parse_percentage(bytes)?;
        Some((percentage * 2.55 + 0.5) as u8)
    } else {
        // Alpha value 0.0-1.0
        let mut result = 0.0f32;
        let mut decimal_places = 0u32;
        let mut pos = 0;

        if bytes[0] == b'-' {
            return None; // negative alpha not allowed
        }

        for &b in &bytes[pos..] {
            match b {
                b'0'..=b'9' => {
                    if decimal_places > 0 {
                        decimal_places += 1;
                        if decimal_places > 4 { break; }
                        result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                    } else {
                        result = result * 10.0 + (b - b'0') as f32;
                    }
                }
                b'.' => {
                    if decimal_places > 0 { return None; }
                    decimal_places = 1;
                }
                _ => return None,
            }
        }

        if result < 0.0 || result > 1.0 {
            return None;
        }

        Some((result * 255.0 + 0.5) as u8)
    }
}

#[inline(always)]
fn parse_hue_value(s: &str) -> Option<f32> {
    let (h_val, unit) = if let Some(stripped) = s.strip_suffix("grad") {
        (parse_float_simple(stripped)?, "grad")
    } else if let Some(stripped) = s.strip_suffix("turn") {
        (parse_float_simple(stripped)?, "turn")
    } else if let Some(stripped) = s.strip_suffix("deg") {
        (parse_float_simple(stripped)?, "deg")
    } else if let Some(stripped) = s.strip_suffix("rad") {
        (parse_float_simple(stripped)?, "rad")
    } else {
        (parse_float_simple(s)?, "")
    };

    Some(normalize_angle(h_val, unit))
}

#[inline(always)]
fn parse_float_simple(s: &str) -> Option<f32> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > 8 {
        return None;
    }

    let mut result = 0.0f32;
    let mut decimal_places = 0u32;
    let mut pos = 0;
    let negative = bytes[0] == b'-';

    if negative { 
        pos = 1;
        if bytes.len() == 1 { return None; }
    }

    for &b in &bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 4 { break; }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                }
            }
            b'.' => {
                if decimal_places > 0 { return None; }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    Some(if negative { -result } else { result })
}

fn normalize_angle(value: f32, unit: &str) -> f32 {
    let degrees = match unit {
        "rad" => value * 57.295779513,
        "grad" => value * 0.9,
        "turn" => value * 360.0,
        _ => value,
    };
    ((degrees % 360.0) + 360.0) % 360.0
}

#[inline(always)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c * 0.5;

    let (r, g, b) = match (h / 60.0) as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0 + 0.5) as u8,
        ((g + m) * 255.0 + 0.5) as u8,
        ((b + m) * 255.0 + 0.5) as u8,
    )
}

#[inline]
fn split_params_fast(s: &str) -> [&str; 4] {
    let mut parts = [""; 4];
    let mut part_count = 0;
    let mut start = 0;
    let bytes = s.as_bytes();
    
    let mut i = 0;
    while i < bytes.len() && part_count < 4 {
        if bytes[i] == b',' || bytes[i] == b' ' {
            if i > start {
                let part = s[start..i].trim();
                if !part.is_empty() {
                    parts[part_count] = part;
                    part_count += 1;
                }
            }
            // Skip consecutive separators
            while i < bytes.len() && (bytes[i] == b',' || bytes[i] == b' ') {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    
    // Handle last part
    if start < s.len() && part_count < 4 {
        let part = s[start..].trim();
        if !part.is_empty() {
            parts[part_count] = part;
        }
    }
    
    parts
}

fn resolve_color_keyword(name: &str) -> Option<Color> {
    COLOR_KEYWORDS
        .binary_search_by_key(&name, |&(keyword, _)| keyword)
        .ok()
        .and_then(|idx| try_convert_color(COLOR_KEYWORDS[idx].1))
}

pub fn try_convert_color(color_str: &str) -> Option<Color> {
    let bytes = color_str.as_bytes();

    // Hex colors
    if bytes[0] == b'#' {
        let hex = &bytes[1..];
        return match hex.len() {
            3 => Some(Color {
                r: parse_hex1(hex[0])?,
                g: parse_hex1(hex[1])?,
                b: parse_hex1(hex[2])?,
                a: 255,
            }),
            4 => Some(Color {
                r: parse_hex1(hex[0])?,
                g: parse_hex1(hex[1])?,
                b: parse_hex1(hex[2])?,
                a: parse_hex1(hex[3])?,
            }),
            6 => Some(Color {
                r: parse_hex2(hex, 0)?,
                g: parse_hex2(hex, 2)?,
                b: parse_hex2(hex, 4)?,
                a: 255,
            }),
            8 => Some(Color {
                r: parse_hex2(hex, 0)?,
                g: parse_hex2(hex, 2)?,
                b: parse_hex2(hex, 4)?,
                a: parse_hex2(hex, 6)?,
            }),
            _ => None,
        };
    }

    // Function colors (rgb/rgba/hsl/hsla)
    if color_str.len() > 4 {
        let (func_name, has_alpha, content) =
            if color_str.starts_with("rgba(") && color_str.ends_with(')') {
                ("rgb", true, &color_str[5..color_str.len() - 1])
            } else if color_str.starts_with("rgb(") && color_str.ends_with(')') {
                ("rgb", false, &color_str[4..color_str.len() - 1])
            } else if color_str.starts_with("hsla(") && color_str.ends_with(')') {
                ("hsl", true, &color_str[5..color_str.len() - 1])
            } else if color_str.starts_with("hsl(") && color_str.ends_with(')') {
                ("hsl", false, &color_str[4..color_str.len() - 1])
            } else {
                return resolve_color_keyword(color_str);
            };

        let parts = split_params_fast(content);

        if func_name == "rgb" {
            if has_alpha {
                // rgba(color, alpha) format
                if parts[1] != "" && parts[2] == "" {
                    if let Some(base_color) = resolve_color_keyword(parts[0]) {
                        let a = parse_alpha_value(parts[1].as_bytes())?;
                        return Some(Color {
                            r: base_color.r,
                            g: base_color.g,
                            b: base_color.b,
                            a,
                        });
                    }
                }
                // rgba(r, g, b, alpha) format
                if parts[3] == "" { return None; }
                
                let r = parse_rgb_value(parts[0].as_bytes())?;
                let g = parse_rgb_value(parts[1].as_bytes())?;
                let b = parse_rgb_value(parts[2].as_bytes())?;
                let a = parse_alpha_value(parts[3].as_bytes())?;
                
                return Some(Color { r, g, b, a });
            } else {
                // rgb(r, g, b) format
                if parts[2] == "" || parts[3] != "" { return None; }
                
                let r = parse_rgb_value(parts[0].as_bytes())?;
                let g = parse_rgb_value(parts[1].as_bytes())?;
                let b = parse_rgb_value(parts[2].as_bytes())?;
                
                return Some(Color { r, g, b, a: 255 });
            }
        } else {
            // HSL
            if has_alpha {
                // hsla(color, alpha) format
                if parts[1] != "" && parts[2] == "" {
                    if let Some(base_color) = resolve_color_keyword(parts[0]) {
                        let a = parse_alpha_value(parts[1].as_bytes())?;
                        return Some(Color {
                            r: base_color.r,
                            g: base_color.g,
                            b: base_color.b,
                            a,
                        });
                    }
                }
                // hsla(h, s, l, alpha) format
                if parts[3] == "" { return None; }
                
                let h = parse_hue_value(parts[0])?;
                let s = parse_percentage(parts[1].as_bytes())?;
                let l = parse_percentage(parts[2].as_bytes())?;
                let (r, g, b) = hsl_to_rgb(h, s, l);
                let a = parse_alpha_value(parts[3].as_bytes())?;
                
                return Some(Color { r, g, b, a });
            } else {
                // hsl(h, s, l) format
                if parts[2] == "" || parts[3] != "" { return None; }
                
                let h = parse_hue_value(parts[0])?;
                let s = parse_percentage(parts[1].as_bytes())?;
                let l = parse_percentage(parts[2].as_bytes())?;
                let (r, g, b) = hsl_to_rgb(h, s, l);
                
                return Some(Color { r, g, b, a: 255 });
            }
        }
    }

    // Color keywords
    resolve_color_keyword(color_str)
}

pub fn is_valid_color(color_str: &str) -> bool {
    try_convert_color(color_str).is_some()
}

// Legacy compatibility
pub fn convert_to_color(color_str: &str) -> Option<Color> {
    try_convert_color(color_str)
}

// ========== SHORTENER FUNCTIONS ==========

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
        if let Some(color) = try_convert_color(hex_val) {
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