//! Gilded marginalia: compass stars, sparkles, edge chains, knots and the star panel.

use crate::color::{self, Rgb};
use crate::geom::{self as g, Cmd, Pt};
use crate::orn_kit as k;
use crate::paint::{self, Paint, NONE};
use crate::rng::Rng;
use crate::wash::{self, WashOpts};
use std::f64::consts::PI;

fn tri(a: Pt, b: Pt, c: Pt) -> Vec<Cmd> {
    vec![Cmd::M(a[0], a[1]), Cmd::L(b[0], b[1]), Cmd::L(c[0], c[1]), Cmd::Z]
}

pub fn compass(pt: &mut Paint, x: f64, y: f64, r0: f64, hair: f64, ring: i64) {
    let c = [x, y];
    let (mut lit, mut shade) = (Vec::new(), Vec::new());
    for kk in 0..8 {
        let a = kk as f64 * PI / 4.0 + PI / 2.0;
        let major = kk % 2 == 0;
        let len = if major { r0 } else { r0 * 0.48 };
        let half = if major { 0.11 } else { 0.13 };
        let tip = g::polar(c, a, len);
        let l = g::polar(c, a + PI / 4.0, len * half);
        let r = g::polar(c, a - PI / 4.0, len * half);
        lit.push(tri(c, tip, l));
        shade.push(tri(c, tip, r));
    }
    if hair > 0.0 {
        let h = r0 * hair;
        pt.stroke_many(&[g::polyline(&[[x, y - h], [x, y + h]], false), g::polyline(&[[x - h, y], [x + h, y]], false)], paint::GOLD, 0.22, 0.75, NONE);
    }
    pt.fill_many(&shade, paint::COPPER, 0.95, NONE);
    pt.fill_many(&lit, paint::GOLD, 0.95, NONE);
    pt.fill(&[Cmd::O(x, y, r0 * 0.07)], paint::CREAM, 1.0, NONE);
    pt.stroke(&[Cmd::O(x, y, r0 * 0.07)], paint::COPPER, 0.2, 0.9, NONE);
    if ring > 0 {
        let dots: Vec<Vec<Cmd>> = (0..ring)
            .map(|kk| {
                let a = (kk as f64 + 0.5) * 2.0 * PI / ring as f64 + PI / 2.0;
                let p = g::polar(c, a, r0 * 0.62);
                vec![Cmd::O(p[0], p[1], r0 * 0.035)]
            })
            .collect();
        pt.fill_many(&dots, paint::GOLD, 0.9, NONE);
    }
}

pub fn sparkle(x: f64, y: f64, r: f64) -> Vec<Cmd> {
    let k = r * 0.16;
    vec![
        Cmd::M(x, y + r),
        Cmd::C(x, y + k, x + k, y, x + r, y),
        Cmd::C(x + k, y, x, y - k, x, y - r),
        Cmd::C(x, y - k, x - k, y, x - r, y),
        Cmd::C(x - k, y, x, y + k, x, y + r),
        Cmd::Z,
    ]
}

#[derive(Clone, Copy)]
pub struct Sparkle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub kind: &'static str,
}

pub fn sparkles(pt: &mut Paint, items: &[Sparkle]) {
    let (mut paths, mut dots, mut hearts, mut stars) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for s in items {
        let (x, y, r) = (s.x, s.y, s.r);
        match s.kind {
            "lozenge" => lozenge(pt, x, y, r * 0.8),
            "rosette" => {
                for kk in 0..4 {
                    let p = g::polar([x, y], kk as f64 * PI / 2.0, r * 0.5);
                    dots.push(vec![Cmd::O(p[0], p[1], r * 0.3)]);
                }
                hearts.push(vec![Cmd::O(x, y, r * 0.2)]);
            }
            "star5" => stars.push(g::polyline(&k::star(x, y, r * 0.75, r * 0.3, 5, PI / 2.0), true)),
            "crescent" => {
                stars.push(g::smooth(&k::crescent(x, y, r * 0.62, 0.62, PI / 4.0), true));
                dots.push(vec![Cmd::O(x + r * 0.8, y - r * 0.45, r * 0.1)]);
            }
            "trefoil" => {
                dots.extend(k::trefoil(x, y - r * 0.1, r * 0.8, PI / 2.0));
                hearts.push(vec![Cmd::O(x, y - r * 0.05, r * 0.12)]);
            }
            "cross" => {
                paths.push(sparkle(x, y, r));
                for kk in 0..4 {
                    let p = g::polar([x, y], kk as f64 * PI / 2.0, r * 1.4);
                    dots.push(vec![Cmd::O(p[0], p[1], r * 0.13)]);
                }
            }
            kind => {
                paths.push(sparkle(x, y, r));
                if kind == "dotted" {
                    for kk in 0..4 {
                        let p = g::polar([x, y], PI / 4.0 + kk as f64 * PI / 2.0, r * 0.75);
                        dots.push(vec![Cmd::O(p[0], p[1], r * 0.1)]);
                    }
                }
            }
        }
    }
    if !paths.is_empty() {
        pt.fill_many(&paths, paint::GOLD, 0.9, NONE);
    }
    if !dots.is_empty() {
        pt.fill_many(&dots, paint::GOLD, 0.85, NONE);
    }
    if !stars.is_empty() {
        pt.fill_many(&stars, paint::GOLD, 0.92, NONE);
    }
    if !hearts.is_empty() {
        pt.fill_many(&hearts, paint::COPPER, 0.85, NONE);
    }
}

