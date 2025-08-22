use super::keywords::COLOR_KEYWORDS;

const HEX_LOOKUP: [bool; 256] = {
    let mut lookup = [false; 256];
    let mut i = b'0';
    while i <= b'9' {
        lookup[i as usize] = true;
        i += 1;
    }
    let mut i = b'a';
    while i <= b'f' {
        lookup[i as usize] = true;
        i += 1;
    }
    let mut i = b'A';
    while i <= b'F' {
        lookup[i as usize] = true;
        i += 1;
    }
    lookup
};

#[inline(always)]
fn is_hex_char(c: u8) -> bool {
    HEX_LOOKUP[c as usize]
}

#[inline(always)]
fn is_color_keyword(name: &str) -> bool {
    COLOR_KEYWORDS
        .binary_search_by_key(&name, |&(keyword, _)| keyword)
        .is_ok()
}

#[inline]
fn parse_f32_fast(s: &str) -> Option<f32> {
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

#[inline(always)]
fn is_valid_percentage(part: &str) -> bool {
    match part.strip_suffix('%') {
        Some(val_str) if !val_str.is_empty() => parse_f32_fast(val_str)
            .map(|val| val >= 0.0 && val <= 100.0)
            .unwrap_or(false),
        _ => false,
    }
}

#[inline(always)]
fn is_valid_hue(part: &str) -> bool {
    if part.is_empty() {
        return false;
    }
    
    let numeric_part = if part.ends_with("grad") {
        &part[..part.len() - 4]
    } else if part.ends_with("turn") {
        &part[..part.len() - 4]
    } else if part.ends_with("deg") {
        &part[..part.len() - 3]
    } else if part.ends_with("rad") {
        &part[..part.len() - 3]
    } else {
        part
    };

    // Важно: для hue любое числовое значение валидно (включая 400, 720, -90, etc.)
    // так как углы нормализуются по модулю 360
    !numeric_part.is_empty() && parse_f32_fast(numeric_part).is_some()
}

#[inline(always)]
fn is_valid_rgb_value(part: &str) -> bool {
    if let Some(val) = parse_f32_fast(part) {
        if val < 0.0 {
            return false;
        }
        
        if part.contains('.') {
            val <= 255.9
        } else {
            val <= 255.0
        }
    } else {
        false
    }
}

#[inline(always)]
fn is_valid_alpha_value(part: &str) -> bool {
    if part.contains('%') {
        is_valid_percentage(part)
    } else {
        parse_f32_fast(part)
            .map(|val| val >= 0.0 && val <= 1.0)
            .unwrap_or(false)
    }
}

pub fn is_valid_color(color_str: &str) -> bool {
    let trimmed = color_str.trim();
    if trimmed.is_empty() {
        return false;
    }

    let bytes = trimmed.as_bytes();

    if bytes[0] == b'#' {
        let hex_part = &bytes[1..];
        let len = hex_part.len();

        if len != 3 && len != 4 && len != 6 && len != 8 {
            return false;
        }

        return hex_part.iter().all(|&b| is_hex_char(b));
    }

    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("rgb") {
        let rgb_body = &lower[3..];
        let (function_body, expected_count) = if rgb_body.starts_with('a') {
            (&rgb_body[1..], 4)
        } else {
            (rgb_body, 3)
        };

        if !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        let parts = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim());

        let mut count = 0;

        for part in parts {
            count += 1;
            if count > expected_count {
                return false;
            }

            if count <= 3 {
                if !is_valid_rgb_value(part) {
                    return false;
                }
            } else if count == 4 {
                if !is_valid_alpha_value(part) {
                    return false;
                }
            }
        }

        return count == expected_count;
    }

    if lower.starts_with("hsl") {
        let hsl_body = &lower[3..];
        let (function_body, expected_count) = if hsl_body.starts_with('a') {
            (&hsl_body[1..], 4)
        } else {
            (hsl_body, 3)
        };

        if !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        let mut parts = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim());

        if let Some(h) = parts.next() {
            if !is_valid_hue(h) {
                return false;
            }

            if let Some(s) = parts.next() {
                if !is_valid_percentage(s) {
                    return false;
                }

                if let Some(l) = parts.next() {
                    if !is_valid_percentage(l) {
                        return false;
                    }

                    if expected_count == 4 {
                        if let Some(a) = parts.next() {
                            return is_valid_alpha_value(a) && parts.next().is_none();
                        }
                        return false;
                    } else {
                        return parts.next().is_none();
                    }
                }
            }
        }

        return false;
    }

    is_color_keyword(&lower)
}