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
    lookup
};

#[inline(always)]
fn is_hex_char(c: u8) -> bool {
    HEX_LOOKUP[c as usize]
}

#[inline]
fn parse_f32_optimized(s: &str) -> Option<f32> {
    if s.is_empty() || s.len() > 10 {
        return None;
    }

    let bytes = s.as_bytes();
    let mut result = 0.0f32;
    let mut decimal_divisor = 0.0f32;
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
                if decimal_divisor > 0.0 {
                    decimal_divisor *= 10.0;
                    result += digit / decimal_divisor;
                } else {
                    result = result * 10.0 + digit;
                }
            }
            b'.' => {
                if decimal_divisor > 0.0 {
                    return None;
                }
                decimal_divisor = 1.0;
            }
            _ => return None,
        }
    }

    Some(if negative { -result } else { result })
}

#[inline(always)]
fn validate_rgb_parts(inner: &str, has_alpha: bool) -> bool {
    let parts: Vec<&str> = inner
        .split(|c| c == ',' || c == ' ')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim())
        .collect();

    let expected = if has_alpha { 4 } else { 3 };
    if parts.len() != expected {
        return false;
    }

    for (i, &part) in parts.iter().enumerate() {
        if i < 3 {
            if let Some(val) = parse_f32_optimized(part) {
                if val < 0.0 || val > if part.contains('.') { 255.9 } else { 255.0 } {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            if part.contains('%') {
                if let Some(val_str) = part.strip_suffix('%') {
                    if let Some(val) = parse_f32_optimized(val_str) {
                        if val < 0.0 || val > 100.0 {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                if let Some(val) = parse_f32_optimized(part) {
                    if val < 0.0 || val > 1.0 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
    true
}

#[inline(always)]
fn validate_hsl_parts(inner: &str, has_alpha: bool) -> bool {
    let parts: Vec<&str> = inner
        .split(|c| c == ',' || c == ' ')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim())
        .collect();

    let expected = if has_alpha { 4 } else { 3 };
    if parts.len() != expected {
        return false;
    }

    for (i, &part) in parts.iter().enumerate() {
        match i {
            0 => {
                let numeric_part = if let Some(stripped) = part.strip_suffix("grad") {
                    stripped
                } else if let Some(stripped) = part.strip_suffix("turn") {
                    stripped
                } else if let Some(stripped) = part.strip_suffix("deg") {
                    stripped
                } else if let Some(stripped) = part.strip_suffix("rad") {
                    stripped
                } else {
                    part
                };
                if numeric_part.is_empty() || parse_f32_optimized(numeric_part).is_none() {
                    return false;
                }
            }
            1 | 2 => {
                if let Some(val_str) = part.strip_suffix('%') {
                    if let Some(val) = parse_f32_optimized(val_str) {
                        if val < 0.0 || val > 100.0 {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            3 => {
                if part.contains('%') {
                    if let Some(val_str) = part.strip_suffix('%') {
                        if let Some(val) = parse_f32_optimized(val_str) {
                            if val < 0.0 || val > 100.0 {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    if let Some(val) = parse_f32_optimized(part) {
                        if val < 0.0 || val > 1.0 {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            _ => return false,
        }
    }
    true
}

pub fn is_valid_color(trimmed_lower: &str) -> bool {
    if trimmed_lower.is_empty() {
        return false;
    }

    let bytes = trimmed_lower.as_bytes();

    if bytes[0] == b'#' {
        let hex_part = &bytes[1..];
        let len = hex_part.len();
        return matches!(len, 6 | 8) && hex_part.iter().all(|&b| is_hex_char(b));
    }

    if let Some(rgb_body) = trimmed_lower.strip_prefix("rgb") {
        let (function_body, has_alpha) = if let Some(body) = rgb_body.strip_prefix('a') {
            (body, true)
        } else {
            (rgb_body, false)
        };

        if function_body.len() < 2 || !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        return validate_rgb_parts(inner, has_alpha);
    }

    if let Some(hsl_body) = trimmed_lower.strip_prefix("hsl") {
        let (function_body, has_alpha) = if let Some(body) = hsl_body.strip_prefix('a') {
            (body, true)
        } else {
            (hsl_body, false)
        };

        if function_body.len() < 2 || !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        return validate_hsl_parts(inner, has_alpha);
    }

    COLOR_KEYWORDS
        .binary_search_by_key(&trimmed_lower, |&(keyword, _)| keyword)
        .is_ok()
}