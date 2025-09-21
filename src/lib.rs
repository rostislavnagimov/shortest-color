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
    slice_len >= suffix_len && slice[slice_len - suffix_len..].eq_ignore_ascii_case(suffix)
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
fn d(s: &str) -> Option<C> {
    let b = s.as_bytes();
    let len = b.len();
    if len == 0 || b[0] != b'#' {
        return None;
    }
    let h = &b[1..];
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
fn e(name: &str) -> Option<C> {
    K.iter()
        .find(|&&(n, _)| n.eq_ignore_ascii_case(name))
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
fn g(s: &str) -> Option<C> {
    let b = s.as_bytes();
    let len = b.len();

    if b[0] == b'#' {
        return d(s);
    }

    if b[3] != (b'(') && b[4] != (b'(') {
        return e(s);
    }

    if b[len - 1] != b')' {
        return None;
    }

    let (func_type, has_alpha, start) = match &b[..3] {
        prefix if prefix.eq_ignore_ascii_case(b"rgb") => {
            if b[3] == (b'(') {
                (0, false, 4)
            } else if b[3] == (b'a') && b[4] == (b'(') {
                (0, true, 5)
            } else {
                return None;
            }
        }
        prefix if prefix.eq_ignore_ascii_case(b"hsl") => {
            if b[3] == (b'(') {
                (1, false, 4)
            } else if b[3] == (b'a') && b[4] == (b'(') {
                (1, true, 5)
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let (parts, count) = f(&b[start..len - 1])?;

    if has_alpha && count == 2 {
        let name = std::str::from_utf8(parts[0]).ok()?;
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

static N: LazyLock<Vec<(u32, &'static str)>> = LazyLock::new(|| {
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
fn i(rgb: u32) -> Option<&'static str> {
    N.binary_search_by_key(&rgb, |&(r, _)| r)
        .ok()
        .map(|i| N[i].1)
}

#[inline(always)]
fn j(c: &C) -> String {
    if c.a != 255 {
        return if h(c.r, c.g, c.b, c.a) {
            format!("#{:x}{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4, c.a >> 4)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
        };
    }

    let x = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);

    if let Some(q) = i(x) {
        let is_short = h(c.r, c.g, c.b, 255);
        let z = if is_short { 4 } else { 7 };
        if q.len() < z {
            return q.to_string();
        }
    }

    if h(c.r, c.g, c.b, 255) {
        format!("#{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4)
    } else {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    }
}

#[inline(always)]
fn trim_whitespace(s: &str) -> &str {
    let start = s
        .bytes()
        .position(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .unwrap_or(s.len());
    let end = s
        .bytes()
        .rposition(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .map(|i| i + 1)
        .unwrap_or(0);
    &s[start..end]
}

pub fn shorten_css_color(i: impl AsRef<str>) -> String {
    let s = i.as_ref();
    if s.is_empty() {
        return String::new();
    }
    let t = trim_whitespace(s);
    let len = t.len();

    if len < 5 {
        return if t.eq_ignore_ascii_case("#f00") {
            "red".to_string()
        } else {
            t.to_ascii_lowercase()
        };
    }

    match g(t) {
        Some(c) => j(&c),
        None => t.to_ascii_lowercase(),
    }
}

const K: &[(&str, &str)] = &[
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