pub fn lozenge(pt: &mut Paint, x: f64, y: f64, h: f64) {
    let w = h * 0.6;
    let outline = g::polyline(&[[x, y + h], [x + w, y], [x, y - h], [x - w, y]], true);
    pt.fill(&g::polyline(&[[x, y + h * 0.45], [x + w * 0.45, y], [x, y - h * 0.45], [x - w * 0.45, y]], true), paint::COPPER, 0.85, NONE);
    pt.stroke(&outline, paint::GOLD, 0.25, 0.9, NONE);
    pt.fill_many(&[vec![Cmd::O(x, y + h * 1.5, h * 0.12)], vec![Cmd::O(x, y - h * 1.5, h * 0.12)]], paint::GOLD, 0.9, NONE);
}

pub fn dot_cross(pt: &mut Paint, x: f64, y: f64, s: f64) {
    let mut dots = vec![vec![Cmd::O(x, y, s * 0.2)]];
    for kk in 0..4 {
        let p = g::polar([x, y], kk as f64 * PI / 2.0, s);
        dots.push(vec![Cmd::O(p[0], p[1], s * 0.11)]);
    }
    pt.fill_many(&dots, paint::GOLD, 0.9, NONE);
}

pub fn finial(pt: &mut Paint, x: f64, y: f64, s: f64) {
    pt.stroke_many(&[g::polyline(&[[x, y + s], [x, y - s * 0.6]], false), g::polyline(&[[x - s * 0.45, y + s * 0.35], [x + s * 0.45, y + s * 0.35]], false)], paint::COPPER, 0.3, 0.9, NONE);
    let d = y - s * 0.6;
    pt.fill(
        &[
            Cmd::M(x, d - s * 0.7),
            Cmd::C(x + s * 0.35, d - s * 0.25, x + s * 0.18, d, x, d + s * 0.05),
            Cmd::C(x - s * 0.18, d, x - s * 0.35, d - s * 0.25, x, d - s * 0.7),
            Cmd::Z,
        ],
        paint::GOLD,
        0.9,
        NONE,
    );
}

pub fn hook(pt: &mut Paint, x: f64, y: f64, s: f64, dir: f64) {
    let b = -dir * s * 0.62;
    let pts = g::spline_points(&[[x, y + s], [x + b, y + s * 0.4], [x + b, y - s * 0.4], [x, y - s * 0.8], [x - b * 0.35, y - s * 0.55]], 6);
    pt.stroke(&g::polyline(&pts, false), paint::GOLD, 0.28, 0.85, NONE);
    pt.fill(&[Cmd::O(x, y + s, s * 0.09)], paint::GOLD, 0.9, NONE);
}

