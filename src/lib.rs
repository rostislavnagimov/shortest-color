use std::collections::HashMap;
use std::sync::LazyLock;
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct A {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
#[inline(always)]
const fn b(v: u8) -> u8 {
    match v {
        b'0'..=b'9' => v - b'0',
        b'A'..=b'F' => v - b'A' + 10,
        b'a'..=b'f' => v - b'a' + 10,
        _ => 255,
    }
}
#[inline(always)]
const fn c(s: &[u8], i: usize) -> Option<u8> {
    let h = b(s[i]);
    let l = b(s[i + 1]);
    if h == 255 || l == 255 {
        None
    } else {
        Some((h << 4) | l)
    }
}
#[inline(always)]
const fn d(v: u8) -> Option<u8> {
    let x = b(v);
    if x == 255 {
        None
    } else {
        Some((x << 4) | x)
    }
}
#[inline(always)]
fn e(s: &[u8], m: f32, g: bool) -> Option<f32> {
    let l = s.len();
    if l == 0 || l > 8 {
        return None;
    }
    if !g && !s.contains(&b'.') {
        let mut r = 0u32;
        for &v in s {
            if !v.is_ascii_digit() {
                return None;
            }
            r = r * 10 + (v - b'0') as u32;
        }
        let f = r as f32;
        return if f > m { None } else { Some(f) };
    }
    let mut r = 0.0f32;
    let mut di = false;
    let mut dv = 10.0f32;
    let ng = g && s[0] == b'-';
    let st = if ng {
        if l == 1 {
            return None;
        }
        1
    } else {
        if !g && s[0] == b'-' {
            return None;
        }
        0
    };
    for &v in &s[st..] {
        match v {
            b'0'..=b'9' => {
                let dt = (v - b'0') as f32;
                if di {
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
                if di {
                    return None;
                }
                di = true;
            }
            _ => return None,
        }
    }
    let f = if ng { -r } else { r };
    if f > m || (!g && f < 0.0) {
        None
    } else {
        Some(f)
    }
}
#[inline(always)]
fn f(s: &[u8]) -> Option<u8> {
    let l = s.len();
    if l == 0 || l > 6 {
        return None;
    }
    if !s.contains(&b'.') {
        let mut r = 0u32;
        for &v in s {
            if !v.is_ascii_digit() {
                return None;
            }
            r = r * 10 + (v - b'0') as u32;
            if r > 255 {
                return None;
            }
        }
        return Some(r as u8);
    }
    let r = e(s, 255.9, false)?;
    Some((r + 0.5) as u8)
}
#[inline(always)]
fn g(s: &[u8]) -> Option<f32> {
    let l = s.len();
    if !(2..=6).contains(&l) || s[l - 1] != b'%' {
        return None;
    }
    let r = e(&s[..l - 1], 100.0, true)?;
    if !(0.0..=100.0).contains(&r) {
        return None;
    }
    Some(r)
}
#[inline(always)]
fn h(s: &[u8]) -> Option<u8> {
    if s.is_empty() {
        return None;
    }
    if s[s.len() - 1] == b'%' {
        let p = g(s)?;
        return Some((p * 2.55 + 0.5) as u8);
    }
    let r = e(s, 1.0, false)?;
    Some((r * 255.0 + 0.5) as u8)
}
#[inline(always)]
fn i(sl: &[u8], sf: &[u8]) -> bool {
    let a = sl.len();
    let z = sf.len();
    a >= z && sl[a - z..].eq_ignore_ascii_case(sf)
}
#[inline(always)]
fn j(s: &str) -> Option<f32> {
    let v = s.as_bytes();
    let l = v.len();
    if l == 0 {
        return None;
    }
    let (ns, mu) = if i(v, b"grad") {
        (&s[..l - 4], 0.9)
    } else if i(v, b"turn") {
        (&s[..l - 4], 360.0)
    } else if i(v, b"deg") {
        (&s[..l - 3], 1.0)
    } else if i(v, b"rad") {
        (&s[..l - 3], 57.29578)
    } else {
        (s, 1.0)
    };
    let hu = e(ns.as_bytes(), f32::MAX, true)?;
    Some(((hu * mu % 360.0) + 360.0) % 360.0)
}
#[inline(always)]
fn k(hue: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let sn = s * 0.01;
    let ln = l * 0.01;
    let ch = (1.0 - (2.0 * ln - 1.0).abs()) * sn;
    let hc = hue / 60.0;
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
fn l(s: &[u8]) -> Option<A> {
    let p = &s[1..];
    match p.len() {
        3 => Some(A {
            r: d(p[0])?,
            g: d(p[1])?,
            b: d(p[2])?,
            a: 255,
        }),
        4 => Some(A {
            r: d(p[0])?,
            g: d(p[1])?,
            b: d(p[2])?,
            a: d(p[3])?,
        }),
        6 => Some(A {
            r: c(p, 0)?,
            g: c(p, 2)?,
            b: c(p, 4)?,
            a: 255,
        }),
        8 => Some(A {
            r: c(p, 0)?,
            g: c(p, 2)?,
            b: c(p, 4)?,
            a: c(p, 6)?,
        }),
        _ => None,
    }
}
static M: LazyLock<HashMap<Vec<u8>, A>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for &(nm, hx) in K {
        if let Some(cl) = l(hx) {
            let lo: Vec<u8> = nm.iter().map(|&v| v | 0x20).collect();
            map.insert(lo, cl);
        }
    }
    map
});
#[inline(always)]
fn n(s: &[u8]) -> Option<A> {
    let lo: Vec<u8> = s.iter().map(|&v| v | 0x20).collect();
    M.get(&lo).copied()
}
#[inline(always)]
fn o(ag: &[u8]) -> Option<([&[u8]; 4], usize)> {
    let mut pt = [&[][..]; 4];
    let mut ct = 0;
    let mut st = 0;
    for idx in 0..=ag.len() {
        let sp = idx == ag.len() || matches!(ag[idx], b' ' | b'\t' | b',');
        if sp {
            if idx > st {
                if ct >= 4 {
                    return None;
                }
                pt[ct] = &ag[st..idx];
                ct += 1;
            }
            st = idx + 1;
        }
    }
    if (2..=4).contains(&ct) {
        Some((pt, ct))
    } else {
        None
    }
}
#[inline(always)]
fn p(s: &[u8]) -> Option<A> {
    let ln = s.len();
    if s[0] == b'#' {
        return l(s);
    }
    if s[3] != b'(' && s[4] != b'(' {
        return n(s);
    }
    if s[ln - 1] != b')' {
        return None;
    }
    let (fmt, ha, st) = match &s[..3] {
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
    let (pt, ct) = o(&s[st..ln - 1])?;
    if ha && ct == 2 {
        if let Some(bc) = n(pt[0]) {
            let av = h(pt[1])?;
            return Some(A { a: av, ..bc });
        }
        return None;
    }
    let ep = if ha { 4 } else { 3 };
    if ct != ep {
        return None;
    }
    let av = if ha { h(pt[3])? } else { 255 };
    match fmt {
        0 => Some(A {
            r: f(pt[0])?,
            g: f(pt[1])?,
            b: f(pt[2])?,
            a: av,
        }),
        _ => {
            let hs = std::str::from_utf8(pt[0]).ok()?;
            let hu = j(hs)?;
            let sa = g(pt[1])?;
            let li = g(pt[2])?;
            let (r, gv, bv) = k(hu, sa, li);
            Some(A {
                r,
                g: gv,
                b: bv,
                a: av,
            })
        }
    }
}
#[inline(always)]
const fn q(r: u8, g: u8, b: u8, a: u8) -> bool {
    (r & 0x0F) * 0x11 == r
        && (g & 0x0F) * 0x11 == g
        && (b & 0x0F) * 0x11 == b
        && (a & 0x0F) * 0x11 == a
}
static R: LazyLock<Vec<(u32, &'static [u8])>> = LazyLock::new(|| {
    let v: Vec<_> = K
        .iter()
        .filter_map(|&(nm, hx)| {
            l(hx).and_then(|cl| {
                if cl.a == 255 {
                    let rg = ((cl.r as u32) << 16) | ((cl.g as u32) << 8) | (cl.b as u32);
                    Some((rg, nm))
                } else {
                    None
                }
            })
        })
        .collect();
    let mut mp = HashMap::new();
    for (rg, nm) in v {
        mp.entry(rg).or_insert(nm);
    }
    let mut res: Vec<_> = mp.into_iter().collect();
    res.sort_unstable_by_key(|&(rg, _)| rg);
    res
});
#[inline(always)]
fn s(rg: u32) -> Option<&'static [u8]> {
    R.binary_search_by_key(&rg, |&(r, _)| r)
        .ok()
        .map(|i| R[i].1)
}
#[inline(always)]
const fn t(v: u8) -> u8 {
    match v {
        0..=9 => b'0' + v,
        _ => b'a' + (v - 10),
    }
}
#[inline(always)]
fn u(cl: &A) -> String {
    if cl.a != 255 {
        let sh = q(cl.r, cl.g, cl.b, cl.a);
        let mut bf = [0u8; 9];
        let sl = if sh {
            bf[0] = b'#';
            bf[1] = t(cl.r >> 4);
            bf[2] = t(cl.g >> 4);
            bf[3] = t(cl.b >> 4);
            bf[4] = t(cl.a >> 4);
            &bf[..5]
        } else {
            bf[0] = b'#';
            bf[1] = t(cl.r >> 4);
            bf[2] = t(cl.r & 0xF);
            bf[3] = t(cl.g >> 4);
            bf[4] = t(cl.g & 0xF);
            bf[5] = t(cl.b >> 4);
            bf[6] = t(cl.b & 0xF);
            bf[7] = t(cl.a >> 4);
            bf[8] = t(cl.a & 0xF);
            &bf[..9]
        };
        return unsafe { std::str::from_utf8_unchecked(sl) }.to_string();
    }
    let rg = ((cl.r as u32) << 16) | ((cl.g as u32) << 8) | (cl.b as u32);
    if let Some(nm) = s(rg) {
        let sh = q(cl.r, cl.g, cl.b, 255);
        let ml = if sh { 4 } else { 7 };
        if nm.len() < ml {
            return unsafe { std::str::from_utf8_unchecked(nm) }.to_string();
        }
    }
    let sh = q(cl.r, cl.g, cl.b, 255);
    let mut bf = [0u8; 7];
    let sl = if sh {
        bf[0] = b'#';
        bf[1] = t(cl.r >> 4);
        bf[2] = t(cl.g >> 4);
        bf[3] = t(cl.b >> 4);
        &bf[..4]
    } else {
        bf[0] = b'#';
        bf[1] = t(cl.r >> 4);
        bf[2] = t(cl.r & 0xF);
        bf[3] = t(cl.g >> 4);
        bf[4] = t(cl.g & 0xF);
        bf[5] = t(cl.b >> 4);
        bf[6] = t(cl.b & 0xF);
        &bf[..7]
    };
    unsafe { std::str::from_utf8_unchecked(sl) }.to_string()
}
#[inline(always)]
fn v(s: &[u8]) -> &[u8] {
    let a = s
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .unwrap_or(s.len());
    let z = s
        .iter()
        .rposition(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .map_or(a, |i| i + 1);
    &s[a..z]
}
#[inline(always)]
fn w(s: &[u8]) -> String {
    let mut r = String::with_capacity(s.len());
    unsafe {
        let b = r.as_mut_vec();
        b.extend(s.iter().map(|&v| v | 0x20));
    }
    r
}
pub fn shorten_css_color(i: impl AsRef<str>) -> String {
    let s = i.as_ref().as_bytes();
    if s.is_empty() {
        return String::new();
    }
    let tr = v(s);
    if tr.len() < 5 {
        if tr.eq_ignore_ascii_case(b"#f00") {
            return String::from("red");
        }
        return w(tr);
    }
    p(tr).map_or_else(|| w(tr), |x| u(&x))
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
