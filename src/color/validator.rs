use super::keywords::COLOR_KEYWORDS;

fn is_valid_percentage(part: &str) -> bool {
    if let Some(val_str) = part.strip_suffix('%') {
        if let Ok(val) = val_str.parse::<f32>() {
            return val >= 0.0 && val <= 100.0;
        }
    }
    false
}

fn is_valid_hue(part: &str) -> bool {
    let mut temp = part;
    if temp.ends_with("deg") {
        temp = temp.strip_suffix("deg").unwrap();
    } else if temp.ends_with("rad") {
        temp = temp.strip_suffix("rad").unwrap();
    } else if temp.ends_with("grad") {
        temp = temp.strip_suffix("grad").unwrap();
    } else if temp.ends_with("turn") {
        temp = temp.strip_suffix("turn").unwrap();
    }
    temp.parse::<f32>().is_ok()
}

fn is_valid_rgb_value(part: &str) -> bool {
    part.parse::<u8>().is_ok()
}

fn is_valid_alpha_value(part: &str) -> bool {
    if let Ok(val) = part.parse::<f32>() {
        return val >= 0.0 && val <= 1.0;
    }
    is_valid_percentage(part)
}

pub fn is_valid_color(color_str: &str) -> bool {
    let trimmed = color_str.trim().to_lowercase();
    if trimmed.is_empty() {
        return false;
    }
    
    // Hex colors (#rgb, #rrggbb, #rgba, #rrggbbaa)
    if let Some(hex) = trimmed.strip_prefix('#') {
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
        return matches!(hex.len(), 3 | 4 | 6 | 8);
    }
    
    // RGB/RGBA colors
    if let Some(body) = trimmed.strip_prefix("rgb").and_then(|s| s.strip_suffix(')')) {
        let (body, has_alpha) = if let Some(body_a) = body.strip_prefix('a') {
            (body_a, true)
        } else {
            (body, false)
        };
        
        let Some(inner) = body.strip_prefix('(') else { return false };
        let parts: Vec<&str> = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
            
        if has_alpha {
            return parts.len() == 4
                && is_valid_rgb_value(parts[0])
                && is_valid_rgb_value(parts[1])
                && is_valid_rgb_value(parts[2])
                && is_valid_alpha_value(parts[3]);
        } else {
            return parts.len() == 3
                && is_valid_rgb_value(parts[0])
                && is_valid_rgb_value(parts[1])
                && is_valid_rgb_value(parts[2]);
        }
    }
    
    // HSL/HSLA colors
    if let Some(body) = trimmed.strip_prefix("hsl").and_then(|s| s.strip_suffix(')')) {
        let (body, has_alpha) = if let Some(body_a) = body.strip_prefix('a') {
            (body_a, true)
        } else {
            (body, false)
        };
        
        let Some(inner) = body.strip_prefix('(') else { return false };
        let parts: Vec<&str> = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
            
        if has_alpha {
            return parts.len() == 4
                && is_valid_hue(parts[0])
                && is_valid_percentage(parts[1])
                && is_valid_percentage(parts[2])
                && is_valid_alpha_value(parts[3]);
        } else {
            return parts.len() == 3
                && is_valid_hue(parts[0])
                && is_valid_percentage(parts[1])
                && is_valid_percentage(parts[2]);
        }
    }
    
    // Named colors
    for &(name, _) in COLOR_KEYWORDS {
        if trimmed == name {
            return true;
        }
    }
    
    false
}