pub fn chain(pt: &mut Paint, rng: &mut Rng, x: f64, ytop: f64, ybot: f64, foot: bool) {
    let mut stops: Vec<(f64, &str, f64)> = Vec::new();
    let mut y = ytop - rng.range(10.0, 24.0);
    let last = if foot { ybot + 34.0 } else { ybot + 8.0 };
    while y > last {
        let kind = rng.weighted(&[("lozenge", 3.0), ("cross", 2.0), ("finial", 2.0), ("hook", 1.5), ("knot", 1.5)]).unwrap();
        let half = if kind == "knot" {
            8.0
        } else if kind == "hook" {
            7.0
        } else {
            5.5
        };
        stops.push((y, kind, half));
        y -= rng.range(40.0, 70.0);
    }
    if foot && ytop - ybot >= 18.0 {
        stops.push((ybot + 8.0, "knot", 8.0));
    }
    let mut spans = Vec::new();
    let mut from = ytop;
    for s in &stops {
        spans.push((from, s.0 + s.2 + 2.0));
        from = s.0 - s.2 - 2.0;
    }
    spans.push((from, ybot));
    let (mut dots, mut rods) = (Vec::new(), Vec::new());
    for (a, b) in spans {
        if a - b > 3.0 {
            if a - b > 14.0 && rng.chance(0.35) {
                rods.push(g::polyline(&[[x, a - 1.5], [x, b + 1.5]], false));
                dots.push(vec![Cmd::O(x, a, 0.55)]);
                dots.push(vec![Cmd::O(x, b, 0.55)]);
            } else {
                let mut yy = a;
                while yy >= b {
                    dots.push(vec![Cmd::O(x, yy, 0.5)]);
                    yy -= 4.6;
                }
            }
        }
    }
    if !rods.is_empty() {
        pt.stroke_many(&rods, paint::GOLD, 0.25, 0.8, NONE);
    }
    if !dots.is_empty() {
        pt.fill_many(&dots, paint::GOLD, 0.85, NONE);
    }
    for s in &stops {
        match s.1 {
            "lozenge" => lozenge(pt, x, s.0, 3.6),
            "cross" => dot_cross(pt, x, s.0, 3.0),
            "finial" => finial(pt, x, s.0, 4.5),
            "hook" => {
                let dir = if rng.chance(0.5) { 1.0 } else { -1.0 };
                hook(pt, x, s.0, 6.0, dir)
            }
            _ => knot(pt, x, s.0, 4.8, 7.2),
        }
    }
}

pub fn knot(pt: &mut Paint, x: f64, y: f64, w: f64, h: f64) {
    let mut paths = vec![
        g::polyline(&[[x, y + h], [x + w, y], [x, y - h], [x - w, y]], true),
        g::polyline(&[[x, y + h * 0.45], [x + w * 0.45, y], [x, y - h * 0.45], [x - w * 0.45, y]], true),
    ];
    for d in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
        paths.push(vec![Cmd::O(x + d[0] * w * 0.5, y + d[1] * h * 0.5, w.min(h) * 0.26)]);
    }
    pt.stroke_many(&paths, paint::GOLD, 0.3, 0.85, NONE);
    pt.fill(&[Cmd::O(x, y, w.min(h) * 0.1)], paint::COPPER, 0.85, NONE);
}

fn notched(x0: f64, y0: f64, x1: f64, y1: f64, c: f64, n: usize) -> Vec<Pt> {
    let corners = [(x0, y0, PI / 2.0), (x1, y0, PI), (x1, y1, 3.0 * PI / 2.0), (x0, y1, 0.0)];
    let mut out: Vec<Pt> = Vec::new();
    for (i, q) in corners.iter().enumerate() {
        for s in 0..=4 {
            out.push(g::polar([q.0, q.1], q.2 - s as f64 / 4.0 * PI / 2.0, c));
        }
        let nx = corners[(i + 1) % 4];
        let (a, b) = (out[out.len() - 1], g::polar([nx.0, nx.1], nx.2, c));
        for s in 1..n {
            out.push(g::mix(a, b, s as f64 / n as f64));
        }
    }
    out
}

pub fn cloud_puffs(rng: &mut Rng, x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 3]> {
    let w = x1 - x0;
    let mut puffs = Vec::new();
    let banks = rng.int(4, 6);
    for _ in 0..banks {
        let mut cx = rng.range(x0 - w * 0.2, x1);
        let mut cy = rng.range(y0, y1);
        let tilt = rng.range(-0.25, 0.25);
        let size = rng.range(0.11, 0.2) * w;
        let m = rng.int(4, 8);
        for _ in 0..m {
            puffs.push([cx, cy, size * rng.range(0.6, 1.2)]);
            cx += size * rng.range(0.6, 1.1);
            cy += size * (tilt + rng.jit(0.3));
        }
    }
    puffs
}

