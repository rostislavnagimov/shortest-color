use std::collections::HashMap;
use std::sync::LazyLock;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.a == 255 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "rgba({}, {}, {}, {:.3})",
                self.r,
                self.g,
                self.b,
                self.a as f32 / 255.0
            )
        }
    }
}

const HEX_MAP: [u8; 128] = {
    let mut map = [255; 128];
    let mut i = 48;
    while i <= 57 {
        map[i] = (i - 48) as u8;
        i += 1;
    }
    i = 65;
    while i <= 70 {
        map[i] = (i - 55) as u8;
        i += 1;
    }
    i = 97;
    while i <= 102 {
        map[i] = (i - 87) as u8;
        i += 1;
    }
    map
};

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    if b < 128 {
        let val = HEX_MAP[b as usize];
        if val > 15 {
            None
        } else {
            Some(val)
        }
    } else {
        None
    }
}

#[inline]
fn parse_hex2(bytes: &[u8], i: usize) -> Option<u8> {
    let high = hex_val(bytes[i])?;
    let low = hex_val(bytes[i + 1])?;
    Some(high << 4 | low)
}

#[inline]
fn parse_hex1(b: u8) -> Option<u8> {
    let val = hex_val(b)?;
    Some(val << 4 | val)
}

fn parse_rgb_value(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() || bytes.len() > 6 || bytes[0] == b'-' {
        return None;
    }

    let mut result = 0.0f32;
    let mut decimal_places = 0u32;

    for &b in bytes {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 4 {
                        break;
                    }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                    if result > 255.9 {
                        return None;
                    }
                }
            }
            b'.' => {
                if decimal_places > 0 {
                    return None;
                }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    Some((result + 0.5) as u8)
}

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
        if num_bytes.len() == 1 {
            return None;
        }
    }

    for &b in &num_bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 3 {
                        break;
                    }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                }
            }
            b'.' => {
                if decimal_places > 0 {
                    return None;
                }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    let final_result = if negative { -result } else { result };
    if !(0.0..=100.0).contains(&final_result) {
        return None;
    }

    Some(final_result)
}

fn parse_alpha_value(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }

    if bytes.ends_with(b"%") {
        let percentage = parse_percentage(bytes)?;
        return Some((percentage * 2.55 + 0.5) as u8);
    }

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
                    if decimal_places > 4 {
                        break;
                    }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                }
            }
            b'.' => {
                if decimal_places > 0 {
                    return None;
                }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    if result > 1.0 {
        return None;
    }
    Some((result * 255.0 + 0.5) as u8)
}

fn parse_hue_value(s: &str) -> Option<f32> {
    let (h_val, unit) = if let Some(stripped) = s.strip_suffix("grad") {
        (parse_float(stripped)?, "grad")
    } else if let Some(stripped) = s.strip_suffix("turn") {
        (parse_float(stripped)?, "turn")
    } else if let Some(stripped) = s.strip_suffix("deg") {
        (parse_float(stripped)?, "deg")
    } else if let Some(stripped) = s.strip_suffix("rad") {
        (parse_float(stripped)?, "rad")
    } else {
        (parse_float(s)?, "")
    };

    Some(normalize_angle(h_val, unit))
}

fn parse_float(s: &str) -> Option<f32> {
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
        if bytes.len() == 1 {
            return None;
        }
    }

    for &b in &bytes[pos..] {
        match b {
            b'0'..=b'9' => {
                if decimal_places > 0 {
                    decimal_places += 1;
                    if decimal_places > 4 {
                        break;
                    }
                    result += (b - b'0') as f32 / (10u32.pow(decimal_places - 1)) as f32;
                } else {
                    result = result * 10.0 + (b - b'0') as f32;
                }
            }
            b'.' => {
                if decimal_places > 0 {
                    return None;
                }
                decimal_places = 1;
            }
            _ => return None,
        }
    }

    Some(if negative { -result } else { result })
}

