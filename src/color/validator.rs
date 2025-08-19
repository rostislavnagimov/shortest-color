#[inline]
fn is_color_keyword(name: &str) -> bool {
    COLOR_KEYWORDS
        .binary_search_by_key(&name, |&(keyword, _)| keyword)
        .is_ok()
}
use super::keywords::COLOR_KEYWORDS;

#[inline]
fn is_valid_percentage(part: &str) -> bool {
    let Some(val_str) = part.strip_suffix('%') else {
        return false;
    };

    match val_str {
        "0" => true,
        "50" => true,
        "100" => true,
        _ => {
            if !val_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return false;
            }
            val_str
                .parse::<f32>()
                .map(|val| val >= 0.0 && val <= 100.0)
                .unwrap_or(false)
        }
    }
}

#[inline]
fn is_valid_hue(part: &str) -> bool {
    let numeric_part = match part {
        s if s.ends_with("deg") => &s[..s.len() - 3],
        s if s.ends_with("rad") => &s[..s.len() - 3],
        s if s.ends_with("grad") => &s[..s.len() - 4],
        s if s.ends_with("turn") => &s[..s.len() - 4],
        s => s,
    };

    if numeric_part.is_empty()
        || !numeric_part
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    {
        return false;
    }

    numeric_part.parse::<f32>().is_ok()
}

#[inline]
fn is_valid_rgb_value(part: &str) -> bool {
    if part.len() > 3 || part.is_empty() {
        return false;
    }

    if !part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    part.parse::<u8>().is_ok()
}

#[inline]
fn is_valid_alpha_value(part: &str) -> bool {
    if part.contains('%') {
        return is_valid_percentage(part);
    }

    match part {
        "0" | "0.0" => true,
        "1" | "1.0" => true,
        "0.5" => true,
        _ => {
            if !part.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return false;
            }
            part.parse::<f32>()
                .map(|val| val >= 0.0 && val <= 1.0)
                .unwrap_or(false)
        }
    }
}

#[inline]
fn parse_function_parts(inner: &str) -> Option<[&str; 4]> {
    let mut parts = [""; 4];
    let mut count = 0;

    for part in inner.split(|c| c == ',' || c == ' ') {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            if count >= 4 {
                return None;
            }
            parts[count] = trimmed;
            count += 1;
        }
    }

    Some(parts)
}

pub fn is_valid_color(color_str: &str) -> bool {
    let trimmed = color_str.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let Some(hex_part) = trimmed.strip_prefix('#') {
        return hex_part.len() == 3
            || hex_part.len() == 4
            || hex_part.len() == 6
            || hex_part.len() == 8 && hex_part.bytes().all(|b| b.is_ascii_hexdigit());
    }

    let lower = trimmed.to_lowercase();

    if let Some(rgb_body) = lower.strip_prefix("rgb") {
        let (function_body, has_alpha) = if let Some(rgba_body) = rgb_body.strip_prefix('a') {
            (rgba_body, true)
        } else {
            (rgb_body, false)
        };

        if !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        let Some(parts) = parse_function_parts(inner) else {
            return false;
        };

        let expected_count = if has_alpha { 4 } else { 3 };
        let actual_count = parts.iter().take_while(|&&s| !s.is_empty()).count();

        if actual_count != expected_count {
            return false;
        }

        if !is_valid_rgb_value(parts[0])
            || !is_valid_rgb_value(parts[1])
            || !is_valid_rgb_value(parts[2])
        {
            return false;
        }

        if has_alpha && !is_valid_alpha_value(parts[3]) {
            return false;
        }

        return true;
    }

    if let Some(hsl_body) = lower.strip_prefix("hsl") {
        let (function_body, has_alpha) = if let Some(hsla_body) = hsl_body.strip_prefix('a') {
            (hsla_body, true)
        } else {
            (hsl_body, false)
        };

        if !function_body.starts_with('(') || !function_body.ends_with(')') {
            return false;
        }

        let inner = &function_body[1..function_body.len() - 1];
        let Some(parts) = parse_function_parts(inner) else {
            return false;
        };

        let expected_count = if has_alpha { 4 } else { 3 };
        let actual_count = parts.iter().take_while(|&&s| !s.is_empty()).count();

        if actual_count != expected_count {
            return false;
        }

        if !is_valid_hue(parts[0])
            || !is_valid_percentage(parts[1])
            || !is_valid_percentage(parts[2])
        {
            return false;
        }

        if has_alpha && !is_valid_alpha_value(parts[3]) {
            return false;
        }

        return true;
    }

    is_color_keyword(&lower)
}
