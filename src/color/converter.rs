use super::keywords::COLOR_KEYWORDS;
use super::model::Color;

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

fn parse_number(s: &str) -> Option<f32> {
    let bytes = s.as_bytes();
    if bytes.is_empty() { return None; }

    let mut result = 0.0f32;
    let mut decimal_places = 0u32;
    let mut pos = 0;
    let negative = bytes[0] == b'-';

    if negative { pos = 1; }

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

#[inline]
fn split_params(s: &str) -> Vec<&str> {
    s.split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect()
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

fn resolve_color_keyword(name: &str) -> Option<Color> {
    COLOR_KEYWORDS
        .binary_search_by_key(&name, |&(keyword, _)| keyword)
        .ok()
        .and_then(|idx| convert_to_color(COLOR_KEYWORDS[idx].1))
}

pub fn convert_to_color(color_str: &str) -> Option<Color> {
    let bytes = color_str.as_bytes();

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

        let parts = split_params(content);

        if func_name == "rgb" {
            if has_alpha {
                if parts.len() == 2 {
                    if let Some(base_color) = resolve_color_keyword(parts[0]) {
                        let alpha_str = parts[1];
                        let alpha_val = parse_number(alpha_str.trim_end_matches('%'))?;
                        let a = if alpha_str.ends_with('%') {
                            (alpha_val * 2.55 + 0.5) as u8
                        } else {
                            (alpha_val * 255.0 + 0.5) as u8
                        };
                        return Some(Color {
                            r: base_color.r,
                            g: base_color.g,
                            b: base_color.b,
                            a,
                        });
                    }
                } else if parts.len() == 4 {
                    let r = parse_number(parts[0])?.clamp(0.0, 255.0).round() as u8;
                    let g = parse_number(parts[1])?.clamp(0.0, 255.0).round() as u8;
                    let b = parse_number(parts[2])?.clamp(0.0, 255.0).round() as u8;
                    let alpha_str = parts[3];
                    let alpha_val = parse_number(alpha_str.trim_end_matches('%'))?;
                    let a = if alpha_str.ends_with('%') {
                        (alpha_val * 2.55 + 0.5) as u8
                    } else {
                        (alpha_val * 255.0 + 0.5) as u8
                    };
                    return Some(Color { r, g, b, a });
                }
                return None;
            } else {
                if parts.len() != 3 { return None; }
                let r = parse_number(parts[0])?.clamp(0.0, 255.0).round() as u8;
                let g = parse_number(parts[1])?.clamp(0.0, 255.0).round() as u8;
                let b = parse_number(parts[2])?.clamp(0.0, 255.0).round() as u8;
                return Some(Color { r, g, b, a: 255 });
            }
        } else {
            if has_alpha {
                if parts.len() == 2 {
                    if let Some(base_color) = resolve_color_keyword(parts[0]) {
                        let alpha_str = parts[1];
                        let alpha_val = parse_number(alpha_str.trim_end_matches('%'))?;
                        let a = if alpha_str.ends_with('%') {
                            (alpha_val * 2.55 + 0.5) as u8
                        } else {
                            (alpha_val * 255.0 + 0.5) as u8
                        };
                        return Some(Color {
                            r: base_color.r,
                            g: base_color.g,
                            b: base_color.b,
                            a,
                        });
                    }
                } else if parts.len() == 4 {
                    let h_str = parts[0];
                    let (h_val, unit) = if let Some(stripped) = h_str.strip_suffix("grad") {
                        (parse_number(stripped)?, "grad")
                    } else if let Some(stripped) = h_str.strip_suffix("turn") {
                        (parse_number(stripped)?, "turn")
                    } else if let Some(stripped) = h_str.strip_suffix("deg") {
                        (parse_number(stripped)?, "deg")
                    } else if let Some(stripped) = h_str.strip_suffix("rad") {
                        (parse_number(stripped)?, "rad")
                    } else {
                        (parse_number(h_str)?, "")
                    };

                    let h = normalize_angle(h_val, unit);
                    let s = parse_number(parts[1].trim_end_matches('%'))?;
                    let l = parse_number(parts[2].trim_end_matches('%'))?;
                    let (r, g, b) = hsl_to_rgb(h, s, l);
                    let alpha_str = parts[3];
                    let alpha_val = parse_number(alpha_str.trim_end_matches('%'))?;
                    let a = if alpha_str.ends_with('%') {
                        (alpha_val * 2.55 + 0.5) as u8
                    } else {
                        (alpha_val * 255.0 + 0.5) as u8
                    };
                    return Some(Color { r, g, b, a });
                }
                return None;
            } else {
                if parts.len() != 3 { return None; }
                let h_str = parts[0];
                let (h_val, unit) = if let Some(stripped) = h_str.strip_suffix("grad") {
                    (parse_number(stripped)?, "grad")
                } else if let Some(stripped) = h_str.strip_suffix("turn") {
                    (parse_number(stripped)?, "turn")
                } else if let Some(stripped) = h_str.strip_suffix("deg") {
                    (parse_number(stripped)?, "deg")
                } else if let Some(stripped) = h_str.strip_suffix("rad") {
                    (parse_number(stripped)?, "rad")
                } else {
                    (parse_number(h_str)?, "")
                };

                let h = normalize_angle(h_val, unit);
                let s = parse_number(parts[1].trim_end_matches('%'))?;
                let l = parse_number(parts[2].trim_end_matches('%'))?;
                let (r, g, b) = hsl_to_rgb(h, s, l);
                return Some(Color { r, g, b, a: 255 });
            }
        }
    }

    resolve_color_keyword(color_str)
}