fn normalize_angle(value: f32, unit: &str) -> f32 {
    let degrees = match unit {
        "rad" => value * 57.295_78,
        "grad" => value * 0.9,
        "turn" => value * 360.0,
        _ => value,
    };
    ((degrees % 360.0) + 360.0) % 360.0
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_sector = h * (1.0 / 60.0);
    let x = c * (1.0 - ((h_sector % 2.0) - 1.0).abs());
    let m = l - c * 0.5;

    let (r, g, b) = match h_sector as u8 {
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

fn split_params(s: &str) -> [&str; 4] {
    let mut parts = [""; 4];
    let mut count = 0;
    let mut start = 0;

    for (i, &b) in s.as_bytes().iter().enumerate() {
        if (b == b',' || b == b' ') && count < 4 {
            if i > start {
                let part = s[start..i].trim();
                if !part.is_empty() {
                    parts[count] = part;
                    count += 1;
                }
            }
            start = i + 1;
            // Пропускаем подряд идущие разделители
            while start < s.len() && (s.as_bytes()[start] == b',' || s.as_bytes()[start] == b' ') {
                start += 1;
            }
        }
    }

    if start < s.len() && count < 4 {
        let part = s[start..].trim();
        if !part.is_empty() {
            parts[count] = part;
        }
    }

    parts
}

fn resolve_keyword(name: &str) -> Option<Color> {
    if name.is_empty() || name.len() > 20 {
        return None;
    }

    KEYWORDS
        .binary_search_by_key(&name, |&(keyword, _)| keyword)
        .ok()
        .and_then(|idx| parse(KEYWORDS[idx].1))
}

pub fn parse(color_str: &str) -> Option<Color> {
    let bytes = color_str.as_bytes();

    if bytes.is_empty() {
        return None;
    }

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

    // Быстрая проверка для ключевых слов (избегаем дорогие string операции)
    if color_str.len() <= 20 && !color_str.contains('(') {
        return resolve_keyword(color_str);
    }

    // Только теперь проверяем функции
    let end_paren = color_str.len().wrapping_sub(1);
    if end_paren == 0 || !color_str.ends_with(')') {
        return resolve_keyword(color_str);
    }

    let (func, has_alpha, content) = if color_str.starts_with("rgba(") {
        ("rgb", true, &color_str[5..color_str.len() - 1])
    } else if color_str.starts_with("rgb(") {
        ("rgb", false, &color_str[4..color_str.len() - 1])
    } else if color_str.starts_with("hsla(") {
        ("hsl", true, &color_str[5..color_str.len() - 1])
    } else if color_str.starts_with("hsl(") {
        ("hsl", false, &color_str[4..color_str.len() - 1])
    } else {
        return resolve_keyword(color_str);
    };

    let parts = split_params(content);

    if func == "rgb" {
        if has_alpha {
            if !parts[1].is_empty() && parts[2].is_empty() {
                if let Some(base) = resolve_keyword(parts[0]) {
                    let a = parse_alpha_value(parts[1].as_bytes())?;
                    return Some(Color {
                        r: base.r,
                        g: base.g,
                        b: base.b,
                        a,
                    });
                }
            }
            if parts[3].is_empty() {
                return None;
            }

            let r = parse_rgb_value(parts[0].as_bytes())?;
            let g = parse_rgb_value(parts[1].as_bytes())?;
            let b = parse_rgb_value(parts[2].as_bytes())?;
            let a = parse_alpha_value(parts[3].as_bytes())?;
            Some(Color { r, g, b, a })
        } else {
            if parts[2].is_empty() || !parts[3].is_empty() {
                return None;
            }
            let r = parse_rgb_value(parts[0].as_bytes())?;
            let g = parse_rgb_value(parts[1].as_bytes())?;
            let b = parse_rgb_value(parts[2].as_bytes())?;
            Some(Color { r, g, b, a: 255 })
        }
    } else if has_alpha {
        if !parts[1].is_empty() && parts[2].is_empty() {
            if let Some(base) = resolve_keyword(parts[0]) {
                let a = parse_alpha_value(parts[1].as_bytes())?;
                return Some(Color {
                    r: base.r,
                    g: base.g,
                    b: base.b,
                    a,
                });
            }
        }
        if parts[3].is_empty() {
            return None;
        }

        let h = parse_hue_value(parts[0])?;
        let s = parse_percentage(parts[1].as_bytes())?;
        let l = parse_percentage(parts[2].as_bytes())?;
        let (r, g, b) = hsl_to_rgb(h, s, l);
        let a = parse_alpha_value(parts[3].as_bytes())?;
        Some(Color { r, g, b, a })
    } else {
        if parts[2].is_empty() || !parts[3].is_empty() {
            return None;
        }
        let h = parse_hue_value(parts[0])?;
        let s = parse_percentage(parts[1].as_bytes())?;
        let l = parse_percentage(parts[2].as_bytes())?;
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Some(Color { r, g, b, a: 255 })
    }
}

#[inline]
fn can_shorten_hex(r: u8, g: u8, b: u8, a: u8) -> bool {
    ((r ^ (r << 4)) | (g ^ (g << 4)) | (b ^ (b << 4)) | (a ^ (a << 4))) & 0xF0 == 0
}

static NAME_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(KEYWORDS.len());
    for &(name, hex_val) in KEYWORDS {
        if let Some(color) = parse(hex_val) {
            if color.a == 255 && name.len() <= 7 {
                let rgb = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
                map.entry(rgb).or_insert(name);
            }
        }
    }
    map
});

