use std::collections::HashMap;
use std::sync::LazyLock;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct C {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const RAD_TO_DEG: f32 = 57.29578;

#[inline(always)]
const fn hex_to_value(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'A'..=b'F' => b - b'A' + 10,
        b'a'..=b'f' => b - b'a' + 10,
        _ => 255,
    }
}

#[inline(always)]
fn ascii_case_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(&x, &y)| (x | 0x20) == (y | 0x20))
}

#[inline(always)]
const fn parse_hex_pair(b: &[u8], i: usize) -> Option<u8> {
    let h = hex_to_value(b[i]);
    let l = hex_to_value(b[i + 1]);
    if h == 255 || l == 255 {
        None
    } else {
        Some((h << 4) | l)
    }
}

#[inline(always)]
const fn parse_hex_single(b: u8) -> Option<u8> {
    let v = hex_to_value(b);
    if v == 255 {
        None
    } else {
        Some((v << 4) | v)
    }
}

#[inline(always)]
fn parse_number(bytes: &[u8], max_val: f32, allow_negative: bool) -> Option<f32> {
    let len = bytes.len();
    if len == 0 || len > 8 {
        return None;
    }

    let mut result = 0.0f32;
    let mut has_dot = false;
    let mut divisor = 10.0f32;
    let negative = allow_negative && bytes[0] == b'-';
    let start = if negative {
        if len == 1 { return None; }
        1
    } else {
        if !allow_negative && bytes[0] == b'-' { return None; }
        0
    };

    for &c in &bytes[start..] {
        match c {
            b'0'..=b'9' => {
                let digit = (c - b'0') as f32;
                if has_dot {
                    result += digit / divisor;
                    divisor *= 10.0;
                    if divisor > 10000.0 { break; }
                } else {
                    result = result * 10.0 + digit;
                    if result > max_val && !negative { return None; }
                }
            }
            b'.' => {
                if has_dot { return None; }
                has_dot = true;
            }
            _ => return None,
        }
    }

    let final_result = if negative { -result } else { result };
    if final_result > max_val || (!allow_negative && final_result < 0.0) {
        None
    } else {
        Some(final_result)
    }
}

#[inline(always)]
fn parse_rgb_component(b: &[u8]) -> Option<u8> {
    let len = b.len();
    if len == 0 || len > 6 {
        return None;
    }

    if !b.contains(&b'.') {
        let mut result = 0u32;
        for &c in b {
            if !c.is_ascii_digit() {
                return None;
            }
            result = result * 10 + (c - b'0') as u32;
            if result > 255 {
                return None;
            }
        }
        return Some(result as u8);
    }

    let r = parse_number(b, 255.9, false)?;
    Some((r + 0.5) as u8)
}

#[inline(always)]
fn parse_percentage(b: &[u8]) -> Option<f32> {
    let len = b.len();
    if !(2..=6).contains(&len) || b[len - 1] != b'%' {
        return None;
    }

    let number_part = &b[..len - 1];
    let result = parse_number(number_part, 100.0, true)?;
    if !(0.0..=100.0).contains(&result) {
        return None;
    }
    Some(result)
}

#[inline(always)]
fn parse_alpha_component(b: &[u8]) -> Option<u8> {
    if b.is_empty() {
        return None;
    }

    if b[b.len() - 1] == b'%' {
        let percentage = parse_percentage(b)?;
        return Some((percentage * 2.55 + 0.5) as u8);
    }

    let result = parse_number(b, 1.0, false)?;
    Some((result * 255.0 + 0.5) as u8)
}

#[inline(always)]
fn ends_with_ignore_case(slice: &[u8], suffix: &[u8]) -> bool {
    let slice_len = slice.len();
    let suffix_len = suffix.len();
    slice_len >= suffix_len && ascii_case_eq(&slice[slice_len - suffix_len..], suffix)
}

#[inline(always)]
fn parse_angle(s: &str) -> Option<f32> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }

    let (number_str, multiplier) = if ends_with_ignore_case(bytes, b"grad") {
        (&s[..len - 4], 0.9)
    } else if ends_with_ignore_case(bytes, b"turn") {
        (&s[..len - 4], 360.0)
    } else if ends_with_ignore_case(bytes, b"deg") {
        (&s[..len - 3], 1.0)
    } else if ends_with_ignore_case(bytes, b"rad") {
        (&s[..len - 3], RAD_TO_DEG)
    } else {
        (s, 1.0)
    };

    let hue = parse_number(number_str.as_bytes(), f32::MAX, true)?;
    Some(((hue * multiplier % 360.0) + 360.0) % 360.0)
}