fn cover(puffs: &[[f64; 3]], x: f64, y: f64) -> f64 {
    let mut c: f64 = 0.0;
    for p in puffs {
        let (dx, dy) = ((x - p[0]) / (p[2] * 1.6), (y - p[1]) / (p[2] * 0.85));
        c = c.max(1.0 - g::smoothstep(0.35, 1.0, (dx * dx + dy * dy).sqrt()));
    }
    c
}

pub fn starfield(pt: &mut Paint, rng: &mut Rng, puffs: &[[f64; 3]], x0: f64, y0: f64, x1: f64, y1: f64) {
    let (mut d1, mut d2, mut twinkles) = (Vec::new(), Vec::new(), Vec::new());
    let n = ((x1 - x0) * (y1 - y0) * 0.09).floor() as i64;
    for _ in 0..n {
        let x = rng.range(x0, x1);
        let y = rng.range(y0, y1);
        let seen = 1.0 - cover(puffs, x, y);
        if seen > 0.15 {
            let bright = rng.next().powf(3.0);
            if bright > 0.8 && seen > 0.5 {
                twinkles.push((0.85 * seen, sparkle(x, y, 0.7 + bright * 0.8)));
            } else if seen > 0.6 {
                d1.push(vec![Cmd::O(x, y, 0.14 + bright * 0.4)]);
            } else {
                d2.push(vec![Cmd::O(x, y, 0.14 + bright * 0.4)]);
            }
        }
    }
    if !d1.is_empty() {
        pt.fill_many(&d1, paint::WHITE, 0.85, NONE);
    }
    if !d2.is_empty() {
        pt.fill_many(&d2, paint::WHITE, 0.4, NONE);
    }
    for (op, path) in &twinkles {
        pt.fill(path, paint::WHITE, *op, NONE);
    }
}

pub fn ellipse(x: f64, y: f64, rx: f64, ry: f64) -> Vec<Cmd> {
    let pts: Vec<Pt> = (0..16)
        .map(|kk| {
            let a = kk as f64 * PI / 8.0;
            [x + rx * a.cos(), y + ry * a.sin()]
        })
        .collect();
    g::smooth(&pts, true)
}

const CLOUD_LAYERS: i64 = 6;

pub fn clouds(pt: &mut Paint, puffs: &[[f64; 3]], sky: Rgb, deep: Rgb) {
    let shadow: Vec<Vec<Cmd>> = puffs.iter().map(|p| ellipse(p[0], p[1] - p[2] * 0.35, p[2] * 1.5, p[2] * 0.6)).collect();
    pt.fill_many(&shadow, deep, 0.1, NONE);
    let tint = paint::mix(paint::WHITE, sky, 0.25);
    for kk in 1..=CLOUD_LAYERS {
        let sc = 1.5 - kk as f64 / CLOUD_LAYERS as f64;
        let layer: Vec<Vec<Cmd>> = puffs.iter().map(|p| ellipse(p[0], p[1] + p[2] * 0.06 * kk as f64, p[2] * 1.6 * sc, p[2] * 0.85 * sc)).collect();
        pt.fill_many(&layer, if kk > CLOUD_LAYERS - 2 { paint::WHITE } else { tint }, 0.09, NONE);
    }
}

const GLOW: Rgb = [0.99, 0.86, 0.72];

pub fn lit_star(pt: &mut Paint, cx: f64, cy: f64, rx: f64, ry: f64) {
    let c = [cx, cy];
    let r = rx.min(ry);
    let light = paint::mix(paint::WHITE, paint::SKY, 0.18);
    let (mut rays, mut haze, mut fine) = (Vec::new(), Vec::new(), Vec::new());
    for kk in 0..16 {
        let a = kk as f64 * PI / 8.0;
        let (len, half, is_ray) = if kk % 4 == 0 {
            (if kk % 8 == 0 { rx } else { ry }, r * 0.045, true)
        } else if kk % 2 == 0 {
            (r * 0.82, r * 0.04, true)
        } else {
            (r * 0.4, r * 0.022, false)
        };
        let p = g::polyline(&[c, g::polar(c, a + PI / 2.0, half), g::polar(c, a, len), g::polar(c, a - PI / 2.0, half)], true);
        if is_ray {
            rays.push(p);
            haze.push(g::polyline(&[c, g::polar(c, a + PI / 2.0, half * 3.0), g::polar(c, a, len * 0.8), g::polar(c, a - PI / 2.0, half * 3.0)], true));
        } else {
            fine.push(p);
        }
    }
    pt.fill_many(&haze, paint::WHITE, 0.12, NONE);
    pt.fill_many(&fine, light, 0.55, NONE);
    pt.fill_many(&rays, light, 0.9, NONE);
    for (i, rr) in [0.28, 0.2, 0.13, 0.07].iter().enumerate() {
        let kk = i + 1;
        pt.fill(&[Cmd::O(cx, cy, r * rr)], if kk == 4 { paint::CREAM } else { GLOW }, 0.2 + 0.18 * kk as f64, NONE);
    }
}