pub fn shorten(color: &Color) -> String {
    if color.a != 255 {
        return if can_shorten_hex(color.r, color.g, color.b, color.a) {
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
        };
    }

    let can_short = can_shorten_hex(color.r, color.g, color.b, 255);
    let short_hex = format!("#{:x}{:x}{:x}", color.r >> 4, color.g >> 4, color.b >> 4);
    let full_hex = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);

    let mut result = if can_short && short_hex.len() < full_hex.len() {
        short_hex
    } else {
        full_hex
    };

    let rgb = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
    if let Some(&name) = NAME_MAP.get(&rgb) {
        if name.len() < result.len() {
            result = name.to_string();
        }
    }

    result
}

pub fn shorten_css_color(color_str: &str) -> String {
    let trimmed = color_str.trim().to_ascii_lowercase();

    if trimmed.len() < 5 {
        return if trimmed == "#f00" {
            "red".to_string()
        } else {
            trimmed
        };
    }

    match parse(&trimmed) {
        Some(color) => shorten(&color),
        None => color_str.to_string(),
    }
}

pub const KEYWORDS: &[(&str, &str)] = &[
    ("aliceblue", "#f0f8ff"),
    ("antiquewhite", "#faebd7"),
    ("aqua", "#0ff"),
    ("aquamarine", "#7fffd4"),
    ("azure", "#f0ffff"),
    ("beige", "#f5f5dc"),
    ("bisque", "#ffe4c4"),
    ("black", "#000"),
    ("blanchedalmond", "#ffebcd"),
    ("blue", "#00f"),
    ("blueviolet", "#8a2be2"),
    ("brown", "#a52a2a"),
    ("burlywood", "#deb887"),
    ("cadetblue", "#5f9ea0"),
    ("chartreuse", "#7fff00"),
    ("chocolate", "#d2691e"),
    ("coral", "#ff7f50"),
    ("cornflowerblue", "#6495ed"),
    ("cornsilk", "#fff8dc"),
    ("crimson", "#dc143c"),
    ("cyan", "#0ff"),
    ("darkblue", "#00008b"),
    ("darkcyan", "#008b8b"),
    ("darkgoldenrod", "#b8860b"),
    ("darkgray", "#a9a9a9"),
    ("darkgreen", "#006400"),
    ("darkgrey", "#a9a9a9"),
    ("darkkhaki", "#bdb76b"),
    ("darkmagenta", "#8b008b"),
    ("darkolivegreen", "#556b2f"),
    ("darkorange", "#ff8c00"),
    ("darkorchid", "#9932cc"),
    ("darkred", "#8b0000"),
    ("darksalmon", "#e9967a"),
    ("darkseagreen", "#8fbc8f"),
    ("darkslateblue", "#483d8b"),
    ("darkslategray", "#2f4f4f"),
    ("darkslategrey", "#2f4f4f"),
    ("darkturquoise", "#00ced1"),
    ("darkviolet", "#9400d3"),
    ("deeppink", "#ff1493"),
    ("deepskyblue", "#00bfff"),
    ("dimgray", "#696969"),
    ("dimgrey", "#696969"),
    ("dodgerblue", "#1e90ff"),
    ("firebrick", "#b22222"),
    ("floralwhite", "#fffaf0"),
    ("forestgreen", "#228b22"),
    ("fuchsia", "#f0f"),
    ("gainsboro", "#dcdcdc"),
    ("ghostwhite", "#f8f8ff"),
    ("gold", "#ffd700"),
    ("goldenrod", "#daa520"),
    ("gray", "#808080"),
    ("green", "#008000"),
    ("greenyellow", "#adff2f"),
    ("grey", "#808080"),
    ("honeydew", "#f0fff0"),
    ("hotpink", "#ff69b4"),
    ("indianred", "#cd5c5c"),
    ("indigo", "#4b0082"),
    ("ivory", "#fffff0"),
    ("khaki", "#f0e68c"),
    ("lavender", "#e6e6fa"),
    ("lavenderblush", "#fff0f5"),
    ("lawngreen", "#7cfc00"),
    ("lemonchiffon", "#fffacd"),
    ("lightblue", "#add8e6"),
    ("lightcoral", "#f08080"),
    ("lightcyan", "#e0ffff"),
    ("lightgoldenrodyellow", "#fafad2"),
    ("lightgray", "#d3d3d3"),
    ("lightgreen", "#90ee90"),
    ("lightgrey", "#d3d3d3"),
    ("lightpink", "#ffb6c1"),
    ("lightsalmon", "#ffa07a"),
    ("lightseagreen", "#20b2aa"),
    ("lightskyblue", "#87cefa"),
    ("lightslategray", "#778899"),
    ("lightslategrey", "#778899"),
    ("lightsteelblue", "#b0c4de"),
    ("lightyellow", "#ffffe0"),
    ("lime", "#0f0"),
    ("limegreen", "#32cd32"),
    ("linen", "#faf0e6"),
    ("magenta", "#f0f"),
    ("maroon", "#800000"),
    ("mediumaquamarine", "#66cdaa"),
    ("mediumblue", "#0000cd"),
    ("mediumorchid", "#ba55d3"),
    ("mediumpurple", "#9370db"),
    ("mediumseagreen", "#3cb371"),
    ("mediumslateblue", "#7b68ee"),
    ("mediumspringgreen", "#00fa9a"),
    ("mediumturquoise", "#48d1cc"),
    ("mediumvioletred", "#c71585"),
    ("midnightblue", "#191970"),
    ("mintcream", "#f5fffa"),
    ("mistyrose", "#ffe4e1"),
    ("moccasin", "#ffe4b5"),
    ("navajowhite", "#ffdead"),
    ("navy", "#000080"),
    ("oldlace", "#fdf5e6"),
    ("olive", "#808000"),
    ("olivedrab", "#6b8e23"),
    ("orange", "#ffa500"),
    ("orangered", "#ff4500"),
    ("orchid", "#da70d6"),
    ("palegoldenrod", "#eee8aa"),
    ("palegreen", "#98fb98"),
    ("paleturquoise", "#afeeee"),
    ("palevioletred", "#db7093"),
    ("papayawhip", "#ffefd5"),
    ("peachpuff", "#ffdab9"),
    ("peru", "#cd853f"),
    ("pink", "#ffc0cb"),
    ("plum", "#dda0dd"),
    ("powderblue", "#b0e0e6"),
    ("purple", "#800080"),
    ("rebeccapurple", "#639"),
    ("red", "#f00"),
    ("rosybrown", "#bc8f8f"),
    ("royalblue", "#4169e1"),
    ("saddlebrown", "#8b4513"),
    ("salmon", "#fa8072"),
    ("sandybrown", "#f4a460"),
    ("seagreen", "#2e8b57"),
    ("seashell", "#fff5ee"),
    ("sienna", "#a0522d"),
    ("silver", "#c0c0c0"),
    ("skyblue", "#87ceeb"),
    ("slateblue", "#6a5acd"),
    ("slategray", "#708090"),
    ("slategrey", "#708090"),
    ("snow", "#fffafa"),
    ("springgreen", "#00ff7f"),
    ("steelblue", "#4682b4"),
    ("tan", "#d2b48c"),
    ("teal", "#008080"),
    ("thistle", "#d8bfd8"),
    ("tomato", "#ff6347"),
    ("transparent", "#0000"),
    ("turquoise", "#40e0d0"),
    ("violet", "#ee82ee"),
    ("wheat", "#f5deb3"),
    ("white", "#fff"),
    ("whitesmoke", "#f5f5f5"),
    ("yellow", "#ff0"),
    ("yellowgreen", "#9acd32"),
];
