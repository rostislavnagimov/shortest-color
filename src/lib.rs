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
const fn h(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'A'..=b'F' => b - b'A' + 10,
        b'a'..=b'f' => b - b'a' + 10,
        _ => 255,
    }
}

#[inline(always)]
const fn p(b: &[u8], i: usize) -> Option<u8> {
    let h1 = h(b[i]);
    let l = h(b[i + 1]);
    if h1 == 255 || l == 255 {
        None
    } else {
        Some((h1 << 4) | l)
    }
}

#[inline(always)]
const fn sg(b: u8) -> Option<u8> {
    let v = h(b);
    if v == 255 {
        None
    } else {
        Some((v << 4) | v)
    }
}

#[inline(always)]
fn n(b: &[u8], m: f32, a: bool) -> Option<f32> {
    let l = b.len();
    if l == 0 || l > 8 {
        return None;
    }

    let mut r = 0.0f32;
    let mut d = false;
    let mut dv = 10.0f32;
    let ng = a && b[0] == b'-';
    let st = if ng {
        if l == 1 {
            return None;
        }
        1
    } else {
        if !a && b[0] == b'-' {
            return None;
        }
        0
    };

    for &c in &b[st..] {
        match c {
            b'0'..=b'9' => {
                let dt = (c - b'0') as f32;
                if d {
                    r += dt / dv;
                    dv *= 10.0;
                    if dv > 10000.0 {
                        break;
                    }
                } else {
                    r = r * 10.0 + dt;
                    if r > m && !ng {
                        return None;
                    }
                }
            }
            b'.' => {
                if d {
                    return None;
                }
                d = true;
            }
            _ => return None,
        }
    }

    let f = if ng { -r } else { r };
    if f > m || (!a && f < 0.0) {
        None
    } else {
        Some(f)
    }
}

#[inline(always)]
fn rt(b: &[u8]) -> Option<u8> {
    let l = b.len();
    if l == 0 || l > 6 {
        return None;
    }

    if !b.contains(&b'.') {
        let mut rs = 0u32;
        for &c in b {
            if !c.is_ascii_digit() {
                return None;
            }
            rs = rs * 10 + (c - b'0') as u32;
            if rs > 255 {
                return None;
            }
        }
        return Some(rs as u8);
    }

    let rt = n(b, 255.9, false)?;
    Some((rt + 0.5) as u8)
}

#[inline(always)]
fn pr(b: &[u8]) -> Option<f32> {
    let l = b.len();
    if !(2..=6).contains(&l) || b[l - 1] != b'%' {
        return None;
    }

    let np = &b[..l - 1];
    let rs = n(np, 100.0, true)?;
    if !(0.0..=100.0).contains(&rs) {
        return None;
    }
    Some(rs)
}

#[inline(always)]
fn al(b: &[u8]) -> Option<u8> {
    if b.is_empty() {
        return None;
    }

    if b[b.len() - 1] == b'%' {
        let pc = pr(b)?;
        return Some((pc * 2.55 + 0.5) as u8);
    }

    let rs = n(b, 1.0, false)?;
    Some((rs * 255.0 + 0.5) as u8)
}

#[inline(always)]
fn ew(sl: &[u8], sf: &[u8]) -> bool {
    let sl_l = sl.len();
    let sf_l = sf.len();
    sl_l >= sf_l && sl[sl_l - sf_l..].eq_ignore_ascii_case(sf)
}

#[inline(always)]
fn an(s: &str) -> Option<f32> {
    let b = s.as_bytes();
    let l = b.len();
    if l == 0 {
        return None;
    }

    let (ns, mu) = if ew(b, b"grad") {
        (&s[..l - 4], 0.9)
    } else if ew(b, b"turn") {
        (&s[..l - 4], 360.0)
    } else if ew(b, b"deg") {
        (&s[..l - 3], 1.0)
    } else if ew(b, b"rad") {
        (&s[..l - 3], 57.29578)
    } else {
        (s, 1.0)
    };

    let hu = n(ns.as_bytes(), f32::MAX, true)?;
    Some(((hu * mu % 360.0) + 360.0) % 360.0)
}