pub struct Field {
    pub deep: Rgb,
    pub dilute: Rgb,
    pub ink: Rgb,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub c: f64,
    pub frame: Vec<Cmd>,
    pub body: Vec<Pt>,
}

#[derive(Clone, Copy, Default)]
pub struct Sky {
    pub pigment: Option<Rgb>,
    pub pool: Option<Rgb>,
    pub light: Option<Pt>,
    pub strength: Option<f64>,
}

pub fn panel_open(pt: &mut Paint, rng: &Rng, x: f64, y: f64, w: f64, h: f64, sky: Sky) -> Field {
    let tint = color::part_ink(Some("cartouche"));
    let (deep, dilute, ink) = (color::recolor(paint::DEEP, tint), color::recolor(paint::SKY, tint), color::recolor(paint::INK, tint));
    let blue = color::recolor(paint::BLUE, tint);
    let band = g::clamp(w * 0.065, 1.6, 3.0);
    let (x0, y0, x1, y1) = (x + band, y + band, x + w - band, y + h - band);
    let c = w.min(h) * 0.09;
    let frame = g::polyline(&[[x, y], [x + w, y], [x + w, y + h], [x, y + h]], true);
    pt.fill(&frame, paint::mix(blue, paint::PAPER, 0.86), 1.0, NONE);
    let body = notched(x0, y0, x1, y1, c, 6);
    let mut r = rng.fork("sky");
    wash::wash(
        pt,
        &body,
        &mut r,
        WashOpts {
            size: w.min(h),
            pigment: Some(sky.pigment.unwrap_or(paint::BLUE)),
            pool: Some(sky.pool.unwrap_or(paint::DEEP)),
            ink: tint,
            light: Some(sky.light.unwrap_or([(x0 + x1) / 2.0, (y0 + y1) / 2.0])),
            strength: Some(sky.strength.unwrap_or(1.0)),
            rings: Some(10),
            lift: Some(0.022),
            grain: 0.04,
            lost: Some(0.0),
            lifts: Some(0.0),
            ..Default::default()
        },
    );
    Field { deep, dilute, ink, x0, y0, x1, y1, c, frame, body }
}

pub fn panel_close(pt: &mut Paint, f: &Field) {
    pt.stroke(&g::polyline(&f.body, true), paint::mix(f.deep, paint::PAPER, 0.35), 0.35, 0.8, NONE);
    let curls: Vec<Vec<Cmd>> = [(f.x0, f.y0, 0.0), (f.x1, f.y0, PI / 2.0), (f.x1, f.y1, PI), (f.x0, f.y1, 3.0 * PI / 2.0)]
        .iter()
        .map(|q| {
            let pts: Vec<Pt> = (0..=6).map(|s| g::polar([q.0, q.1], q.2 + s as f64 / 6.0 * PI / 2.0, f.c * 0.55)).collect();
            g::polyline(&pts, false)
        })
        .collect();
    pt.stroke_many(&curls, paint::WHITE, 0.35, 0.9, NONE);
    pt.stroke(&f.frame, paint::mix(f.ink, paint::PAPER, 0.25), 0.45, 0.9, NONE);
}

pub fn panel(pt: &mut Paint, rng: &Rng, x: f64, y: f64, w: f64, h: f64) {
    let f = panel_open(pt, rng, x, y, w, h, Sky::default());
    let puffs = cloud_puffs(&mut rng.fork("clouds"), f.x0, f.y0, f.x1, f.y1);
    pt.clip_begin(&g::polyline(&f.body, true));
    starfield(pt, &mut rng.fork("stars"), &puffs, f.x0, f.y0, f.x1, f.y1);
    clouds(pt, &puffs, f.dilute, f.deep);
    pt.clip_end();
    panel_close(pt, &f);
    lit_star(pt, (f.x0 + f.x1) / 2.0, (f.y0 + f.y1) / 2.0, (f.x1 - f.x0) * 0.47, (f.y1 - f.y0) * 0.47);
}

