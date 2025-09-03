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

static HEX_TABLE: [u8; 256] = {
    const INVALID: u8 = 255;
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < 10 {
        table[(b'0' + i) as usize] = i;
        i += 1;
    }
    let mut i = 0;
    while i < 6 {
        table[(b'A' + i) as usize] = 10 + i;
        table[(b'a' + i) as usize] = 10 + i;
        i += 1;
    }
    table
};

#[inline(always)]
const fn hex2(b: &[u8], i: usize) -> Option<u8> {
    let h = HEX_TABLE[b[i] as usize];
    let l = HEX_TABLE[b[i + 1] as usize];
    if h == 255 || l == 255 {
        None
    } else {
        Some((h << 4) | l)
    }
}

#[inline(always)]
const fn hex1(b: u8) -> Option<u8> {
    let v = HEX_TABLE[b as usize];
    if v == 255 {
        None
    } else {
        Some((v << 4) | v)
    }
}

fn parse_float_with_limits(b: &[u8], max_val: f32, allow_negative: bool) -> Option<f32> {
    if b.is_empty() || b.len() > 8 {
        return None;
    }

    let mut r = 0.0f32;
    let mut d = 0u32;
    let mut p = 0;
    let neg = allow_negative && b[0] == b'-';

    if neg {
        p = 1;
        if b.len() == 1 {
            return None;
        }
    } else if !allow_negative && b[0] == b'-' {
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
                    if r > max_val && !neg {
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

    let result = if neg { -r } else { r };
    if result > max_val || (!allow_negative && result < 0.0) {
        return None;
    }
    Some(result)
}

fn rgb(b: &[u8]) -> Option<u8> {
    if b.is_empty() || b.len() > 6 {
        return None;
    }

    let dot_pos = b.iter().position(|&x| x == b'.');

    if dot_pos.is_none() {
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

    let r = parse_float_with_limits(b, 255.9, false)?;
    Some((r + 0.5) as u8)
}

fn pct(b: &[u8]) -> Option<f32> {
    if b.len() < 2 || b.len() > 6 || b[b.len() - 1] != b'%' {
        return None;
    }

    let n = &b[..b.len() - 1];
    let r = parse_float_with_limits(n, 100.0, true)?;
    if !(0.0..=100.0).contains(&r) {
        return None;
    }
    Some(r)
}

fn alpha(b: &[u8]) -> Option<u8> {
    if b.is_empty() {
        return None;
    }

    if b[b.len() - 1] == b'%' {
        let p = pct(b)?;
        return Some((p * 2.55 + 0.5) as u8);
    }

    let r = parse_float_with_limits(b, 1.0, false)?;
    Some((r * 255.0 + 0.5) as u8)
}

fn hue(s: &str) -> Option<f32> {
    let b = s.as_bytes();
    let l = b.len();

    if l == 0 {
        return None;
    }

    let (num_part, multiplier) = match (l >= 4, l >= 3) {
        (true, _) if b.ends_with(b"grad") => (&s[..l - 4], 0.9),
        (true, _) if b.ends_with(b"turn") => (&s[..l - 4], 360.0),
        (_, true) if b.ends_with(b"deg") => (&s[..l - 3], 1.0),
        (_, true) if b.ends_with(b"rad") => (&s[..l - 3], 57.295_78),
        _ => (s, 1.0),
    };

    let h = parse_float_with_limits(num_part.as_bytes(), f32::MAX, true)?;
    Some(((h * multiplier % 360.0) + 360.0) % 360.0)
}

fn hsl2rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s * 0.01;
    let l = l * 0.01;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_s = h * (1.0 / 60.0);
    let x = c * (1.0 - ((h_s % 2.0) - 1.0).abs());
    let m = l - c * 0.5;

    let (r, g, b) = match h_s as u8 {
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

fn split(s: &str) -> [&str; 4] {
    let mut parts = [""; 4];

    for (i, part) in s
        .split(&[',', ' '] as &[char])
        .filter_map(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .take(4)
        .enumerate()
    {
        parts[i] = part;
    }
    parts
}

static KW: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(KEYWORDS.len());
    for &(n, h) in KEYWORDS {
        if let Some(c) = parse_hex_color(h) {
            m.insert(n, c);
        }
    }
    m
});

fn parse_hex_color(s: &str) -> Option<Color> {
    let b = s.as_bytes();
    if b.is_empty() || b[0] != b'#' {
        return None;
    }
    let h = &b[1..];
    match h.len() {
        3 => Some(Color {
            r: hex1(h[0])?,
            g: hex1(h[1])?,
            b: hex1(h[2])?,
            a: 255,
        }),
        4 => Some(Color {
            r: hex1(h[0])?,
            g: hex1(h[1])?,
            b: hex1(h[2])?,
            a: hex1(h[3])?,
        }),
        6 => Some(Color {
            r: hex2(h, 0)?,
            g: hex2(h, 2)?,
            b: hex2(h, 4)?,
            a: 255,
        }),
        8 => Some(Color {
            r: hex2(h, 0)?,
            g: hex2(h, 2)?,
            b: hex2(h, 4)?,
            a: hex2(h, 6)?,
        }),
        _ => None,
    }
}

#[inline(always)]
fn kw(n: &str) -> Option<Color> {
    if n.len() > 20 {
        return None;
    }
    KW.get(n).copied()
}

fn parse(s: &str) -> Option<Color> {
    let b = s.as_bytes();
    let l = b.len();

    if l == 0 {
        return None;
    }

    if b[0] == b'#' {
        return parse_hex_color(s);
    }

    if l <= 20 && !b.contains(&b'(') {
        return kw(s);
    }

    if b[l - 1] != b')' {
        return kw(s);
    }

    let (f, a, cs, ce) = match b {
        [b'r', b'g', b'b', b'a', b'(', ..] if l >= 6 => ("rgb", true, 5, l - 1),
        [b'r', b'g', b'b', b'(', ..] => ("rgb", false, 4, l - 1),
        [b'h', b's', b'l', b'a', b'(', ..] if l >= 6 => ("hsl", true, 5, l - 1),
        [b'h', b's', b'l', b'(', ..] => ("hsl", false, 4, l - 1),
        _ => return kw(s),
    };

    let c = std::str::from_utf8(&b[cs..ce]).ok()?;
    let pt = split(c);

    if f == "rgb" {
        if a {
            if !pt[1].is_empty() && pt[2].is_empty() {
                if let Some(base) = kw(pt[0]) {
                    let al = alpha(pt[1].as_bytes())?;
                    return Some(Color {
                        r: base.r,
                        g: base.g,
                        b: base.b,
                        a: al,
                    });
                }
            }
            if pt[3].is_empty() {
                return None;
            }

            let r = rgb(pt[0].as_bytes())?;
            let g = rgb(pt[1].as_bytes())?;
            let bl = rgb(pt[2].as_bytes())?;
            let al = alpha(pt[3].as_bytes())?;
            Some(Color { r, g, b: bl, a: al })
        } else {
            if pt[2].is_empty() || !pt[3].is_empty() {
                return None;
            }
            let r = rgb(pt[0].as_bytes())?;
            let g = rgb(pt[1].as_bytes())?;
            let bl = rgb(pt[2].as_bytes())?;
            Some(Color {
                r,
                g,
                b: bl,
                a: 255,
            })
        }
    } else if a {
        if !pt[1].is_empty() && pt[2].is_empty() {
            if let Some(base) = kw(pt[0]) {
                let al = alpha(pt[1].as_bytes())?;
                return Some(Color {
                    r: base.r,
                    g: base.g,
                    b: base.b,
                    a: al,
                });
            }
        }
        if pt[3].is_empty() {
            return None;
        }

        let h = hue(pt[0])?;
        let sa = pct(pt[1].as_bytes())?;
        let li = pct(pt[2].as_bytes())?;
        let (r, g, bl) = hsl2rgb(h, sa, li);
        let al = alpha(pt[3].as_bytes())?;
        Some(Color { r, g, b: bl, a: al })
    } else {
        if pt[2].is_empty() || !pt[3].is_empty() {
            return None;
        }
        let h = hue(pt[0])?;
        let sa = pct(pt[1].as_bytes())?;
        let li = pct(pt[2].as_bytes())?;
        let (r, g, bl) = hsl2rgb(h, sa, li);
        Some(Color {
            r,
            g,
            b: bl,
            a: 255,
        })
    }
}

#[inline(always)]
const fn short(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) * 0x11 == r
        && (g & 0x0F) * 0x11 == g
        && (b & 0x0F) * 0x11 == b
        && (a & 0x0F) * 0x11 == a
}

static NAMES: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(KEYWORDS.len());
    for &(n, h) in KEYWORDS {
        if let Some(c) = parse_hex_color(h) {
            if c.a == 255 && n.len() <= 7 {
                let rgb = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
                m.entry(rgb).or_insert(n);
            }
        }
    }
    m
});

fn shorten(c: &Color) -> String {
    if c.a != 255 {
        return if short(c.r, c.g, c.b, c.a) {
            format!("#{:x}{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4, c.a >> 4)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
        };
    }

    let rgb = ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);

    if let Some(&name) = NAMES.get(&rgb) {
        let hex_len = if short(c.r, c.g, c.b, 255) { 4 } else { 7 };
        if name.len() < hex_len {
            return name.to_string();
        }
    }

    if short(c.r, c.g, c.b, 255) {
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

    match parse(&t) {
        Some(c) => shorten(&c),
        None => s.to_string(),
    }
}

const KEYWORDS: &[(&str, &str)] = &[
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
