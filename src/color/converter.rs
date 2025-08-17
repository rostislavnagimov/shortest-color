use super::model::Color;
use super::keywords::COLOR_KEYWORDS;

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s / 100.0;
    let l = l / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
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

pub fn convert_to_color(color_str: &str) -> Option<Color> {
    let trimmed = color_str.trim().to_lowercase();
    
    if let Some(hex) = trimmed.strip_prefix('#') {
        let (r, g, b, a) = match hex.len() {
            3 => (
                u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
                255,
            ),
            4 => (
                u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?,
            ),
            6 => (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                255,
            ),
            8 => (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                u8::from_str_radix(&hex[6..8], 16).ok()?,
            ),
            _ => return None,
        };
        return Some(Color { r, g, b, a });
    }
    
    if let Some(body) = trimmed.strip_prefix("rgb").and_then(|s| s.strip_suffix(')')) {
        let (body, has_alpha) = if let Some(body_a) = body.strip_prefix('a') {
            (body_a, true)
        } else {
            (body, false)
        };
        
        let Some(inner) = body.strip_prefix('(') else { return None };
        let parts: Vec<&str> = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
            
        if has_alpha && parts.len() == 4 {
            let r = parts[0].parse().ok()?;
            let g = parts[1].parse().ok()?;
            let b = parts[2].parse().ok()?;
            let a_f32 = parts[3].trim_end_matches('%').parse::<f32>().ok()?;
            let a = if parts[3].contains('%') {
                (a_f32 / 100.0 * 255.0).round() as u8
            } else {
                (a_f32 * 255.0).round() as u8
            };
            return Some(Color { r, g, b, a });
        } else if !has_alpha && parts.len() == 3 {
            let r = parts[0].parse().ok()?;
            let g = parts[1].parse().ok()?;
            let b = parts[2].parse().ok()?;
            return Some(Color { r, g, b, a: 255 });
        }
    }
    
    if let Some(body) = trimmed.strip_prefix("hsl").and_then(|s| s.strip_suffix(')')) {
        let (body, has_alpha) = if let Some(body_a) = body.strip_prefix('a') {
            (body_a, true)
        } else {
            (body, false)
        };
        
        let Some(inner) = body.strip_prefix('(') else { return None };
        let parts: Vec<&str> = inner
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
            
        if has_alpha && parts.len() == 4 {
            let h = parts[0]
                .trim_end_matches(|c: char| !c.is_numeric() && c != '.')
                .parse()
                .ok()?;
            let s = parts[1].trim_end_matches('%').parse().ok()?;
            let l = parts[2].trim_end_matches('%').parse().ok()?;
            let (r, g, b) = hsl_to_rgb(h, s, l);
            let a_f32 = parts[3].trim_end_matches('%').parse::<f32>().ok()?;
            let a = if parts[3].contains('%') {
                (a_f32 / 100.0 * 255.0).round() as u8
            } else {
                (a_f32 * 255.0).round() as u8
            };
            return Some(Color { r, g, b, a });
        } else if !has_alpha && parts.len() == 3 {
            let h = parts[0]
                .trim_end_matches(|c: char| !c.is_numeric() && c != '.')
                .parse()
                .ok()?;
            let s = parts[1].trim_end_matches('%').parse().ok()?;
            let l = parts[2].trim_end_matches('%').parse().ok()?;
            let (r, g, b) = hsl_to_rgb(h, s, l);
            return Some(Color { r, g, b, a: 255 });
        }
    }
    
    for &(name, hex) in COLOR_KEYWORDS {
        if trimmed == name {
            return convert_to_color(hex);
        }
    }
    
    None
}

pub fn color_to_hex(color: &Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", color.r, color.g, color.b, color.a)
    }
}