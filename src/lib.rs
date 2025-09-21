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

#[inline(always)]
const fn hex_to_value(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'A'..=b'F' => b - b'A' + 10,
        b'a'..=b'f' => b - b'a' + 10,
        _ => 255,
    }
}

fn ascii_case_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if (a[i] | 0x20) != (b[i] | 0x20) {
            return false;
        }
    }
    true
}

#[inline(always)]
const fn u(b: &[u8], i: usize) -> Option<u8> {
    let h = hex_to_value(b[i]);
    let l = hex_to_value(b[i + 1]);
    if h == 255 || l == 255 {
        None
    } else {
        Some((h << 4) | l)
    }
}

#[inline(always)]
const fn v(b: u8) -> Option<u8> {
    let v = hex_to_value(b);
    if v == 255 {
        None
    } else {
        Some((v << 4) | v)
    }
}

#[inline(always)]
fn w(bytes: &[u8], m: f32, n: bool) -> Option<f32> {
    let len = bytes.len();
    if len == 0 || len > 8 {
        return None;
    }

    let mut r = 0.0f32;
    let mut has_dot = false;
    let mut divisor = 10.0f32;
    let neg = n && bytes[0] == b'-';
    let p = if neg {
        if len == 1 {
            return None;
        }
        1
    } else {
        if !n && bytes[0] == b'-' {
            return None;
        }
        0
    };

    let mut i = p;
    while i < len {
        let c = bytes[i];
        match c {
            b'0'..=b'9' => {
                let digit = (c - b'0') as f32;
                if has_dot {
                    r += digit / divisor;
                    divisor *= 10.0;
                    if divisor > 10000.0 {
                        break;
                    }
                } else {
                    r = r * 10.0 + digit;
                    if r > m && !neg {
                        return None;
                    }
                }
            }
            b'.' => {
                if has_dot {
                    return None;
                }
                has_dot = true;
            }
            _ => return None,
        }
        i += 1;
    }

    let res = if neg { -r } else { r };
    if res > m || (!n && res < 0.0) {
        None
    } else {
        Some(res)
    }
}

#[inline(always)]
fn x(b: &[u8]) -> Option<u8> {
    let len = b.len();
    if len == 0 || len > 6 {
        return None;
    }

    if !b.contains(&b'.') {
        let mut r = 0u32;
        for &c in b {
            if !c.is_ascii_digit() {
                return None;
            }
            r = r * 10 + (c - b'0') as u32;
            if r > 255 {
                return None;
            }
        }
        return Some(r as u8);
    }

    let r = w(b, 255.9, false)?;
    Some((r + 0.5) as u8)
}

#[inline(always)]
fn y(b: &[u8]) -> Option<f32> {
    let len = b.len();
    if !(2..=6).contains(&len) || b[len - 1] != b'%' {
        return None;
    }

    let n = &b[..len - 1];
    let r = w(n, 100.0, true)?;
    if !(0.0..=100.0).contains(&r) {
        return None;
    }
    Some(r)
}

#[inline(always)]
fn z(b: &[u8]) -> Option<u8> {
    let len = b.len();
    if len == 0 {
        return None;
    }

    if b[len - 1] == b'%' {
        let p = y(b)?;
        return Some((p * 2.55 + 0.5) as u8);
    }

    let r = w(b, 1.0, false)?;
    Some((r * 255.0 + 0.5) as u8)
}

#[inline(always)]
fn a(slice: &[u8], suffix: &[u8]) -> bool {
    let slice_len = slice.len();
    let suffix_len = suffix.len();
    slice_len >= suffix_len && ascii_case_eq(&slice[slice_len - suffix_len..], suffix)
}

#[inline(always)]
fn m(s: &str) -> Option<f32> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }

    let (np_str, mul) = if a(bytes, b"grad") {
        (&s[..len - 4], 0.9)
    } else if a(bytes, b"turn") {
        (&s[..len - 4], 360.0)
    } else if a(bytes, b"deg") {
        (&s[..len - 3], 1.0)
    } else if a(bytes, b"rad") {
        (&s[..len - 3], 57.295_78)
    } else {
        (s, 1.0)
    };

    let h = w(np_str.as_bytes(), f32::MAX, true)?;
    Some(((h * mul % 360.0) + 360.0) % 360.0)
}

