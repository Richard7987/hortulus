//! Flowers: parametric petals, frontal blooms, tilted blooms and tiny florets.

use crate::color::{self, Rgb};
use crate::geom::{self as g, Cmd, Pt};
use crate::leaf;
use crate::lua::sort;
use crate::paint::{self, Map, Paint, MULT, NONE};
use crate::rng::Rng;
use crate::tune::tune;
use crate::wash::{self, HairOpts, Rays, WashOpts};
use std::f64::consts::PI;
use std::rc::Rc;

struct Kind {
    k: f64,
    e: f64,
    claw: f64,
    notch: f64,
    wave: f64,
    freq: f64,
    wedge: bool,
    shoulder: f64,
    base: f64,
    rise: f64,
    scallop: Option<f64>,
}

const fn kind(k: f64, e: f64, claw: f64, notch: f64, wave: f64, freq: f64) -> Kind {
    Kind { k, e, claw, notch, wave, freq, wedge: false, shoulder: 0.0, base: 0.0, rise: 0.0, scallop: None }
}

fn kind_of(name: &str) -> Kind {
    match name {
        "obovate" => kind(0.72, 0.60, 0.46, 0.10, 0.11, 5.0),
        "lance" => kind(1.15, 1.30, 0.10, 0.00, 0.05, 3.0),
        "round" => kind(0.95, 0.50, 0.20, 0.05, 0.07, 3.0),
        "ghost" => kind(0.85, 0.60, 0.20, 0.06, 0.14, 4.0),
        "fan" => kind(0.56, 0.42, 0.50, 0.04, 0.16, 6.0),
        _ => Kind { k: 0.0, e: 0.0, claw: 0.0, notch: 0.03, wave: 0.12, freq: 6.0, wedge: true, shoulder: 0.74, base: 0.06, rise: 1.3, scallop: Some(0.035) },
    }
}

fn profile(k: &Kind, t: f64) -> f64 {
    if k.wedge {
        if t <= k.shoulder {
            return k.base + (1.0 - k.base) * (PI / 2.0 * t / k.shoulder).sin().powf(k.rise);
        }
        let u = (t - k.shoulder) / (1.0 - k.shoulder);
        return (1.0 - u * u).max(0.0).sqrt();
    }
    (PI * t.powf(k.k)).sin().powf(k.e) * g::smoothstep(0.0, k.claw, t).powf(0.7)
}

pub fn petal(rng: &mut Rng, kind: &str, len: f64, wd: f64, bend: f64, wave: Option<f64>, slot: Option<f64>) -> Vec<Pt> {
    let k = kind_of(kind);
    let n = 26;
    let nl = g::Fbm::new(rng, 32);
    let nr = g::Fbm::new(rng, 32);
    let al = 1.0 + rng.jit(0.12);
    let ar = 1.0 + rng.jit(0.12);
    let wv = wave.unwrap_or(k.wave);
    let mut wd = wd;
    if let Some(sl) = slot {
        let mut fit: f64 = 1.0;
        for i in 4..=20 {
            let t = i as f64 / 20.0;
            let need = wd * profile(&k, t) * al.max(ar) * (1.0 + wv * t.powf(1.5));
            fit = fit.min(sl * len * t * (1.0 - k.notch) / need.max(1e-9));
        }
        wd *= fit;
    }
    let mut left = Vec::with_capacity(n + 1);
    let mut right = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let w = wd * profile(&k, t);
        let y = len * t - k.notch * len * (-((1.0 - t) / 0.045).powf(2.0)).exp();
        let wobble = wv * t.powf(1.5);
        let bx = bend * len * t * t;
        let mut wl = w * al * (1.0 + wobble * nl.at(t * k.freq * 3.0));
        let mut wr = w * ar * (1.0 + wobble * nr.at(t * k.freq * 3.0));
        let (mut yl, mut yr) = (y, y);
        if let (Some(sc), true) = (k.scallop, t > k.shoulder) {
            let s = g::smoothstep(k.shoulder, 1.0, t) * sc * len;
            yl = y + s * nl.at(7.0 + t * k.freq * 2.0);
            yr = y + s * nr.at(7.0 + t * k.freq * 2.0);
        }
        if let Some(sl) = slot {
            let cap = sl * yl.min(yr).max(0.0);
            wl = wl.min(cap + bx).max(0.0);
            wr = wr.min(cap - bx).max(0.0);
        }
        left.push([bx - wl, yl]);
        right.push([bx + wr, yr]);
    }
    let mut out = left;
    out.extend(right[1..n].iter().rev());
    out
}

