use super::keywords::COLOR_KEYWORDS;
use super::model::Color;

const HEX_VALUES: [u8; 256] = {
    let mut values = [16; 256];
    let mut i = 0;
    while i < 10 {
        values[(b'0' + i) as usize] = i;
        i += 1;
    }
    let mut i = 0;
    while i < 6 {
        values[(b'a' + i) as usize] = 10 + i;
        values[(b'A' + i) as usize] = 10 + i;
        i += 1;
    }
    values
};

#[inline(always)]
fn hex_char_to_value(c: u8) -> u8 {
    unsafe { *HEX_VALUES.get_unchecked(c as usize) }
}

#[inline(always)]
fn parse_hex_pair(bytes: &[u8], pos: usize) -> Option<u8> {
    let high = hex_char_to_value(bytes[pos]);
    let low = hex_char_to_value(bytes[pos + 1]);
    if high > 15 || low > 15 {
        None
    } else {
        Some(high << 4 | low)
    }
}

#[inline(always)]
fn parse_hex_single(byte: u8) -> Option<u8> {
    let val = hex_char_to_value(byte);
    if val > 15 {
        None
    } else {
        Some(val << 4 | val)
    }
}

#[inline(always)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c * 0.5;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r_prime + m) * 255.0).round() as u8,
        ((g_prime + m) * 255.0).round() as u8,
        ((b_prime + m) * 255.0).round() as u8,
    )
}

#[inline]
fn parse_numeric_fast(s: &str) -> Option<f32> {
    if s.is_empty() {
        return None;
    }

    let bytes = s.as_bytes();
    let mut result = 0.0f32;
    let mut decimal_divisor = 1.0f32;
    let mut pos = 0;
    let negative = bytes[0] == b'-';

    if negative {
        pos = 1;
        if bytes.len() == 1 {
            return None;
        }
    }

    let mut found_decimal = false;

    for &b in &bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                let digit = (b - b'0') as f32;
                if found_decimal {
                    decimal_divisor *= 10.0;
                    result += digit / decimal_divisor;
                } else {
                    result = result * 10.0 + digit;
                }
            }
            b'.' => {
                if found_decimal {
                    return None;
                }
                found_decimal = true;
            }
            _ => return None,
        }
    }

    Some(if negative { -result } else { result })
}

#[inline]
fn split_function_values(inner: &str) -> Vec<&str> {
    inner
        .split(|c| c == ',' || c == ' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

#[inline]
fn normalize_hue(h: f32, unit: &str) -> f32 {
    let normalized = match unit {
        "deg" | "" => h,
        "rad" => h * 180.0 / std::f32::consts::PI,
        "grad" => h * 0.9,
        "turn" => h * 360.0,
        _ => h,
    };

    let mut result = normalized % 360.0;
    if result < 0.0 {
        result += 360.0;
    }
    result
}

pub fn convert_to_color(color_str: &str) -> Option<Color> {
    let bytes = color_str.as_bytes();

    if bytes[0] == b'#' {
        let hex_len = bytes.len() - 1;
        let hex_bytes = &bytes[1..];

        return match hex_len {
            6 => Some(Color {
                r: parse_hex_pair(hex_bytes, 0)?,
                g: parse_hex_pair(hex_bytes, 2)?,
                b: parse_hex_pair(hex_bytes, 4)?,
                a: 255,
            }),
            8 => Some(Color {
                r: parse_hex_pair(hex_bytes, 0)?,
                g: parse_hex_pair(hex_bytes, 2)?,
                b: parse_hex_pair(hex_bytes, 4)?,
                a: parse_hex_pair(hex_bytes, 6)?,
            }),
            3 => Some(Color {
                r: parse_hex_single(hex_bytes[0])?,
                g: parse_hex_single(hex_bytes[1])?,
                b: parse_hex_single(hex_bytes[2])?,
                a: 255,
            }),
            4 => Some(Color {
                r: parse_hex_single(hex_bytes[0])?,
                g: parse_hex_single(hex_bytes[1])?,
                b: parse_hex_single(hex_bytes[2])?,
                a: parse_hex_single(hex_bytes[3])?,
            }),
            _ => None,
        };
    }

    if bytes.len() > 4 && bytes[0] == b'r' && bytes[1] == b'g' && bytes[2] == b'b' {
        let has_alpha = bytes[3] == b'a';
        let start_idx = if has_alpha { 5 } else { 4 };

        let end_idx = color_str.len() - 1;
        let inner = &color_str[start_idx..end_idx];
        let parts = split_function_values(inner);

        if has_alpha && parts.len() == 4 {
            let r = parse_numeric_fast(parts[0])?.round().clamp(0.0, 255.0) as u8;
            let g = parse_numeric_fast(parts[1])?.round().clamp(0.0, 255.0) as u8;
            let b = parse_numeric_fast(parts[2])?.round().clamp(0.0, 255.0) as u8;
            let a_str = parts[3];
            let a_val = parse_numeric_fast(a_str.trim_end_matches('%'))?;
            let a = if a_str.contains('%') {
                (a_val * 2.55).round() as u8
            } else {
                (a_val * 255.0).round() as u8
            };
            return Some(Color { r, g, b, a });
        } else if !has_alpha && parts.len() == 3 {
            let r = parse_numeric_fast(parts[0])?.round().clamp(0.0, 255.0) as u8;
            let g = parse_numeric_fast(parts[1])?.round().clamp(0.0, 255.0) as u8;
            let b = parse_numeric_fast(parts[2])?.round().clamp(0.0, 255.0) as u8;
            return Some(Color { r, g, b, a: 255 });
        }
    }

    if bytes.len() > 4 && bytes[0] == b'h' && bytes[1] == b's' && bytes[2] == b'l' {
        let has_alpha = bytes[3] == b'a';
        let start_idx = if has_alpha { 5 } else { 4 };

        let end_idx = color_str.len() - 1;
        let inner = &color_str[start_idx..end_idx];
        let parts = split_function_values(inner);

        if parts.len() >= 3 {
            let h_str = parts[0];
            let (h_val, unit) = if h_str.ends_with("grad") {
                (&h_str[..h_str.len() - 4], "grad")
            } else if h_str.ends_with("turn") {
                (&h_str[..h_str.len() - 4], "turn")
            } else if h_str.ends_with("deg") {
                (&h_str[..h_str.len() - 3], "deg")
            } else if h_str.ends_with("rad") {
                (&h_str[..h_str.len() - 3], "rad")
            } else {
                (h_str, "")
            };

            let h_raw = parse_numeric_fast(h_val)?;
            let h = normalize_hue(h_raw, unit);
            let s = parse_numeric_fast(parts[1].trim_end_matches('%'))?;
            let l = parse_numeric_fast(parts[2].trim_end_matches('%'))?;
            let (r, g, b) = hsl_to_rgb(h, s, l);

            if has_alpha && parts.len() == 4 {
                let a_str = parts[3];
                let a_val = parse_numeric_fast(a_str.trim_end_matches('%'))?;
                let a = if a_str.contains('%') {
                    (a_val * 2.55).round() as u8
                } else {
                    (a_val * 255.0).round() as u8
                };
                return Some(Color { r, g, b, a });
            } else if !has_alpha {
                return Some(Color { r, g, b, a: 255 });
            }
        }
    }

    COLOR_KEYWORDS
        .binary_search_by_key(&color_str, |&(name, _)| name)
        .ok()
        .and_then(|idx| convert_to_color(COLOR_KEYWORDS[idx].1))
}

#[inline]
pub fn color_to_hex(color: &Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )
    }
}