#[inline(always)]
fn hs(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let sn = s * 0.01;
    let ln = l * 0.01;
    let ch = (1.0 - (2.0 * ln - 1.0).abs()) * sn;
    let hc = h / 60.0;
    let x = ch * (1.0 - ((hc % 2.0) - 1.0).abs());
    let m = ln - ch * 0.5;

    let (r, g, b) = match hc as u8 {
        0 => (ch, x, 0.0),
        1 => (x, ch, 0.0),
        2 => (0.0, ch, x),
        3 => (0.0, x, ch),
        4 => (x, 0.0, ch),
        _ => (ch, 0.0, x),
    };

    (
        ((r + m) * 255.0 + 0.5) as u8,
        ((g + m) * 255.0 + 0.5) as u8,
        ((b + m) * 255.0 + 0.5) as u8,
    )
}

#[inline(always)]
fn hc(s: &[u8]) -> Option<C> {
    let hp = &s[1..];
    match hp.len() {
        3 => Some(C {
            r: sg(hp[0])?,
            g: sg(hp[1])?,
            b: sg(hp[2])?,
            a: 255,
        }),
        4 => Some(C {
            r: sg(hp[0])?,
            g: sg(hp[1])?,
            b: sg(hp[2])?,
            a: sg(hp[3])?,
        }),
        6 => Some(C {
            r: p(hp, 0)?,
            g: p(hp, 2)?,
            b: p(hp, 4)?,
            a: 255,
        }),
        8 => Some(C {
            r: p(hp, 0)?,
            g: p(hp, 2)?,
            b: p(hp, 4)?,
            a: p(hp, 6)?,
        }),
        _ => None,
    }
}

static NAME_MAP: LazyLock<HashMap<Vec<u8>, C>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for &(nm, hx) in K {
        if let Some(c) = hc(hx) {
            let lower_name = nm.iter().map(|&b| b | 0x20).collect();
            map.insert(lower_name, c);
        }
    }
    map
});

#[inline(always)]
fn lk(nm: &[u8]) -> Option<C> {
    let lower_nm: Vec<u8> = nm.iter().map(|&b| b | 0x20).collect();
    NAME_MAP.get(&lower_nm).copied()
}

#[inline(always)]
fn ar(ag: &[u8]) -> Option<([&[u8]; 4], usize)> {
    let mut pt = [&[][..]; 4];
    let mut c = 0;
    let mut st = 0;
    let mut ia = false;

    for i in 0..ag.len() {
        let b = ag[i];
        let is = matches!(b, b' ' | b'\t' | b',');

        if !is && !ia {
            st = i;
            ia = true;
        } else if is && ia {
            if c >= 4 {
                return None;
            }
            pt[c] = &ag[st..i];
            c += 1;
            ia = false;
        }
    }

    if ia {
        if c >= 4 {
            return None;
        }
        pt[c] = &ag[st..];
        c += 1;
    }

    if (2..=4).contains(&c) {
        Some((pt, c))
    } else {
        None
    }
}