#[inline(always)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s_norm = s * 0.01;
    let l_norm = l * 0.01;
    let chroma = (1.0 - (2.0 * l_norm - 1.0).abs()) * s_norm;
    let hue_sector = h / 60.0;
    let x = chroma * (1.0 - ((hue_sector % 2.0) - 1.0).abs());
    let m = l_norm - chroma * 0.5;

    let (r, g, b) = match hue_sector as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };

    (
        ((r + m) * 255.0 + 0.5) as u8,
        ((g + m) * 255.0 + 0.5) as u8,
        ((b + m) * 255.0 + 0.5) as u8,
    )
}

#[inline(always)]
fn parse_hex_color(s: &[u8]) -> Option<C> {
    let len = s.len();
    if len == 0 || s[0] != b'#' {
        return None;
    }
    let hex_part = &s[1..];
    match hex_part.len() {
        3 => Some(C {
            r: parse_hex_single(hex_part[0])?,
            g: parse_hex_single(hex_part[1])?,
            b: parse_hex_single(hex_part[2])?,
            a: 255,
        }),
        4 => Some(C {
            r: parse_hex_single(hex_part[0])?,
            g: parse_hex_single(hex_part[1])?,
            b: parse_hex_single(hex_part[2])?,
            a: parse_hex_single(hex_part[3])?,
        }),
        6 => Some(C {
            r: parse_hex_pair(hex_part, 0)?,
            g: parse_hex_pair(hex_part, 2)?,
            b: parse_hex_pair(hex_part, 4)?,
            a: 255,
        }),
        8 => Some(C {
            r: parse_hex_pair(hex_part, 0)?,
            g: parse_hex_pair(hex_part, 2)?,
            b: parse_hex_pair(hex_part, 4)?,
            a: parse_hex_pair(hex_part, 6)?,
        }),
        _ => None,
    }
}

#[inline(always)]
fn lookup_color_name(name: &[u8]) -> Option<C> {
    K.iter()
        .find(|&&(n, _)| ascii_case_eq(n, name))
        .and_then(|&(_, hex)| parse_hex_color(hex))
}

#[inline(always)]
fn parse_function_args(args: &[u8]) -> Option<([&[u8]; 4], usize)> {
    let mut parts = [&[][..]; 4];
    let mut count = 0;
    let mut start = 0;
    let mut in_arg = false;

    for i in 0..args.len() {
        let b = args[i];
        let is_separator = matches!(b, b' ' | b'\t' | b',');

        if !is_separator && !in_arg {
            start = i;
            in_arg = true;
        } else if is_separator && in_arg {
            if count >= 4 { return None; }
            parts[count] = &args[start..i];
            count += 1;
            in_arg = false;
        }
    }

    if in_arg {
        if count >= 4 { return None; }
        parts[count] = &args[start..];
        count += 1;
    }

    if (2..=4).contains(&count) {
        Some((parts, count))
    } else {
        None
    }
}

