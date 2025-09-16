use std::collections::HashMap;
use std::sync::LazyLock;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

static H: [u8; 256] = {
    const I: u8 = 255;
    let mut t = [I; 256];
    let mut i = 0;
    while i < 10 {
        t[(b'0' + i) as usize] = i;
        i += 1;
    }
    let mut i = 0;
    while i < 6 {
        t[(b'A' + i) as usize] = 10 + i;
        t[(b'a' + i) as usize] = 10 + i;
        i += 1;
    }
    t
};

#[inline(always)]
const fn h2(b: &[u8], i: usize) -> Option<u8> {
    let h = H[b[i] as usize];
    let l = H[b[i + 1] as usize];
    if h == 255 || l == 255 {
        None
    } else {
        Some((h << 4) | l)
    }
}

#[inline(always)]
const fn h1(b: u8) -> Option<u8> {
    let v = H[b as usize];
    if v == 255 {
        None
    } else {
        Some((v << 4) | v)
    }
}

fn pf(b: &[u8], m: f32, n: bool) -> Option<f32> {
    if b.is_empty() || b.len() > 8 {
        return None;
    }

    let mut r = 0.0f32;
    let mut d = 0u32;
    let mut p = 0;
    let neg = n && b[0] == b'-';

    if neg {
        p = 1;
        if b.len() == 1 {
            return None;
        }
    } else if !n && b[0] == b'-' {
        return None;
    }

    for &c in &b[p..] {
        match c {
            b'0'..=b'9' => {
                if d > 0 {
                    d += 1;
                    if d > 4 {
                        break;
                    }
                    r += (c - b'0') as f32 / (10u32.pow(d - 1)) as f32;
                } else {
                    r = r * 10.0 + (c - b'0') as f32;
                    if r > m && !neg {
                        return None;
                    }
                }
            }
            b'.' => {
                if d > 0 {
                    return None;
                }
                d = 1;
            }
            _ => return None,
        }
    }

    let res = if neg { -r } else { r };
    if res > m || (!n && res < 0.0) {
        return None;
    }
    Some(res)
}

fn rgb(b: &[u8]) -> Option<u8> {
    if b.is_empty() || b.len() > 6 {
        return None;
    }

    let dp = b.iter().position(|&x| x == b'.');

    if dp.is_none() {
        let mut r = 0u16;
        for &c in b {
            if !c.is_ascii_digit() {
                return None;
            }
            r = r * 10 + (c - b'0') as u16;
            if r > 255 {
                return None;
            }
        }
        return Some(r as u8);
    }

    let r = pf(b, 255.9, false)?;
    Some((r + 0.5) as u8)
}

fn pct(b: &[u8]) -> Option<f32> {
    if b.len() < 2 || b.len() > 6 || b[b.len() - 1] != b'%' {
        return None;
    }

    let n = &b[..b.len() - 1];
    let r = pf(n, 100.0, true)?;
    if !(0.0..=100.0).contains(&r) {
        return None;
    }
    Some(r)
}

fn a(b: &[u8]) -> Option<u8> {
    if b.is_empty() {
        return None;
    }

    if b[b.len() - 1] == b'%' {
        let p = pct(b)?;
        return Some((p * 2.55 + 0.5) as u8);
    }

    let r = pf(b, 1.0, false)?;
    Some((r * 255.0 + 0.5) as u8)
}

fn hue(s: &str) -> Option<f32> {
    let b = s.as_bytes();
    let l = b.len();

    if l == 0 {
        return None;
    }

    let (np, mul) = match (l >= 4, l >= 3) {
        (true, _) if b.ends_with(b"grad") => (&s[..l - 4], 0.9),
        (true, _) if b.ends_with(b"turn") => (&s[..l - 4], 360.0),
        (_, true) if b.ends_with(b"deg") => (&s[..l - 3], 1.0),
        (_, true) if b.ends_with(b"rad") => (&s[..l - 3], 57.295_78),
        _ => (s, 1.0),
    };

    let h = pf(np.as_bytes(), f32::MAX, true)?;
    Some(((h * mul % 360.0) + 360.0) % 360.0)
}

fn h2r(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hs = h * (1.0 / 60.0);
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

static KW: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(K.len());
    for &(n, h) in K {
        if let Some(c) = ph(h) {
            m.insert(n, c);
        }
    }
    m
});