const ROSE: Rgb = [0.72, 0.36, 0.40];

fn circle(c: Pt, r: f64, n: usize) -> Vec<Pt> {
    (0..n)
        .map(|k| {
            let t = k as f64 * 2.0 * PI / n as f64;
            [c[0] + r * t.cos(), c[1] + r * t.sin()]
        })
        .collect()
}

pub fn eye(pt: &mut Paint, rng: &mut Rng, cx: f64, cy: f64, r0: f64, halo: f64, stamens: i64) {
    let c = [cx, cy];
    let mut glow = Vec::with_capacity(6);
    for k in 1..=6 {
        let r = r0 * halo * (1.0 - (k - 1) as f64 / 7.0);
        let p = g::polar(c, rng.range(0.0, 2.0 * PI), r0 * 0.012);
        glow.push(g::smooth(&g::deform(&circle(p, r, 18), rng, r * 0.12), true));
    }
    for b in &glow {
        pt.fill(b, paint::WHITE, 0.18, NONE);
    }
    for k in 1..=4 {
        let r = r0 * (0.14 - 0.022 * k as f64);
        let b = g::smooth(&g::deform(&circle(c, r, 16), rng, r * 0.2), true);
        pt.fill(&b, ROSE, 0.08, MULT);
    }

    let m = stamens;
    let (mut fil, mut anthers, mut warm, mut pollen) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let a0 = rng.range(0.0, 2.0 * PI);
    for i in 1..=m {
        let a = a0 + ((i - 1) as f64 + rng.jit(0.35)) * 2.0 * PI / m as f64;
        let len = r0 * rng.range(0.10, 0.21);
        let p0 = g::polar(c, a, r0 * 0.035);
        let dir = a + rng.jit(0.12);
        let bend = rng.jit(0.6);
        let line = g::curve(p0, dir, len, bend, 0.0, 6);
        let (tip, ta) = (line[line.len() - 1], g::angle(line[line.len() - 2], line[line.len() - 1]));
        fil.push(line);
        let ar = r0 * rng.range(0.007, 0.011);
        let anther = g::transform(&circle([0.0, 0.0], 1.0, 10), 0.0, 0.0, 0.0, ar * 1.5, ar);
        anthers.push(g::transform(&anther, tip[0], tip[1], ta, 1.0, 1.0));
        warm.push(rng.range(0.0, 1.0));
    }
    for _ in 0..(m as f64 * 0.6).floor() as i64 {
        let q = g::polar(c, rng.range(0.0, 2.0 * PI), r0 * rng.range(0.05, 0.13));
        pollen.push(vec![Cmd::O(q[0], q[1], r0 * rng.range(0.004, 0.008))]);
    }
    wash::hairlines(pt, &fil, paint::INK, 0.12f64.max(r0 * 0.0035), 0.32, HairOpts { head: Some(0.3), blend: MULT.blend });
    let (mut a1, mut a2) = (Vec::new(), Vec::new());
    for (i, a) in anthers.iter().enumerate() {
        let path = g::smooth(a, true);
        if warm[i] < 0.6 {
            a1.push(path);
        } else {
            a2.push(path);
        }
    }
    if !a1.is_empty() {
        pt.fill_many(&a1, paint::COPPER, 0.85, NONE);
    }
    if !a2.is_empty() {
        pt.fill_many(&a2, ROSE, 0.8, NONE);
    }
    if !pollen.is_empty() {
        pt.fill_many(&pollen, paint::COPPER, 0.55, NONE);
    }
    let disc = g::smooth(&g::deform(&circle(c, r0 * 0.032, 12), rng, r0 * 0.005), true);
    pt.fill(&disc, [0.55, 0.26, 0.22], 0.95, NONE);
    pt.stroke(&disc, paint::INK, 0.12f64.max(r0 * 0.004), 0.45, NONE);
    pt.fill(&[Cmd::O(cx - r0 * 0.009, cy + r0 * 0.009, r0 * 0.010)], paint::WHITE, 0.35, NONE);
}