#[inline(always)]
fn parse_color(s: &[u8]) -> Option<C> {
    let len = s.len();
    if len == 0 {
        return None;
    }

    if s[0] == b'#' {
        return parse_hex_color(s);
    }

    if len < 4 || (s[3] != b'(' && s[4] != b'(') {
        return lookup_color_name(s);
    }

    if s[len - 1] != b')' {
        return None;
    }

    let (func_type, has_alpha, start) = match &s[..3] {
        prefix if ascii_case_eq(b"rgb", prefix) => {
            if s[3] == b'(' {
                (0, false, 4)
            } else if s[3] == b'a' {
                (0, true, 5)
            } else {
                return None;
            }
        }
        prefix if ascii_case_eq(b"hsl", prefix) => {
            if s[3] == b'(' {
                (1, false, 4)
            } else if s[3] == b'a' {
                (1, true, 5)
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let (parts, count) = parse_function_args(&s[start..len - 1])?;

    if has_alpha && count == 2 {
        let name = parts[0];
        if let Some(base_color) = lookup_color_name(name) {
            let alpha = parse_alpha_component(parts[1])?;
            return Some(C { a: alpha, ..base_color });
        }
        return None;
    }

    let expected_parts = if has_alpha { 4 } else { 3 };
    if count != expected_parts {
        return None;
    }

    let alpha = if has_alpha {
        parse_alpha_component(parts[3])?
    } else {
        255
    };

    match func_type {
        0 => {
            let r = parse_rgb_component(parts[0])?;
            let g = parse_rgb_component(parts[1])?;
            let b = parse_rgb_component(parts[2])?;
            Some(C { r, g, b, a: alpha })
        }
        _ => {
            let hue_str = std::str::from_utf8(parts[0]).ok()?;
            let hue = parse_angle(hue_str)?;
            let saturation = parse_percentage(parts[1])?;
            let lightness = parse_percentage(parts[2])?;
            let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
            Some(C { r, g, b, a: alpha })
        }
    }
}

#[inline(always)]
const fn is_short_hex(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) * 0x11 == r
        && (g & 0x0F) * 0x11 == g
        && (b & 0x0F) * 0x11 == b
        && (a & 0x0F) * 0x11 == a
}

static COLOR_LOOKUP: LazyLock<Vec<(u32, &'static [u8])>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for &(name, hex) in K {
        if let Some(color) = parse_hex_color(hex) {
            if color.a == 255 {
                let rgb = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);
                map.entry(rgb).or_insert(name);
            }
        }
    }
    let mut vec: Vec<_> = map.into_iter().collect();
    vec.sort_unstable_by_key(|&(rgb, _)| rgb);
    vec
});

#[inline(always)]
fn find_color_name(rgb: u32) -> Option<&'static [u8]> {
    COLOR_LOOKUP
        .binary_search_by_key(&rgb, |&(r, _)| r)
        .ok()
        .map(|i| COLOR_LOOKUP[i].1)
}

#[inline(always)]
const fn hex_digit(n: u8) -> u8 {
    match n {
        0..=9 => b'0' + n,
        _ => b'a' + (n - 10),
    }
}

#[inline(always)]
fn color_to_string(color: &C) -> String {
    if color.a != 255 {
        let short = is_short_hex(color.r, color.g, color.b, color.a);
        let mut buf = [0u8; 9];
        let slice = if short {
            buf[0] = b'#';
            buf[1] = hex_digit(color.r >> 4);
            buf[2] = hex_digit(color.g >> 4);
            buf[3] = hex_digit(color.b >> 4);
            buf[4] = hex_digit(color.a >> 4);
            &buf[..5]
        } else {
            buf[0] = b'#';
            buf[1] = hex_digit(color.r >> 4);
            buf[2] = hex_digit(color.r & 0xF);
            buf[3] = hex_digit(color.g >> 4);
            buf[4] = hex_digit(color.g & 0xF);
            buf[5] = hex_digit(color.b >> 4);
            buf[6] = hex_digit(color.b & 0xF);
            buf[7] = hex_digit(color.a >> 4);
            buf[8] = hex_digit(color.a & 0xF);
            &buf[..9]
        };
        return unsafe { std::str::from_utf8_unchecked(slice) }.to_string();
    }

    let rgb = ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32);

    if let Some(name) = find_color_name(rgb) {
        let short = is_short_hex(color.r, color.g, color.b, 255);
        let max_len = if short { 4 } else { 7 };
        if name.len() < max_len {
            return unsafe { std::str::from_utf8_unchecked(name) }.to_string();
        }
    }

    let short = is_short_hex(color.r, color.g, color.b, 255);
    let mut buf = [0u8; 7];
    let slice = if short {
        buf[0] = b'#';
        buf[1] = hex_digit(color.r >> 4);
        buf[2] = hex_digit(color.g >> 4);
        buf[3] = hex_digit(color.b >> 4);
        &buf[..4]
    } else {
        buf[0] = b'#';
        buf[1] = hex_digit(color.r >> 4);
        buf[2] = hex_digit(color.r & 0xF);
        buf[3] = hex_digit(color.g >> 4);
        buf[4] = hex_digit(color.g & 0xF);
        buf[5] = hex_digit(color.b >> 4);
        buf[6] = hex_digit(color.b & 0xF);
        &buf[..7]
    };
    unsafe { std::str::from_utf8_unchecked(slice) }.to_string()
}