#[inline(always)]
fn c(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hs = h / 60.0;
    let x = c * (1.0 - ((hs % 2.0) - 1.0).abs());
    let m = l - c * 0.5;

    let (r, g, b) = match hs as u8 {
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

#[inline(always)]
fn d(s: &[u8]) -> Option<C> {
    let len = s.len();
    if len == 0 || s[0] != b'#' {
        return None;
    }
    let h = &s[1..];
    match h.len() {
        3 => Some(C {
            r: v(h[0])?,
            g: v(h[1])?,
            b: v(h[2])?,
            a: 255,
        }),
        4 => Some(C {
            r: v(h[0])?,
            g: v(h[1])?,
            b: v(h[2])?,
            a: v(h[3])?,
        }),
        6 => Some(C {
            r: u(h, 0)?,
            g: u(h, 2)?,
            b: u(h, 4)?,
            a: 255,
        }),
        8 => Some(C {
            r: u(h, 0)?,
            g: u(h, 2)?,
            b: u(h, 4)?,
            a: u(h, 6)?,
        }),
        _ => None,
    }
}

#[inline(always)]
fn e(name: &[u8]) -> Option<C> {
    K.iter()
        .find(|&&(n, _)| ascii_case_eq(n, name))
        .and_then(|&(_, hex)| d(hex))
}

#[inline(always)]
fn f(args: &[u8]) -> Option<([&[u8]; 4], usize)> {
    let mut parts = [&[][..]; 4];
    let mut count = 0;
    let mut start = 0;
    let mut in_arg = false;
    let len = args.len();

    for i in 0..len {
        let b = args[i];
        let is_sep = b == b' ' || b == b'\t' || b == b',';

        if !is_sep && !in_arg {
            start = i;
            in_arg = true;
        } else if is_sep && in_arg {
            if count >= 4 {
                return None;
            }
            parts[count] = &args[start..i];
            count += 1;
            in_arg = false;
        }
    }

    if in_arg {
        if count >= 4 {
            return None;
        }
        parts[count] = &args[start..];
        count += 1;
    }

    if !(2..=4).contains(&count) {
        return None;
    }
    Some((parts, count))
}

#[inline(always)]
fn g(s: &[u8]) -> Option<C> {
    let len = s.len();

    if s[0] == b'#' {
        return d(s);
    }

    if s[3] != (b'(') && s[4] != (b'(') {
        return e(s);
    }

    if s[len - 1] != b')' {
        return None;
    }

    let (func_type, has_alpha, start) = match &s[..3] {
        prefix if ascii_case_eq(b"rgb", prefix) => {
            if s[3] == (b'(') {
                (0, false, 4)
            } else if s[3] == (b'a') && s[4] == (b'(') {
                (0, true, 5)
            } else {
                return None;
            }
        }
        prefix if ascii_case_eq(b"hsl", prefix) => {
            if s[3] == (b'(') {
                (1, false, 4)
            } else if s[3] == (b'a') && s[4] == (b'(') {
                (1, true, 5)
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let (parts, count) = f(&s[start..len - 1])?;

    if has_alpha && count == 2 {
        let name = parts[0];
        if let Some(base) = e(name) {
            let alpha = z(parts[1])?;
            return Some(C { a: alpha, ..base });
        }
        return None;
    }

    let expected_parts = if has_alpha { 4 } else { 3 };
    if count != expected_parts {
        return None;
    }

    let alpha = if has_alpha { z(parts[3])? } else { 255 };

    match func_type {
        0 => {
            let r = x(parts[0])?;
            let g = x(parts[1])?;
            let b = x(parts[2])?;
            Some(C { r, g, b, a: alpha })
        }
        _ => {
            let h_str = std::str::from_utf8(parts[0]).ok()?;
            let h = m(h_str)?;
            let s = y(parts[1])?;
            let l = y(parts[2])?;
            let (r, g, b) = c(h, s, l);
            Some(C { r, g, b, a: alpha })
        }
    }
}

#[inline(always)]
const fn h(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) * 0x11 == r
        && (g & 0x0F) * 0x11 == g
        && (b & 0x0F) * 0x11 == b
        && (a & 0x0F) * 0x11 == a
}

static N: LazyLock<Vec<(u32, &'static [u8])>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for &(n, h) in K {
        if let Some(c) = d(h) {
            if c.a == 255 {
                let rgb = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
                m.entry(rgb).or_insert(n);
            }
        }
    }
    let mut vec: Vec<_> = m.into_iter().collect();
    vec.sort_unstable_by_key(|&(rgb, _)| rgb);
    vec
});

#[inline(always)]
fn i(rgb: u32) -> Option<&'static [u8]> {
    N.binary_search_by_key(&rgb, |&(r, _)| r)
        .ok()
        .map(|i| N[i].1)
}

#[inline(always)]
fn j(c: &C) -> String {
    if c.a != 255 {
        let short = h(c.r, c.g, c.b, c.a);
        let mut buf = [0u8; 9];
        let s = if short {
            buf[0] = b'#';
            buf[1] = hex_digit(c.r >> 4);
            buf[2] = hex_digit(c.g >> 4);
            buf[3] = hex_digit(c.b >> 4);
            buf[4] = hex_digit(c.a >> 4);
            std::str::from_utf8(&buf[..5]).unwrap()
        } else {
            buf[0] = b'#';
            buf[1] = hex_digit(c.r >> 4);
            buf[2] = hex_digit(c.r & 0xF);
            buf[3] = hex_digit(c.g >> 4);
            buf[4] = hex_digit(c.g & 0xF);
            buf[5] = hex_digit(c.b >> 4);
            buf[6] = hex_digit(c.b & 0xF);
            buf[7] = hex_digit(c.a >> 4);
            buf[8] = hex_digit(c.a & 0xF);
            std::str::from_utf8(&buf[..9]).unwrap()
        };
        return s.to_string();
    }

    let x = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);

    if let Some(q) = i(x) {
        let is_short = h(c.r, c.g, c.b, 255);
        let max_len = if is_short { 4 } else { 7 };
        if q.len() < max_len {
            return std::str::from_utf8(q).unwrap().to_string();
        }
    }

    let short = h(c.r, c.g, c.b, 255);
    if short {
        let mut buf = [0u8; 4];
        buf[0] = b'#';
        buf[1] = hex_digit(c.r >> 4);
        buf[2] = hex_digit(c.g >> 4);
        buf[3] = hex_digit(c.b >> 4);
        std::str::from_utf8(&buf).unwrap().to_string()
    } else {
        let mut buf = [0u8; 7];
        buf[0] = b'#';
        buf[1] = hex_digit(c.r >> 4);
        buf[2] = hex_digit(c.r & 0xF);
        buf[3] = hex_digit(c.g >> 4);
        buf[4] = hex_digit(c.g & 0xF);
        buf[5] = hex_digit(c.b >> 4);
        buf[6] = hex_digit(c.b & 0xF);
        std::str::from_utf8(&buf).unwrap().to_string()
    }
}

#[inline(always)]
fn hex_digit(n: u8) -> u8 {
    match n {
        0..=9 => b'0' + n,
        10..=15 => b'a' + (n - 10),
        _ => b'0',
    }
}


#[inline(always)]
fn trim_whitespace(s: &[u8]) -> &[u8] {
    let start = s
        .iter()
        .position(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .unwrap_or(s.len());
    let end = s
        .iter()
        .rposition(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .map(|i| i + 1)
        .unwrap_or(0);
    &s[start..end]
}

pub fn shorten_css_color(i: impl AsRef<str>) -> String {
    let s = i.as_ref().as_bytes();
    if s.is_empty() {
        return String::new();
    }
    let t = trim_whitespace(s);
    let len = t.len();

    if len < 5 {
        return if ascii_case_eq(t, b"#f00") {
            "red".to_string()
        } else {
            std::str::from_utf8(t).unwrap().to_ascii_lowercase()
        };
    }

    match g(t) {
        Some(c) => j(&c),
        None => std::str::from_utf8(t).unwrap().to_ascii_lowercase(),
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