#[inline(always)]
fn pc(s: &[u8]) -> Option<C> {
    let l = s.len();

    if s[0] == b'#' {
        return hc(s);
    }

    if s[3] != b'(' && s[4] != b'(' {
        return lk(s);
    }

    if s[l - 1] != b')' {
        return None;
    }

    let (ft, ha, st) = match &s[..3] {
        px if px.eq_ignore_ascii_case(b"rgb") => match s[3] {
            b'(' => (0, false, 4),
            b'a' => (0, true, 5),
            _ => return None,
        },
        px if px.eq_ignore_ascii_case(b"hsl") => match s[3] {
            b'(' => (1, false, 4),
            b'a' => (1, true, 5),
            _ => return None,
        },
        _ => return None,
    };

    let (pt, c) = ar(&s[st..l - 1])?;

    if ha && c == 2 {
        let nm = pt[0];
        if let Some(bc) = lk(nm) {
            let a = al(pt[1])?;
            return Some(C { a, ..bc });
        }
        return None;
    }

    let ep = if ha { 4 } else { 3 };
    if c != ep {
        return None;
    }

    let a = if ha { al(pt[3])? } else { 255 };

    match ft {
        0 => {
            let rg = rt(pt[0])?;
            let g = rt(pt[1])?;
            let b = rt(pt[2])?;
            Some(C { r: rg, g, b, a })
        }
        _ => {
            let hs_str = std::str::from_utf8(pt[0]).ok()?;
            let hu = an(hs_str)?;
            let sa = pr(pt[1])?;
            let li = pr(pt[2])?;
            let (r, g, b) = hs(hu, sa, li);
            Some(C { r, g, b, a })
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

static L: LazyLock<Vec<(u32, &'static [u8])>> = LazyLock::new(|| {
    let v: Vec<_> = K
        .iter()
        .filter_map(|&(nm, hx)| {
            hc(hx).and_then(|cl| {
                if cl.a == 255 {
                    let rg = ((cl.r as u32) << 16) | ((cl.g as u32) << 8) | (cl.b as u32);
                    Some((rg, nm))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut m = HashMap::new();
    for (rg, nm) in v {
        m.entry(rg).or_insert(nm);
    }

    let mut result: Vec<_> = m.into_iter().collect();
    result.sort_unstable_by_key(|&(rg, _)| rg);
    result
});

#[inline(always)]
fn fn1(rg: u32) -> Option<&'static [u8]> {
    L.binary_search_by_key(&rg, |&(r, _)| r)
        .ok()
        .map(|i| L[i].1)
}

#[inline(always)]
const fn hd(n: u8) -> u8 {
    match n {
        0..=9 => b'0' + n,
        _ => b'a' + (n - 10),
    }
}

#[inline(always)]
fn cs(cl: &C) -> String {
    if cl.a != 255 {
        let sh1 = sh(cl.r, cl.g, cl.b, cl.a);
        let mut bf = [0u8; 9];
        let sl = if sh1 {
            bf[0] = b'#';
            bf[1] = hd(cl.r >> 4);
            bf[2] = hd(cl.g >> 4);
            bf[3] = hd(cl.b >> 4);
            bf[4] = hd(cl.a >> 4);
            &bf[..5]
        } else {
            bf[0] = b'#';
            bf[1] = hd(cl.r >> 4);
            bf[2] = hd(cl.r & 0xF);
            bf[3] = hd(cl.g >> 4);
            bf[4] = hd(cl.g & 0xF);
            bf[5] = hd(cl.b >> 4);
            bf[6] = hd(cl.b & 0xF);
            bf[7] = hd(cl.a >> 4);
            bf[8] = hd(cl.a & 0xF);
            &bf[..9]
        };
        return unsafe { std::str::from_utf8_unchecked(sl) }.to_string();
    }

    let rg = ((cl.r as u32) << 16) | ((cl.g as u32) << 8) | (cl.b as u32);

    if let Some(nm) = fn1(rg) {
        let sh1 = sh(cl.r, cl.g, cl.b, 255);
        let ml = if sh1 { 4 } else { 7 };
        if nm.len() < ml {
            return unsafe { std::str::from_utf8_unchecked(nm) }.to_string();
        }
    }

    let sh1 = sh(cl.r, cl.g, cl.b, 255);
    let mut bf = [0u8; 7];
    let sl = if sh1 {
        bf[0] = b'#';
        bf[1] = hd(cl.r >> 4);
        bf[2] = hd(cl.g >> 4);
        bf[3] = hd(cl.b >> 4);
        &bf[..4]
    } else {
        bf[0] = b'#';
        bf[1] = hd(cl.r >> 4);
        bf[2] = hd(cl.r & 0xF);
        bf[3] = hd(cl.g >> 4);
        bf[4] = hd(cl.g & 0xF);
        bf[5] = hd(cl.b >> 4);
        bf[6] = hd(cl.b & 0xF);
        &bf[..7]
    };
    unsafe { std::str::from_utf8_unchecked(sl) }.to_string()
}

#[inline(always)]
fn tw(s: &[u8]) -> &[u8] {
    let st = s
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .unwrap_or(s.len());
    let ed = s
        .iter()
        .rposition(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .map_or(st, |i| i + 1);
    &s[st..ed]
}

#[inline(always)]
fn to_lower_fast(s: &[u8]) -> String {
    let mut result = String::with_capacity(s.len());
    unsafe {
        let bytes = result.as_mut_vec();
        bytes.extend(s.iter().map(|&b| b | 0x20));
    }
    result
}

pub fn shorten_css_color(i: impl AsRef<str>) -> String {
    let s = i.as_ref().as_bytes();
    if s.is_empty() {
        return String::new();
    }

    let tr = tw(s);

    if tr.len() < 5 {
        if tr.eq_ignore_ascii_case(b"#f00") {
            return String::from("red");
        }
        return to_lower_fast(tr);
    }

    match pc(tr) {
        Some(cl) => cs(&cl),
        None => to_lower_fast(tr),
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