#[inline(always)]
fn trim_whitespace(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r')).unwrap_or(s.len());
    let end = s.iter().rposition(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r')).map_or(start, |i| i + 1);
    &s[start..end]
}

pub fn shorten_css_color(input: impl AsRef<str>) -> String {
    let s = input.as_ref().as_bytes();
    if s.is_empty() {
        return String::new();
    }
    
    let trimmed = trim_whitespace(s);
    
    if trimmed.len() < 5 {
        if ascii_case_eq(trimmed, b"#f00") {
            return "red".to_string();
        }
        return unsafe { std::str::from_utf8_unchecked(trimmed) }.to_ascii_lowercase();
    }

    match parse_color(trimmed) {
        Some(color) => color_to_string(&color),
        None => unsafe { std::str::from_utf8_unchecked(trimmed) }.to_ascii_lowercase(),
    }
}

const K: &[(&[u8], &[u8])] = &[
    (b"aliceblue", b"#f0f8ff"),
    (b"antiquewhite", b"#faebd7"),
    (b"aqua", b"#0ff"),
    (b"aquamarine", b"#7fffd4"),
    (b"azure", b"#f0ffff"),
    (b"beige", b"#f5f5dc"),
    (b"bisque", b"#ffe4c4"),
    (b"black", b"#000"),
    (b"blanchedalmond", b"#ffebcd"),
    (b"blue", b"#00f"),
    (b"blueviolet", b"#8a2be2"),
    (b"brown", b"#a52a2a"),
    (b"burlywood", b"#deb887"),
    (b"cadetblue", b"#5f9ea0"),
    (b"chartreuse", b"#7fff00"),
    (b"chocolate", b"#d2691e"),
    (b"coral", b"#ff7f50"),
    (b"cornflowerblue", b"#6495ed"),
    (b"cornsilk", b"#fff8dc"),
    (b"crimson", b"#dc143c"),
    (b"cyan", b"#0ff"),
    (b"darkblue", b"#00008b"),
    (b"darkcyan", b"#008b8b"),
    (b"darkgoldenrod", b"#b8860b"),
    (b"darkgray", b"#a9a9a9"),
    (b"darkgreen", b"#006400"),
    (b"darkgrey", b"#a9a9a9"),
    (b"darkkhaki", b"#bdb76b"),
    (b"darkmagenta", b"#8b008b"),
    (b"darkolivegreen", b"#556b2f"),
    (b"darkorange", b"#ff8c00"),
    (b"darkorchid", b"#9932cc"),
    (b"darkred", b"#8b0000"),
    (b"darksalmon", b"#e9967a"),
    (b"darkseagreen", b"#8fbc8f"),
    (b"darkslateblue", b"#483d8b"),
    (b"darkslategray", b"#2f4f4f"),
    (b"darkslategrey", b"#2f4f4f"),
    (b"darkturquoise", b"#00ced1"),
    (b"darkviolet", b"#9400d3"),
    (b"deeppink", b"#ff1493"),
    (b"deepskyblue", b"#00bfff"),
    (b"dimgray", b"#696969"),
    (b"dimgrey", b"#696969"),
    (b"dodgerblue", b"#1e90ff"),
    (b"firebrick", b"#b22222"),
    (b"floralwhite", b"#fffaf0"),
    (b"forestgreen", b"#228b22"),
    (b"fuchsia", b"#f0f"),
    (b"gainsboro", b"#dcdcdc"),
    (b"ghostwhite", b"#f8f8ff"),
    (b"gold", b"#ffd700"),
    (b"goldenrod", b"#daa520"),
    (b"gray", b"#808080"),
    (b"green", b"#008000"),
    (b"greenyellow", b"#adff2f"),
    (b"grey", b"#808080"),
    (b"honeydew", b"#f0fff0"),
    (b"hotpink", b"#ff69b4"),
    (b"indianred", b"#cd5c5c"),
    (b"indigo", b"#4b0082"),
    (b"ivory", b"#fffff0"),
    (b"khaki", b"#f0e68c"),
    (b"lavender", b"#e6e6fa"),
    (b"lavenderblush", b"#fff0f5"),
    (b"lawngreen", b"#7cfc00"),
    (b"lemonchiffon", b"#fffacd"),
    (b"lightblue", b"#add8e6"),
    (b"lightcoral", b"#f08080"),
    (b"lightcyan", b"#e0ffff"),
    (b"lightgoldenrodyellow", b"#fafad2"),
    (b"lightgray", b"#d3d3d3"),
    (b"lightgreen", b"#90ee90"),
    (b"lightgrey", b"#d3d3d3"),
    (b"lightpink", b"#ffb6c1"),
    (b"lightsalmon", b"#ffa07a"),
    (b"lightseagreen", b"#20b2aa"),
    (b"lightskyblue", b"#87cefa"),
    (b"lightslategray", b"#778899"),
    (b"lightslategrey", b"#778899"),
    (b"lightsteelblue", b"#b0c4de"),
    (b"lightyellow", b"#ffffe0"),
    (b"lime", b"#0f0"),
    (b"limegreen", b"#32cd32"),
    (b"linen", b"#faf0e6"),
    (b"magenta", b"#f0f"),
    (b"maroon", b"#800000"),
    (b"mediumaquamarine", b"#66cdaa"),
    (b"mediumblue", b"#0000cd"),
    (b"mediumorchid", b"#ba55d3"),
    (b"mediumpurple", b"#9370db"),
    (b"mediumseagreen", b"#3cb371"),
    (b"mediumslateblue", b"#7b68ee"),
    (b"mediumspringgreen", b"#00fa9a"),
    (b"mediumturquoise", b"#48d1cc"),
    (b"mediumvioletred", b"#c71585"),
    (b"midnightblue", b"#191970"),
    (b"mintcream", b"#f5fffa"),
    (b"mistyrose", b"#ffe4e1"),
    (b"moccasin", b"#ffe4b5"),
    (b"navajowhite", b"#ffdead"),
    (b"navy", b"#000080"),
    (b"oldlace", b"#fdf5e6"),
    (b"olive", b"#808000"),
    (b"olivedrab", b"#6b8e23"),
    (b"orange", b"#ffa500"),
    (b"orangered", b"#ff4500"),
    (b"orchid", b"#da70d6"),
    (b"palegoldenrod", b"#eee8aa"),
    (b"palegreen", b"#98fb98"),
    (b"paleturquoise", b"#afeeee"),
    (b"palevioletred", b"#db7093"),
    (b"papayawhip", b"#ffefd5"),
    (b"peachpuff", b"#ffdab9"),
    (b"peru", b"#cd853f"),
    (b"pink", b"#ffc0cb"),
    (b"plum", b"#dda0dd"),
    (b"powderblue", b"#b0e0e6"),
    (b"purple", b"#800080"),
    (b"rebeccapurple", b"#639"),
    (b"red", b"#f00"),
    (b"rosybrown", b"#bc8f8f"),
    (b"royalblue", b"#4169e1"),
    (b"saddlebrown", b"#8b4513"),
    (b"salmon", b"#fa8072"),
    (b"sandybrown", b"#f4a460"),
    (b"seagreen", b"#2e8b57"),
    (b"seashell", b"#fff5ee"),
    (b"sienna", b"#a0522d"),
    (b"silver", b"#c0c0c0"),
    (b"skyblue", b"#87ceeb"),
    (b"slateblue", b"#6a5acd"),
    (b"slategray", b"#708090"),
    (b"slategrey", b"#708090"),
    (b"snow", b"#fffafa"),
    (b"springgreen", b"#00ff7f"),
    (b"steelblue", b"#4682b4"),
    (b"tan", b"#d2b48c"),
    (b"teal", b"#008080"),
    (b"thistle", b"#d8bfd8"),
    (b"tomato", b"#ff6347"),
    (b"transparent", b"#0000"),
    (b"turquoise", b"#40e0d0"),
    (b"violet", b"#ee82ee"),
    (b"wheat", b"#f5deb3"),
    (b"white", b"#fff"),
    (b"whitesmoke", b"#f5f5f5"),
    (b"yellow", b"#ff0"),
    (b"yellowgreen", b"#9acd32"),
];