#[derive(Clone, Copy)]
pub struct WashLook {
    ramp: &'static str,
    pigment: Rgb,
    pool: Rgb,
    strength: f64,
    rings: i64,
    lift: f64,
    streaks: f64,
    dark: i64,
    grain: f64,
    outline: f64,
    knockout: Option<f64>,
}

fn look_of(name: &str) -> WashLook {
    match name {
        "fan" => WashLook { ramp: "fan", pigment: paint::BLUE, pool: paint::DEEP, strength: 1.15, rings: 12, lift: 0.05, streaks: 40.0, dark: 16, grain: 2.0, outline: 0.5, knockout: None },
        "hero" => WashLook { ramp: "hero", pigment: paint::BLUE, pool: paint::DEEP, strength: 1.15, rings: 12, lift: 0.05, streaks: 40.0, dark: 22, grain: 2.0, outline: 0.55, knockout: None },
        "soft" => WashLook { ramp: "soft", pigment: paint::SKY, pool: paint::DEEP, strength: 0.95, rings: 10, lift: 0.07, streaks: 22.0, dark: 12, grain: 0.6, outline: 0.70, knockout: None },
        "ghost" => WashLook { ramp: "ghost", pigment: paint::MIST, pool: paint::SKY, strength: 0.45, rings: 8, lift: 0.12, streaks: 8.0, dark: 3, grain: 0.15, outline: 0.45, knockout: Some(0.35) },
        _ => WashLook { ramp: "vivid", pigment: paint::BLUE, pool: paint::DEEP, strength: 1.15, rings: 12, lift: 0.05, streaks: 40.0, dark: 22, grain: 0.9, outline: 0.55, knockout: None },
    }
}

fn tip_of(poly: &[Pt], eye: Pt) -> Pt {
    let (mut best, mut bd) = (poly[0], -1.0);
    for p in poly {
        let d = g::dist(*p, eye);
        if d > bd {
            best = *p;
            bd = d;
        }
    }
    best
}

#[derive(Default)]
struct Extra {
    eye_fade: Option<f64>,
    lifts: Option<f64>,
    veins: Option<i64>,
    rays: Option<([f64; 2], [f64; 2], i64)>,
}

fn wash_petal(pt: &mut Paint, rng: &mut Rng, poly: &[Pt], len: f64, light: Pt, look: WashLook, lost: Option<f64>, extra: &Extra) {
    wash::wash(
        pt,
        poly,
        rng,
        WashOpts {
            ramp: Some(look.ramp),
            axis: Some((light, tip_of(poly, light))),
            size: len,
            light: Some(light),
            pigment: Some(look.pigment),
            pool: Some(look.pool),
            strength: Some(look.strength),
            rings: Some(look.rings),
            lift: Some(look.lift),
            bleed: Some((len * 0.02).min(1.6)),
            streaks: (look.streaks * g::clamp(len / 40.0, 0.35, 1.4)).floor(),
            dark_streaks: extra.veins.unwrap_or(look.dark),
            grain: look.grain,
            rays: extra.rays.map(|r| Rays { count: r.2, reach: r.0, contrast: r.1 }),
            outline: look.outline,
            outline_w: Some(0.25f64.max(len * 0.009)),
            knockout: look.knockout,
            lost,
            eye_fade: extra.eye_fade,
            lifts: extra.lifts,
            ..Default::default()
        },
    );
}

pub fn style_parts(style: &str) -> (&'static str, &'static str) {
    match style {
        "fan" => ("fan", "fan"),
        "round" => ("round", "vivid"),
        "soft" => ("round", "soft"),
        "obovate" => ("obovate", "vivid"),
        _ => ("wedge", "vivid"),
    }
}