pub fn pendant(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, w: f64, h: f64, cord: f64) {
    let shape = rng.weighted(&[("ogee", 3.0), ("lozenge", 2.0), ("drop", 2.0), ("roundel", 1.5)]).unwrap();
    let knot_kind = rng.weighted(&[("interlace", 3.0), ("quatrefoil", 2.0), ("cross", 2.0)]).unwrap();
    let (hw, hh) = (w / 2.0, h / 2.0 - 2.2);
    let cy = y + 1.1;
    let half: Vec<Pt> = match shape {
        "ogee" => vec![[0.0, 1.0], [0.62, 0.86], [1.0, 0.42], [0.86, -0.12], [0.36, -0.62], [0.0, -1.0]],
        "lozenge" => vec![[0.0, 1.0], [0.5, 0.48], [1.0, 0.0], [0.5, -0.48], [0.0, -1.0]],
        "drop" => vec![[0.0, 1.0], [0.8, 0.72], [1.0, 0.2], [0.7, -0.4], [0.0, -1.0]],
        _ => (0..=8).map(|kk| [(kk as f64 * PI / 8.0).sin(), (kk as f64 * PI / 8.0).cos()]).collect(),
    };
    let outline = |sc: f64| -> Vec<Pt> {
        let mut pts: Vec<Pt> = half.iter().map(|q| [x + q[0] * hw * sc, cy + q[1] * hh * sc]).collect();
        for i in (1..half.len() - 1).rev() {
            pts.push([x - half[i][0] * hw * sc, cy + half[i][1] * hh * sc]);
        }
        pts
    };
    pt.group_begin(0.7);
    let path = |pts: Vec<Pt>| if shape != "lozenge" { g::smooth(&pts, true) } else { g::polyline(&pts, true) };
    pt.stroke_many(&[path(outline(0.96)), path(outline(0.8))], paint::GOLD, 0.32, 0.85, NONE);
    let (mut paths, mut dots) = (Vec::new(), Vec::new());
    let u = hw.min(hh);
    if knot_kind == "interlace" {
        paths.push(ellipse(x, cy, u * 0.3, hh * 0.5));
        paths.push(ellipse(x, cy, hw * 0.5, u * 0.3));
        paths.push(vec![Cmd::O(x, cy, u * 0.14)]);
    } else if knot_kind == "quatrefoil" {
        for d in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            paths.push(vec![Cmd::O(x + d[0] * u * 0.26, cy + d[1] * u * 0.26, u * 0.24)]);
        }
        dots.push(vec![Cmd::O(x, cy, u * 0.08)]);
    } else {
        paths.push(g::polyline(&[[x, cy + hh * 0.6], [x, cy - hh * 0.6]], false));
        paths.push(g::polyline(&[[x - hw * 0.55, cy], [x + hw * 0.55, cy]], false));
        paths.push(g::polyline(&[[x, cy + u * 0.2], [x + u * 0.2, cy], [x, cy - u * 0.2], [x - u * 0.2, cy]], true));
        for q in [[0.0, hh * 0.6], [0.0, -hh * 0.6], [hw * 0.55, 0.0], [-hw * 0.55, 0.0]] {
            dots.push(vec![Cmd::O(x + q[0], cy + q[1], 0.6)]);
        }
    }
    pt.stroke_many(&paths, paint::GOLD, 0.28, 0.85, NONE);
    dots.push(vec![Cmd::O(x, y - h / 2.0 + 0.9, 0.9)]);
    let top = cy + hh * 0.96;
    if cord > 0.0 {
        pt.stroke(&g::polyline(&[[x, top], [x, top + cord]], false), paint::GOLD, 0.25, 0.8, NONE);
        let beads = rng.int(1, 3);
        for kk in 1..=beads {
            dots.push(vec![Cmd::O(x, top + cord * kk as f64 / (beads + 1) as f64, 0.75)]);
        }
    }
    pt.fill_many(&dots, paint::GOLD, 0.9, NONE);
    pt.group_end();
}
