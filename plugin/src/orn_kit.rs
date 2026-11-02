//! Shared shapes for the ornament kinds. Angles are radians, y points up.

use crate::geom::{self as g, Cmd, Pt};
use crate::paint::{self, Paint, NONE};
use std::f64::consts::PI;
use std::rc::Rc;

/// Centre, width, height and smaller side of a box.
pub fn measure(b: [f64; 4]) -> (f64, f64, f64, f64, f64) {
    let (w, h) = (b[2] - b[0], b[3] - b[1]);
    ((b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, w, h, w.min(h))
}

pub fn dot(x: f64, y: f64, r: f64) -> Vec<Cmd> {
    vec![Cmd::O(x, y, r)]
}

pub fn ring(x: f64, y: f64, rr: f64, n: i64, r: f64, a0: f64) -> Vec<Vec<Cmd>> {
    (0..n)
        .map(|k| {
            let p = g::polar([x, y], a0 + k as f64 * 2.0 * PI / n as f64, rr);
            dot(p[0], p[1], r)
        })
        .collect()
}

pub fn ellipse(x: f64, y: f64, rx: f64, ry: f64, rot: f64, n: usize) -> Vec<Pt> {
    let (c, s) = (rot.cos(), rot.sin());
    (0..n)
        .map(|k| {
            let a = k as f64 * 2.0 * PI / n as f64;
            let (u, v) = (rx * a.cos(), ry * a.sin());
            [x + u * c - v * s, y + u * s + v * c]
        })
        .collect()
}

pub fn star(x: f64, y: f64, rr: f64, r: f64, n: i64, rot: f64) -> Vec<Pt> {
    (0..2 * n).map(|k| g::polar([x, y], rot + k as f64 * PI / n as f64, if k % 2 == 0 { rr } else { r })).collect()
}

#[derive(Clone, Copy)]
pub struct RayOpts {
    pub rot: f64,
    pub short: f64,
    pub width: f64,
    pub flames: bool,
}

impl Default for RayOpts {
    fn default() -> Self {
        RayOpts { rot: PI / 2.0, short: 0.62, width: 0.18, flames: false }
    }
}

pub fn rays(x: f64, y: f64, r0: f64, r1: f64, n: i64, o: RayOpts) -> (Vec<Vec<Cmd>>, Vec<Vec<Cmd>>) {
    let (mut lit, mut shade) = (Vec::new(), Vec::new());
    for k in 0..n {
        let a = o.rot + k as f64 * 2.0 * PI / n as f64;
        let long = k % 2 == 0;
        let len = if long { r1 } else { r0 + (r1 - r0) * o.short };
        let half = o.width * 2.0 * PI / n as f64 * r0;
        let base = g::polar([x, y], a, r0);
        let tip = g::polar([x, y], a, len);
        let (l, r) = (g::polar(base, a + PI / 2.0, half), g::polar(base, a - PI / 2.0, half));
        if o.flames && !long {
            let spine: Vec<Pt> = (0..=8)
                .map(|i| {
                    let t = i as f64 / 8.0;
                    let p = g::polar([x, y], a, r0 + (len - r0) * t);
                    g::polar(p, a + PI / 2.0, half * 0.7 * (t * 2.0 * PI).sin() * (1.0 - t))
                })
                .collect();
            let (mut left, mut right) = (Vec::new(), Vec::new());
            for (i, p) in spine.iter().enumerate() {
                let w = half * (1.0 - i as f64 / 8.0);
                left.push(g::polar(*p, a + PI / 2.0, w));
                right.push(g::polar(*p, a - PI / 2.0, w));
            }
            let mut poly = left;
            poly.extend(right.iter().rev());
            lit.push(g::smooth(&poly, true));
        } else {
            lit.push(g::polyline(&[base, tip, l], true));
            shade.push(g::polyline(&[base, tip, r], true));
        }
    }
    (lit, shade)
}

pub fn gilt(pt: &mut Paint, lit: &[Vec<Cmd>], shade: &[Vec<Cmd>], opacity: f64) {
    if !shade.is_empty() {
        pt.fill_many(shade, paint::COPPER, opacity, NONE);
    }
    if !lit.is_empty() {
        pt.fill_many(lit, paint::GOLD, opacity, NONE);
    }
}

pub fn crescent(x: f64, y: f64, r: f64, bite: f64, rot: f64) -> Vec<Pt> {
    let mut out = Vec::with_capacity(48);
    let d = r * bite;
    let m = g::clamp(d / (2.0 * r), -1.0, 1.0).acos();
    let n = 24;
    for k in 0..=n {
        let a = rot + m + (2.0 * PI - 2.0 * m) * k as f64 / n as f64;
        out.push(g::polar([x, y], a, r));
    }
    let c2 = [x + d * rot.cos(), y + d * rot.sin()];
    for k in 1..n {
        out.push(g::polar(c2, rot + PI + m - 2.0 * m * k as f64 / n as f64, r));
    }
    out
}

pub fn scroll(p: Pt, ang: f64, s: f64, dir: f64, turns: f64) -> Vec<Pt> {
    let n = 28;
    let mut out = Vec::with_capacity(n + 1);
    let (mut x, mut y, mut a) = (p[0], p[1], ang);
    for i in 0..=n {
        out.push([x, y]);
        let t = i as f64 / n as f64;
        let step = s * 0.11 * (1.0 - 0.82 * t);
        a += dir * turns * 2.0 * PI / n as f64 * (0.55 + 0.9 * t);
        x += step * a.cos();
        y += step * a.sin();
    }
    out
}

pub fn taper(pts: &[Pt], w: f64, tail: f64) -> Vec<Cmd> {
    g::smooth(&g::ribbon(pts, |t| w * (1.0 - (1.0 - tail) * t) / 2.0), true)
}

pub fn leaf(x: f64, y: f64, s: f64, ang: f64, wide: f64) -> Vec<Pt> {
    let pts: Vec<Pt> = (0..40)
        .map(|k| {
            let t = k as f64 * 2.0 * PI / 40.0;
            let hx = 16.0 * t.sin().powf(3.0);
            let hy = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
            [hx / 22.0 * 0.9, (5.0 - hy) / 22.0]
        })
        .collect();
    g::transform(&pts, x, y, ang - PI / 2.0, s * wide, s)
}

pub fn lozenge(x: f64, y: f64, s: f64, ang: f64) -> Vec<Pt> {
    let right: Vec<Pt> = (0..=12)
        .map(|k| {
            let t = k as f64 / 12.0;
            [0.34 * (PI * t).sin().powf(1.3) * (1.0 - 0.25 * t), t]
        })
        .collect();
    let mut pts = right.clone();
    for k in (1..right.len() - 1).rev() {
        pts.push([-right[k][0], right[k][1]]);
    }
    g::transform(&pts, x, y, ang - PI / 2.0, s, s)
}

pub fn trefoil(x: f64, y: f64, s: f64, ang: f64) -> Vec<Vec<Cmd>> {
    [-1.0, 0.0, 1.0]
        .iter()
        .map(|k: &f64| {
            let p = g::polar([x, y], ang + k * 2.0 * PI / 3.0, s * 0.42);
            dot(p[0], p[1], s * 0.36)
        })
        .collect()
}

pub fn armillary(pt: &mut Paint, x: f64, y: f64, r: f64, tilt: f64, floor: f64) {
    let rings = vec![
        g::smooth(&ellipse(x, y, r, r, 0.0, 40), true),
        g::smooth(&ellipse(x, y, r, r * 0.3, -tilt, 40), true),
        g::smooth(&ellipse(x, y, r * 0.32, r, 0.0, 40), true),
        g::smooth(&ellipse(x, y, r, r * 0.24, tilt * 0.6, 40), true),
    ];
    let w = 0.3f64.max(r * 0.025);
    pt.stroke_many(&rings, paint::COPPER, w * 1.7, 0.55, NONE);
    pt.stroke_many(&rings, paint::GOLD, w, 0.95, NONE);
    pt.stroke(&g::smooth(&ellipse(x, y, r * 0.93, r * 0.22, -tilt, 40), true), paint::GOLD, w * 0.55, 0.7, NONE);
    let a = PI / 2.0 - tilt * 0.6;
    pt.stroke(&g::polyline(&[g::polar([x, y], a, r * 1.22), g::polar([x, y], a + PI, r * 1.22)], false), paint::COPPER, w * 0.9, 0.9, NONE);
    pt.fill(&dot(x, y, r * 0.13), paint::GOLD, 0.95, NONE);
    pt.fill(&dot(x - r * 0.03, y + r * 0.03, r * 0.06), paint::CREAM, 0.9, NONE);
    let foot = floor.max(y - r * 1.45);
    pt.stroke(&g::polyline(&[[x, y - r], [x, foot]], false), paint::GOLD, w * 1.2, 0.9, NONE);
    pt.fill(&g::smooth(&ellipse(x, foot, r * 0.42, r * 0.09, 0.0, 20), true), paint::GOLD, 0.9, NONE);
}

/// Run `draw` for the lower-left corner of a box, mirrored into another corner.
pub fn corner(pt: &mut Paint, b: [f64; 4], flip: Option<[f64; 2]>, draw: impl FnOnce(&mut Paint, f64, f64, f64)) {
    let (cx, cy, _, _, s) = measure(b);
    let [fx, fy] = flip.unwrap_or([1.0, 1.0]);
    let base = pt.frame_fn.clone();
    let inner = base.clone();
    pt.frame(Some(Rc::new(move |x, y| {
        let (x, y) = (cx + fx * (x - cx), cy + fy * (y - cy));
        match &inner {
            Some(f) => f(x, y),
            None => (x, y),
        }
    })));
    draw(pt, cx - s / 2.0, cy - s / 2.0, s);
    pt.frame(base);
}