pub fn pick_style(rng: &mut Rng, names: &[&'static str]) -> &'static str {
    let w: Vec<(&'static str, f64)> = names.iter().map(|n| (*n, tune().bloom(n).weight.max(0.0))).collect();
    let total: f64 = w.iter().map(|e| e.1).sum();
    if total <= 0.0 {
        return names[0];
    }
    rng.weighted(&w).unwrap_or(names[0])
}

pub fn slot(n: i64, gap: f64, spacing: f64) -> f64 {
    (PI / n as f64 - spacing).max(0.05).tan() * (1.0 - gap)
}

#[derive(Default, Clone)]
pub struct BloomOpts {
    pub style: Option<&'static str>,
    pub kind: Option<&'static str>,
    pub petals: Option<i64>,
    pub rot: Option<f64>,
    pub look: Option<&'static str>,
    pub width: Option<f64>,
    pub lost: Option<f64>,
    pub far_first: bool,
}

pub fn bloom(pt: &mut Paint, rng: &mut Rng, cx: f64, cy: f64, r0: f64, o: &BloomOpts) {
    let st = o.style.map(|s| tune().bloom(s));
    let base = o.style.map(style_parts);
    let n = match (o.petals, st) {
        (Some(p), _) => p,
        (None, Some(s)) => rng.int(s.petals[0] as i64, s.petals[1] as i64),
        _ => 5,
    };
    let width = match (o.width, st) {
        (Some(w), _) => w,
        (None, Some(s)) => rng.rangev(s.width),
        _ => 0.34,
    };
    let kind = o.kind.or(base.map(|b| b.0)).unwrap_or("obovate");
    let look_name = o.look.or(base.map(|b| b.1)).unwrap_or("vivid");
    let look = look_of(look_name);
    let ragged = st.map_or(0.08, |s| s.ragged);
    let spacing = st.map_or(0.12, |s| s.spacing);
    let mut extra = Extra::default();
    if let Some(s) = st {
        extra.eye_fade = s.eye_fade;
        extra.lifts = s.backruns;
        extra.veins = Some((rng.rangev(s.veins) + 0.5).floor() as i64);
        let rays = (rng.rangev(s.rays) + 0.5).floor() as i64;
        if rays > 0 {
            extra.rays = Some((s.ray_reach, s.ray_contrast, rays));
        }
    }
    let a0 = match o.rot {
        Some(r) => r,
        None => rng.range(0.0, 2.0 * PI),
    };
    let sl = st.and_then(|s| s.gap.map(|gap| slot(n, gap, spacing)));
    let mut petals: Vec<(Rng, f64)> = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let mut r = rng.fork(&format!("petal{}", i));
        let a = a0 + (i - 1) as f64 * 2.0 * PI / n as f64 + r.jit(spacing);
        petals.push((r, a));
    }
    if o.far_first {
        sort(&mut petals, |p, q| p.1.sin() > q.1.sin());
    }
    for (r, a) in petals.iter_mut() {
        let len = r0 * r.range(1.0 - ragged * 1.5, 1.0 + ragged * 0.5);
        let wd = r0 * width * r.range(0.92, 1.08);
        let bend = r.jit(if sl.is_some() { 0.04 } else { 0.10 });
        let local = petal(r, kind, len, wd, bend, st.map(|s| s.rim), sl);
        let poly = g::transform(&local, cx, cy, *a - PI / 2.0, 1.0, 1.0);
        wash_petal(pt, r, &poly, len, [cx, cy], look, o.lost, &extra);
    }
    let halo = if look_name == "ghost" {
        0.25
    } else if kind == "fan" || kind == "wedge" {
        0.2
    } else {
        0.36
    };
    let mut er = rng.fork("eye");
    eye(pt, &mut er, cx, cy, r0, halo, if look_name == "ghost" { 8 } else { 14 });
}

#[derive(Default, Clone)]
pub struct TiltedOpts {
    pub kind: Option<&'static str>,
    pub petals: Option<i64>,
    pub cup: Option<f64>,
    pub look: Option<&'static str>,
    pub rot: Option<f64>,
    pub width: Option<f64>,
}

pub fn tilted(pt: &mut Paint, rng: &mut Rng, cx: f64, cy: f64, r0: f64, facing: f64, tilt: f64, o: &TiltedOpts) -> Pt {
    let n = o.petals.unwrap_or(5);
    let look = look_of(o.look.unwrap_or("vivid"));
    let calyx = color::recolor(paint::DEEP, color::part_ink(color::part_of(look.ramp)));
    let cup = o.cup.unwrap_or(0.6);
    let (ct, st) = (tilt.cos(), tilt.sin());
    let (cf, sf) = ((facing - PI / 2.0).cos(), (facing - PI / 2.0).sin());
    let project = |x: f64, y: f64, z: Option<f64>| -> (Pt, f64) {
        let z = z.unwrap_or(cup * (x * x + y * y) / r0);
        let (y2, z2) = (y * ct - z * st, y * st + z * ct);
        ([cx + x * cf - y2 * sf, cy + x * sf + y2 * cf], z2)
    };

    let tube: Vec<Pt> = (0..=8).map(|i| project(0.0, 0.0, Some(-r0 * 0.42 * i as f64 / 8.0)).0).collect();
    let stem_at = tube[8];
    let rb = g::ribbon(&g::spline_points(&[tube[8], tube[4], tube[0]], 4), |t| r0 * (0.035 + 0.07 * t.powf(1.5)));
    pt.fill(&g::smooth(&rb, true), calyx, 0.92, MULT);
    let mut sepals = Vec::new();
    for _ in 0..3 {
        let b = rng.range(0.0, 2.0 * PI);
        let p0 = project(0.0, 0.0, Some(-r0 * 0.05)).0;
        let p1 = project(r0 * 0.28 * b.cos(), r0 * 0.28 * b.sin(), Some(-r0 * 0.2)).0;
        sepals.push([p0[0], p0[1], g::angle(p0, p1), g::dist(p0, p1) * 1.1 + r0 * 0.08]);
    }
    let mut sr = rng.fork("sepals");
    leaf::leaflets(pt, &mut sr, &sepals, Some(calyx), 0.9);
    let a0 = match o.rot {
        Some(r) => r,
        None => rng.range(0.0, 2.0 * PI),
    };
    struct P {
        poly: Vec<Pt>,
        z: f64,
        len: f64,
        r: Rng,
    }
    let mut petals = Vec::new();
    for i in 1..=n {
        let mut r = rng.fork(&format!("tp{}", i));
        let a = a0 + (i - 1) as f64 * 2.0 * PI / n as f64 + r.jit(0.15);
        let len = r0 * r.range(0.85, 1.05);
        let wd = r0 * o.width.unwrap_or(0.34) * r.range(0.92, 1.08);
        let bend = r.jit(0.15);
        let local = petal(&mut r, o.kind.unwrap_or("lance"), len, wd, bend, None, None);
        let pts = g::transform(&local, 0.0, 0.0, a - PI / 2.0, 1.0, 1.0);
        let mut zsum = 0.0;
        let mut poly = Vec::with_capacity(pts.len());
        for p in &pts {
            let (q, z) = project(p[0], p[1], None);
            poly.push(q);
            zsum += z;
        }
        petals.push(P { poly, z: zsum / pts.len() as f64, len, r });
    }
    sort(&mut petals, |a, b| a.z < b.z);
    for p in petals.iter_mut() {
        wash_petal(pt, &mut p.r, &p.poly, p.len, [cx, cy], look, None, &Extra::default());
    }
    let e = project(0.0, 0.0, None).0;
    let mut er = rng.fork("eye");
    eye(pt, &mut er, e[0], e[1], r0 * 0.7, 0.3, 9);
    stem_at
}

#[derive(Clone, Copy)]
pub struct Floret {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub rot: f64,
    pub tint: usize,
    pub soft: bool,
}

pub fn florets(pt: &mut Paint, items: &[Floret]) {
    let mut tints: [Vec<Vec<Cmd>>; 3] = Default::default();
    let mut soft: [Vec<Vec<Cmd>>; 3] = Default::default();
    let (mut eyes, mut specks, mut rims, mut halos, mut soft_eyes) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for f in items {
        let (x, y, r, rot) = (f.x, f.y, f.r, f.rot);
        if f.soft {
            halos.push(vec![Cmd::O(x, y, 1.05 * r)]);
        }
        for i in 0..5 {
            let a = rot + i as f64 * 2.0 * PI / 5.0;
            let petal = vec![Cmd::O(x + 0.55 * r * a.cos(), y + 0.55 * r * a.sin(), 0.47 * r)];
            if r > 1.1 && !f.soft {
                rims.push(petal.clone());
            }
            if f.soft {
                soft[f.tint - 1].push(petal);
            } else {
                tints[f.tint - 1].push(petal);
            }
        }
        if f.soft {
            if r > 1.1 {
                soft_eyes.push(vec![Cmd::O(x, y, 0.22 * r)]);
            }
        } else if r > 0.9 {
            eyes.push(vec![Cmd::O(x, y, 0.28 * r)]);
            specks.push(vec![Cmd::O(x, y, 0.12 * r)]);
        }
    }
    let tint = color::part_ink(Some("sprays"));
    let (deep, blue, sky) = (color::recolor(paint::DEEP, tint), color::recolor(paint::BLUE, tint), color::recolor(paint::SKY, tint));
    let cols = [deep, blue, sky];
    if !halos.is_empty() {
        pt.fill_many(&halos, sky, 0.14, MULT);
    }
    for i in 0..3 {
        if !tints[i].is_empty() {
            pt.fill_many(&tints[i], cols[i], 0.88, MULT);
        }
        if !soft[i].is_empty() {
            pt.fill_many(&soft[i], cols[i], 0.55, MULT);
        }
    }
    if !soft_eyes.is_empty() {
        pt.fill_many(&soft_eyes, paint::WHITE, 0.6, NONE);
    }
    if !rims.is_empty() {
        pt.stroke_many(&rims, paint::mix(color::recolor(paint::INK, tint), blue, 0.5), 0.14, 0.45, MULT);
    }
    if !eyes.is_empty() {
        pt.fill_many(&eyes, paint::WHITE, 0.85, NONE);
    }
    if !specks.is_empty() {
        pt.fill_many(&specks, paint::COPPER, 0.85, NONE);
    }
}

pub fn bud(pt: &mut Paint, x: f64, y: f64, ang: f64, len: f64, rgb: Rgb) {
    let mut pts = Vec::with_capacity(32);
    let w = |t: f64| len * 0.32 * (PI * t.powf(0.7)).sin().powf(0.9);
    for i in 0..=16 {
        let t = i as f64 / 16.0;
        pts.push([-w(t), len * t]);
    }
    for i in (1..=15).rev() {
        let t = i as f64 / 16.0;
        pts.push([w(t), len * t]);
    }
    let poly = g::transform(&pts, x, y, ang - PI / 2.0, 1.0, 1.0);
    pt.fill(&g::smooth(&poly, true), rgb, 0.9, MULT);
}

const CUP: f64 = 0.08;
const VIEW: f64 = 2.8;

pub fn tilt(cx: f64, cy: f64, r0: f64, deg: f64) -> (Map, f64) {
    let t = deg.to_radians();
    let (ct, st) = (t.cos(), t.sin());
    let d = VIEW * r0;
    let f = move |x: f64, y: f64| -> (f64, f64) {
        let (dx, dy) = (x - cx, y - cy);
        let z = CUP * (dx * dx + dy * dy).min(1.6 * r0 * r0) / r0;
        let (sy, depth) = (dy * ct + z * st, z * ct - dy * st);
        let s = d / (d - depth);
        (cx + dx * s, cy + sy * s)
    };
    let mut rh: f64 = 0.0;
    for k in 0..16 {
        let a = k as f64 * PI / 8.0;
        let (_, y) = f(cx + r0 * a.cos(), cy + r0 * a.sin());
        rh = rh.max((y - cy).abs());
    }
    (Rc::new(f), rh)
}

pub fn hero(pt: &mut Paint, rng: &mut Rng, h: [f64; 3]) {
    let hc = &tune().hero;
    let f = if hc.tilt > 0.0 { Some(tilt(h[0], h[1], h[2], hc.tilt).0) } else { None };
    pt.project(f);
    bloom(
        pt,
        rng,
        h[0],
        h[1],
        h[2],
        &BloomOpts {
            style: Some(hc.style),
            look: Some("hero"),
            petals: Some(hc.petals),
            width: Some(hc.petal_width),
            rot: if hc.rotation >= 0.0 { Some(hc.rotation.to_radians()) } else { None },
            lost: Some(hc.lost_edges),
            far_first: hc.tilt > 0.0,
            kind: None,
        },
    );
    pt.project(None);
}