fn ph(s: &str) -> Option<Color> {
    let b = s.as_bytes();
    if b.is_empty() || b[0] != b'#' {
        return None;
    }
    let h = &b[1..];
    match h.len() {
        3 => Some(Color {
            r: h1(h[0])?,
            g: h1(h[1])?,
            b: h1(h[2])?,
            a: 255,
        }),
        4 => Some(Color {
            r: h1(h[0])?,
            g: h1(h[1])?,
            b: h1(h[2])?,
            a: h1(h[3])?,
        }),
        6 => Some(Color {
            r: h2(h, 0)?,
            g: h2(h, 2)?,
            b: h2(h, 4)?,
            a: 255,
        }),
        8 => Some(Color {
            r: h2(h, 0)?,
            g: h2(h, 2)?,
            b: h2(h, 4)?,
            a: h2(h, 6)?,
        }),
        _ => None,
    }
}

fn kw(n: &str) -> Option<Color> {
    KW.get(n).copied()
}

fn parse_func_args(args: &[u8]) -> Option<([&[u8]; 4], usize)> {
    let mut parts = [&[][..]; 4];
    let mut count = 0;
    let mut start = 0;
    let mut in_space = true;
    
    for (i, &b) in args.iter().enumerate() {
        match b {
            b' ' | b'\t' => {
                if !in_space && start < i {
                    if count >= 4 { return None; }
                    parts[count] = &args[start..i];
                    count += 1;
                }
                in_space = true;
            }
            b',' => {
                if !in_space && start < i {
                    if count >= 4 { return None; }
                    parts[count] = &args[start..i];
                    count += 1;
                }
                in_space = true;
            }
            _ => {
                if in_space {
                    start = i;
                    in_space = false;
                }
            }
        }
    }
    
    if !in_space && start < args.len() {
        if count >= 4 { return None; }
        parts[count] = &args[start..];
        count += 1;
    }
    
    if count < 2 || count > 4 { return None; }
    Some((parts, count))
}

fn p(s: &str) -> Option<Color> {
    let b = s.as_bytes();
    let l = b.len();

    if b[0] == b'#' {
        return ph(s);
    }

    if !b.contains(&b'(') {
        return kw(s);
    }

    if b[l - 1] != b')' {
        return None;
    }

    let (func_type, has_alpha, start) = match b {
        [b'r', b'g', b'b', b'a', b'(', ..] => (0, true, 5),
        [b'r', b'g', b'b', b'(', ..] => (0, false, 4),
        [b'h', b's', b'l', b'a', b'(', ..] => (1, true, 5),
        [b'h', b's', b'l', b'(', ..] => (1, false, 4),
        _ => return None,
    };

    let args = &b[start..l-1];
    let (parts, count) = parse_func_args(args)?;
    
    if has_alpha && count == 2 {
        if let Some(base) = kw(unsafe { std::str::from_utf8_unchecked(parts[0]) }) {
            let alpha = a(parts[1])?;
            return Some(Color { r: base.r, g: base.g, b: base.b, a: alpha });
        }
        return None;
    }

    let expected_parts = if has_alpha { 4 } else { 3 };
    if count != expected_parts {
        return None;
    }

    match func_type {
        0 => {
            let r = rgb(parts[0])?;
            let g = rgb(parts[1])?;
            let bl = rgb(parts[2])?;
            let alpha = if has_alpha { a(parts[3])? } else { 255 };
            Some(Color { r, g, b: bl, a: alpha })
        }
        _ => {
            let h = hue(unsafe { std::str::from_utf8_unchecked(parts[0]) })?;
            let s = pct(parts[1])?;
            let l = pct(parts[2])?;
            let (r, g, bl) = h2r(h, s, l);
            let alpha = if has_alpha { a(parts[3])? } else { 255 };
            Some(Color { r, g, b: bl, a: alpha })
        }
    }
}

#[inline(always)]
const fn sh(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) * 0x11 == r
        && (g & 0x0F) * 0x11 == g
        && (b & 0x0F) * 0x11 == b
        && (a & 0x0F) * 0x11 == a
}

static N: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(K.len());
    for &(n, h) in K {
        if let Some(c) = ph(h) {
            if c.a == 255 {
                let rgb = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
                m.entry(rgb).or_insert(n);
            }
        }
    }
    m
});

fn shr(c: &Color) -> String {
    if c.a != 255 {
        return if sh(c.r, c.g, c.b, c.a) {
            format!("#{:x}{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4, c.a >> 4)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
        };
    }

    let rgb = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);

    if let Some(&name) = N.get(&rgb) {
        let is_short = sh(c.r, c.g, c.b, 255);
        if name.len() < if is_short { 4 } else { 7 } {
            return name.to_string();
        }
    }

    if sh(c.r, c.g, c.b, 255) {
        format!("#{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4)
    } else {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    }
}

pub fn shorten_css_color(s: &str) -> String {
    let t = s.trim().to_ascii_lowercase();

    if t.len() < 5 {
        return if t == "#f00" { "red".to_string() } else { t };
    }

    match p(&t) {
        Some(c) => shr(&c),
        None => s.to_string(),
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