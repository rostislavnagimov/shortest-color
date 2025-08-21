use super::keywords::COLOR_KEYWORDS;
use super::model::Color;

const HEX_VALUES: [u8; 256] = {
    let mut values = [255; 256];
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
    HEX_VALUES[c as usize]
}

#[inline(always)]
fn parse_hex_pair(bytes: &[u8], pos: usize) -> Option<u8> {
    if pos + 1 >= bytes.len() {
        return None;
    }
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
        Some(val * 17)
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
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    )
}

#[inline]
fn parse_numeric(s: &str) -> Option<f32> {
    if s.is_empty() || s.len() > 10 {
        return None;
    }

    let bytes = s.as_bytes();
    let mut result = 0.0f32;
    let mut decimal_pos = None;
    let mut pos = 0;
    let negative = bytes[0] == b'-';

    if negative {
        pos = 1;
        if s.len() == 1 {
            return None;
        }
    }

    for &b in &bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                let digit = (b - b'0') as f32;
                if let Some(dp) = decimal_pos {
                    let power = 10.0f32.powi((pos - dp) as i32);
                    result += digit / power;
                } else {
                    result = result * 10.0 + digit;
                }
            }
            b'.' => {
                if decimal_pos.is_some() {
                    return None;
                }
                decimal_pos = Some(pos);
            }
            _ => return None,
        }
        pos += 1;
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

pub fn convert_to_color(color_str: &str) -> Option<Color> {
    let trimmed = color_str.trim();
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();

    if bytes[0] == b'#' {
        let hex_bytes = &bytes[1..];
        let len = hex_bytes.len();

        return match len {
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
            _ => None,
        };
    }

    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("rgb") {
        let rgb_body = &lower[3..];
        let (body, has_alpha) = if rgb_body.starts_with('a') {
            (&rgb_body[1..], true)
        } else {
            (rgb_body, false)
        };

        if !body.starts_with('(') || !body.ends_with(')') {
            return None;
        }

        let inner = &body[1..body.len() - 1];
        let parts = split_function_values(inner);

        if has_alpha && parts.len() == 4 {
            let r = parse_numeric(parts[0])? as u8;
            let g = parse_numeric(parts[1])? as u8;
            let b = parse_numeric(parts[2])? as u8;
            let a_str = parts[3];
            let a_f32 = parse_numeric(a_str.trim_end_matches('%'))?;
            let a = if a_str.contains('%') {
                (a_f32 * 2.55).round() as u8
            } else {
                (a_f32 * 255.0).round() as u8
            };
            return Some(Color { r, g, b, a });
        } else if !has_alpha && parts.len() == 3 {
            let r = parse_numeric(parts[0])? as u8;
            let g = parse_numeric(parts[1])? as u8;
            let b = parse_numeric(parts[2])? as u8;
            return Some(Color { r, g, b, a: 255 });
        }
    }

    if lower.starts_with("hsl") {
        let hsl_body = &lower[3..];
        let (body, has_alpha) = if hsl_body.starts_with('a') {
            (&hsl_body[1..], true)
        } else {
            (hsl_body, false)
        };

        if !body.starts_with('(') || !body.ends_with(')') {
            return None;
        }

        let inner = &body[1..body.len() - 1];
        let parts = split_function_values(inner);

        if has_alpha && parts.len() == 4 {
            let h_str =
                parts[0].trim_end_matches(|c: char| !c.is_numeric() && c != '.' && c != '-');
            let h = parse_numeric(h_str)?;
            let s = parse_numeric(parts[1].trim_end_matches('%'))?;
            let l = parse_numeric(parts[2].trim_end_matches('%'))?;
            let (r, g, b) = hsl_to_rgb(h, s, l);
            let a_str = parts[3];
            let a_f32 = parse_numeric(a_str.trim_end_matches('%'))?;
            let a = if a_str.contains('%') {
                (a_f32 * 2.55).round() as u8
            } else {
                (a_f32 * 255.0).round() as u8
            };
            return Some(Color { r, g, b, a });
        } else if !has_alpha && parts.len() == 3 {
            let h_str =
                parts[0].trim_end_matches(|c: char| !c.is_numeric() && c != '.' && c != '-');
            let h = parse_numeric(h_str)?;
            let s = parse_numeric(parts[1].trim_end_matches('%'))?;
            let l = parse_numeric(parts[2].trim_end_matches('%'))?;
            let (r, g, b) = hsl_to_rgb(h, s, l);
            return Some(Color { r, g, b, a: 255 });
        }
    }

    COLOR_KEYWORDS
        .binary_search_by_key(&&*lower, |&(name, _)| name)
        .ok()
        .and_then(|idx| convert_to_color(COLOR_KEYWORDS[idx].1))
}

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
