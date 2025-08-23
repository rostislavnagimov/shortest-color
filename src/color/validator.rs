use super::keywords::COLOR_KEYWORDS;

const HEX_LOOKUP: u128 = {
    let mut mask = 0u128;
    let mut i = b'0';
    while i <= b'9' {
        mask |= 1u128 << i;
        i += 1;
    }
    let mut i = b'a';
    while i <= b'f' {
        mask |= 1u128 << i;
        i += 1;
    }
    mask
};

#[inline(always)]
fn is_hex_char(c: u8) -> bool {
    c < 128 && (HEX_LOOKUP & (1u128 << c)) != 0
}

#[inline(always)]
fn parse_u16_fast(bytes: &[u8]) -> Option<u16> {
    if bytes.is_empty() || bytes.len() > 3 {
        return None;
    }
    
    let mut result = 0u16;
    for &b in bytes {
        match b {
            b'0'..=b'9' => {
                let digit = (b - b'0') as u16;
                result = result.wrapping_mul(10).wrapping_add(digit);
                if result > 255 {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(result)
}

#[inline(always)]
fn parse_f32_fast(bytes: &[u8]) -> Option<f32> {
    if bytes.is_empty() || bytes.len() > 10 {
        return None;
    }

    let mut result = 0.0f32;
    let mut decimal_divisor = 0.0f32;
    let mut pos = 0;
    let negative = bytes[0] == b'-';
    
    if negative {
        pos = 1;
        if bytes.len() == 1 { return None; }
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
                if decimal_divisor > 0.0 { return None; }
                decimal_divisor = 1.0;
            }
            _ => return None,
        }
    }

    Some(if negative { -result } else { result })
}

#[inline(always)]
fn is_color_keyword(name: &str) -> bool {
    COLOR_KEYWORDS.binary_search_by_key(&name, |&(keyword, _)| keyword).is_ok()
}

#[inline(always)]
fn parse_parts_no_alloc(s: &str) -> [&str; 4] {
    let mut parts = [""; 4];
    let mut part_count = 0;
    let mut start = 0;
    let bytes = s.as_bytes();
    
    let mut i = 0;
    while i < bytes.len() && part_count < 4 {
        if bytes[i] == b',' || bytes[i] == b' ' {
            if i > start {
                let part = &s[start..i];
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    parts[part_count] = trimmed;
                    part_count += 1;
                }
            }
            i += 1;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b',') {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    
    if start < s.len() && part_count < 4 {
        let part = &s[start..];
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            parts[part_count] = trimmed;
        }
    }
    
    parts
}

#[inline(always)]
fn validate_alpha_part(part: &str) -> bool {
    if part.is_empty() { return false; }
    
    let bytes = part.as_bytes();
    if bytes[bytes.len() - 1] == b'%' {
        let val_bytes = &bytes[..bytes.len() - 1];
        parse_f32_fast(val_bytes).map_or(false, |v| v >= 0.0 && v <= 100.0)
    } else {
        parse_f32_fast(bytes).map_or(false, |v| v >= 0.0 && v <= 1.0)
    }
}

#[inline(always)]
fn validate_rgb_value(part: &str) -> bool {
    if part.is_empty() { return false; }
    let bytes = part.as_bytes();
    
    if bytes.iter().any(|&b| b == b'.') {
        parse_f32_fast(bytes).map_or(false, |v| v >= 0.0 && v <= 255.9)
    } else {
        parse_u16_fast(bytes).is_some()
    }
}

#[inline(always)]
fn validate_hsl_angle(part: &str) -> bool {
    if part.is_empty() { return false; }
    
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
    
    !numeric_part.is_empty() && parse_f32_fast(numeric_part.as_bytes()).is_some()
}

#[inline(always)]
fn validate_hsl_percent(part: &str) -> bool {
    if part.is_empty() || !part.ends_with('%') { return false; }
    let val_bytes = &part.as_bytes()[..part.len() - 1];
    parse_f32_fast(val_bytes).map_or(false, |v| v >= 0.0 && v <= 100.0)
}

#[inline(always)]
fn validate_rgb_parts_fast(inner: &str, has_alpha: bool) -> bool {
    let parts = parse_parts_no_alloc(inner);
    
    if has_alpha {
        if parts[1] != "" && parts[2] == "" && is_color_keyword(parts[0]) {
            return validate_alpha_part(parts[1]);
        }
        
        if parts[3] == "" { return false; }
        
        for i in 0..3 {
            if parts[i] == "" || !validate_rgb_value(parts[i]) {
                return false;
            }
        }
        
        validate_alpha_part(parts[3])
    } else {
        if parts[2] == "" { return false; }
        if parts[3] != "" { return false; }
        
        for i in 0..3 {
            if parts[i] == "" || !validate_rgb_value(parts[i]) {
                return false;
            }
        }
        
        true
    }
}

#[inline(always)]
fn validate_hsl_parts_fast(inner: &str, has_alpha: bool) -> bool {
    let parts = parse_parts_no_alloc(inner);
    
    if has_alpha {
        if parts[1] != "" && parts[2] == "" && is_color_keyword(parts[0]) {
            return validate_alpha_part(parts[1]);
        }
        
        if parts[3] == "" { return false; }
        
        if parts[0] == "" || !validate_hsl_angle(parts[0]) {
            return false;
        }
        
        for i in 1..3 {
            if parts[i] == "" || !validate_hsl_percent(parts[i]) {
                return false;
            }
        }
        
        validate_alpha_part(parts[3])
    } else {
        if parts[2] == "" { return false; }
        if parts[3] != "" { return false; }
        
        if parts[0] == "" || !validate_hsl_angle(parts[0]) {
            return false;
        }
        
        for i in 1..3 {
            if parts[i] == "" || !validate_hsl_percent(parts[i]) {
                return false;
            }
        }
        
        true
    }
}

pub fn is_valid_color(trimmed_lower: &str) -> bool {
    if trimmed_lower.is_empty() {
        return false;
    }

    let bytes = trimmed_lower.as_bytes();

    if bytes[0] == b'#' {
        let hex_part = &bytes[1..];
        let len = hex_part.len();
        return matches!(len, 3 | 4 | 6 | 8) && hex_part.iter().all(|&b| is_hex_char(b));
    }

    if bytes.len() >= 4 && &bytes[..3] == b"rgb" {
        let has_alpha = bytes.get(3) == Some(&b'a');
        let start_pos = if has_alpha { 4 } else { 3 };
        
        if bytes.len() <= start_pos + 1 
            || bytes[start_pos] != b'(' 
            || bytes[bytes.len() - 1] != b')' {
            return false;
        }

        let inner = &trimmed_lower[start_pos + 1..trimmed_lower.len() - 1];
        return validate_rgb_parts_fast(inner, has_alpha);
    }

    if bytes.len() >= 4 && &bytes[..3] == b"hsl" {
        let has_alpha = bytes.get(3) == Some(&b'a');
        let start_pos = if has_alpha { 4 } else { 3 };
        
        if bytes.len() <= start_pos + 1 
            || bytes[start_pos] != b'(' 
            || bytes[bytes.len() - 1] != b')' {
            return false;
        }

        let inner = &trimmed_lower[start_pos + 1..trimmed_lower.len() - 1];
        return validate_hsl_parts_fast(inner, has_alpha);
    }

    is_color_keyword(trimmed_lower)
}