//! Colour in OKLCH: per-shape ramps and measured ink colours.

use crate::inks::{GROUPS, RAMPS};
use crate::lua::fmod;
use crate::rng::Rng;
use crate::tune::{palette, tune};

pub type Rgb = [f64; 3];

fn cbrt(x: f64) -> f64 {
    if x < 0.0 {
        -((-x).powf(1.0 / 3.0))
    } else {
        x.powf(1.0 / 3.0)
    }
}

fn encode(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn linear(l: f64, ch: f64, c: f64, sn: f64) -> (f64, f64, f64) {
    let (a, b) = (ch * c, ch * sn);
    let cube = |x: f64| x * x * x;
    let ll = cube(l + 0.3963377774 * a + 0.2158037573 * b);
    let m = cube(l - 0.1055613458 * a - 0.0638541728 * b);
    let s = cube(l - 0.0894841775 * a - 1.2914855480 * b);
    (
        4.0767416621 * ll - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * ll + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * ll - 0.7034186147 * m + 1.7076147010 * s,
    )
}

fn in_gamut(l: f64, ch: f64, c: f64, sn: f64) -> bool {
    let (r, g, b) = linear(l, ch, c, sn);
    let e = 1e-4;
    r >= -e && r <= 1.0 + e && g >= -e && g <= 1.0 + e && b >= -e && b <= 1.0 + e
}

pub fn max_chroma(l: f64, h: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 0.4);
    let (c, sn) = (h.to_radians().cos(), h.to_radians().sin());
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if in_gamut(l, mid, c, sn) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

pub fn oklch(l: f64, ch: f64, h: f64) -> Rgb {
    let ch = ch.min(max_chroma(l, h));
    let (r, g, b) = linear(l, ch, h.to_radians().cos(), h.to_radians().sin());
    let clamp = |v: f64| encode(v.max(0.0)).min(1.0).max(0.0);
    [clamp(r), clamp(g), clamp(b)]
}

pub fn lightness(rgb: Rgb) -> f64 {
    let dec = |c: f64| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (dec(rgb[0]), dec(rgb[1]), dec(rgb[2]));
    let l = cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    let m = cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    let s = cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s
}

pub fn ramp_of(name: &str) -> Option<&'static [[f64; 3]]> {
    RAMPS.iter().find(|e| e.0 == name).map(|e| e.1)
}

pub fn group_of(name: &str) -> Option<&'static [&'static str]> {
    GROUPS.iter().find(|e| e.0 == name).map(|e| e.1)
}

fn richest(r: &[[f64; 3]]) -> usize {
    let (mut best, mut bi) = (-1.0, 0);
    for (i, s) in r.iter().enumerate() {
        let c = s[1] * s[1] + s[2] * s[2];
        if c > best {
            best = c;
            bi = i;
        }
    }
    bi
}

pub fn ink_at(name: &str, l: f64) -> (f64, f64) {
    let all = ramp_of(name).expect("ink");
    let r = &all[richest(all)..];
    let (lo, hi) = (r[0], r[r.len() - 1]);
    let (mut a, mut b) = (lo[1], lo[2]);
    if l <= lo[0] {
    } else if l >= hi[0] {
        a = hi[1];
        b = hi[2];
    } else {
        for i in 0..r.len() - 1 {
            let (p, q) = (r[i], r[i + 1]);
            if l >= p[0] && l <= q[0] {
                let t = (l - p[0]) / (q[0] - p[0]).max(1e-9);
                a = p[1] + (q[1] - p[1]) * t;
                b = p[2] + (q[2] - p[2]) * t;
                break;
            }
        }
    }
    ((a * a + b * b).sqrt(), fmod(b.atan2(a).to_degrees(), 360.0))
}

pub fn ink_for(rng: &mut Rng, choice: Option<&'static str>) -> Option<&'static str> {
    let choice = choice?;
    if choice == "look" {
        return None;
    }
    if let Some(g) = group_of(choice) {
        return Some(g[(rng.int(1, g.len() as i64) - 1) as usize]);
    }
    if ramp_of(choice).is_some() {
        return Some(choice);
    }
    None
}

/// `ink_for` taking a group's first ink, spending no randomness.
pub fn ink_first(choice: Option<&'static str>) -> Option<&'static str> {
    let choice = choice?;
    if choice == "look" {
        return None;
    }
    if let Some(g) = group_of(choice) {
        return Some(g[0]);
    }
    if ramp_of(choice).is_some() {
        return Some(choice);
    }
    None
}

pub fn ink_rgb(name: &str, l: f64, share: f64) -> Rgb {
    let (ch, h) = ink_at(name, l);
    oklch(l, ch * share, h)
}

pub fn part_of(look: &str) -> Option<&'static str> {
    match look {
        "leaf" => Some("leaves"),
        "hero" => Some("hero"),
        "vivid" | "fan" | "soft" => Some("blooms"),
        "ghost" => Some("ghost"),
        _ => None,
    }
}

pub fn part_ink(part: Option<&str>) -> Option<&'static str> {
    let choice = palette(part?)?;
    if choice == "look" || choice == "blue" {
        return None;
    }
    if ramp_of(choice).is_some() {
        return Some(choice);
    }
    group_of(choice).map(|g| g[0])
}

pub fn recolor(rgb: Rgb, ink: Option<&str>) -> Rgb {
    match ink {
        Some(i) if !group_of("blue").unwrap().contains(&i) => ink_rgb(i, lightness(rgb), 1.0),
        _ => rgb,
    }
}

pub struct RampInfo {
    pub h: f64,
    pub l: f64,
    pub share: f64,
    pub ink: Option<&'static str>,
}

pub fn ramp(rng: &mut Rng, look: &str, part: Option<&str>) -> (Vec<(f64, Rgb)>, RampInfo) {
    let cfg = &tune().color;
    let lk = tune().look(look);
    let h = cfg.hue + rng.jit(cfg.hue_jitter);
    let base_l = rng.rangev(lk.base_l);
    let tip_l = rng.rangev(lk.tip_l);
    let mut share = rng.rangev(lk.chroma);
    let curve = rng.rangev(lk.curve);
    let mut grey = 1.0;
    if look == "leaf" && rng.chance(cfg.grey_chance) {
        grey = cfg.grey_share;
        share *= cfg.grey_share;
    }
    let n = 8;
    let mut stops = Vec::with_capacity(n);
    let key = part.or_else(|| part_of(look));
    let ink = ink_for(rng, key.and_then(palette));
    if let Some(ink) = ink {
        for i in 0..n {
            let t = i as f64 / (n as f64 - 1.0);
            let l = base_l + (tip_l - base_l) * t.powf(curve);
            stops.push((t, ink_rgb(ink, l, grey)));
        }
        let (ch, ih) = ink_at(ink, tip_l);
        let share = (ch * grey / max_chroma(tip_l, ih).max(1e-6)).min(1.0);
        return (stops, RampInfo { h: ih, l: tip_l, share, ink: Some(ink) });
    }
    for i in 0..n {
        let t = i as f64 / (n as f64 - 1.0);
        let l = base_l + (tip_l - base_l) * t.powf(curve);
        let hh = h + cfg.hue_drift * (1.0 - t);
        let s = share * (0.55 + 0.45 * t);
        stops.push((t, oklch(l, s * max_chroma(l, hh), hh)));
    }
    (stops, RampInfo { h, l: tip_l, share, ink: None })
}
