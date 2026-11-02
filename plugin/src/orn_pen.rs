//! The pen behind the penwork ornaments: clothoid paths and pressure-shaded strokes.

use crate::geom::{self as g, Pt};
use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum Shape {
    C,
    In,
    Out,
    Bell,
}

fn shape_of(s: Shape) -> (fn(f64) -> f64, f64) {
    match s {
        Shape::C => (|_| 1.0, 1.0),
        Shape::In => (|t| t * t, 1.0 / 3.0),
        Shape::Out => (|t| (1.0 - t).powf(2.0), 1.0 / 3.0),
        Shape::Bell => (|t| (PI * t).sin(), 2.0 / PI),
    }
}

pub fn path(p: Pt, a: f64, segs: &[(f64, f64, Shape)]) -> (Vec<Pt>, f64) {
    let mut pts = vec![p];
    let mut q = p;
    let mut a = a;
    for s in segs {
        let (f, mean) = shape_of(s.2);
        let kmax = 2.0 * PI * s.1 / (s.0 * mean);
        let n = 12f64.max((s.1.abs() * 72.0).ceil()).max((s.0 * 40.0).ceil()) as usize;
        let (seg, na) = g::flow(q, a, s.0, |t| kmax * f(t), n);
        a = na;
        pts.extend_from_slice(&seg[1..]);
        q = seg[seg.len() - 1];
    }
    (pts, a)
}

pub fn looped(p: Pt, a: f64, rx: f64, ry: f64, drift: f64, side: f64) -> Vec<Pt> {
    let n = 72;
    (0..=n)
        .map(|i| {
            let th = -PI / 2.0 + 2.0 * PI * i as f64 / n as f64;
            let x = rx * th.cos() + drift * i as f64 / n as f64;
            let y = side * ry * (th.sin() + 1.0);
            [p[0] + x * a.cos() - y * a.sin(), p[1] + x * a.sin() + y * a.cos()]
        })
        .collect()
}

pub fn sprig(p: Pt, a: f64, s: f64, dir: f64, turns: f64) -> Vec<Pt> {
    path(p, a, &[(s * 0.35, dir * 0.06, Shape::C), (s * 0.65, dir * turns, Shape::In)]).0
}

pub fn echo(pts: &[Pt], d: f64) -> Vec<Pt> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let a = g::angle(pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]);
            g::polar(pts[i], a + PI / 2.0, d)
        })
        .collect()
}

pub fn mirror(pts: &[Pt], cx: f64, d: f64) -> Vec<Pt> {
    if d == 1.0 {
        return pts.to_vec();
    }
    pts.iter().map(|p| [2.0 * cx - p[0], p[1]]).collect()
}

pub fn rotate(pts: &[Pt], cx: f64, cy: f64) -> Vec<Pt> {
    pts.iter().map(|p| [2.0 * cx - p[0], 2.0 * cy - p[1]]).collect()
}

pub fn extent(lists: &[&[Pt]]) -> [f64; 4] {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for pts in lists {
        for p in *pts {
            b = [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])];
        }
    }
    b
}

/// Scale point lists to fit a box, centred. Returns (ox, oy, scale).
pub fn fit(ext: [f64; 4], b: [f64; 4], pad: f64) -> (f64, f64, f64) {
    let (w, h) = (ext[2] - ext[0], ext[3] - ext[1]);
    let s = ((b[2] - b[0] - 2.0 * pad) / w.max(1e-6)).min((b[3] - b[1] - 2.0 * pad) / h.max(1e-6));
    let ox = (b[0] + b[2]) / 2.0 - s * (ext[0] + ext[2]) / 2.0;
    let oy = (b[1] + b[3]) / 2.0 - s * (ext[1] + ext[3]) / 2.0;
    (ox, oy, s)
}

pub fn apply_fit(f: (f64, f64, f64), pts: &[Pt]) -> Vec<Pt> {
    pts.iter().map(|p| [f.0 + f.2 * p[0], f.1 + f.2 * p[1]]).collect()
}

fn smooth(v: &[f64], r: i64) -> Vec<f64> {
    let n = v.len() as i64;
    (0..n)
        .map(|i| {
            let (mut sum, mut c) = (0.0, 0.0);
            for j in (i - r).max(0)..=(i + r).min(n - 1) {
                sum += v[j as usize];
                c += 1.0;
            }
            sum / c
        })
        .collect()
}

#[derive(Clone, Copy)]
pub struct Pointed {
    pub hair: f64,
    pub shade: f64,
    pub down: f64,
    pub t0: f64,
    pub t1: f64,
    pub step: f64,
}

impl Default for Pointed {
    fn default() -> Self {
        Pointed { hair: 0.18, shade: 1.2, down: -PI / 2.0, t0: 0.06, t1: 0.06, step: 0.5 }
    }
}

pub fn pointed(pts: &[Pt], o: Pointed) -> Vec<Pt> {
    let n = 8f64.max((g::length(pts) / o.step).ceil()) as usize;
    let q = g::resample(pts, n);
    let (dx, dy) = (o.down.cos(), o.down.sin());
    let s: Vec<f64> = (0..n)
        .map(|i| {
            let (a, b) = (q[i.saturating_sub(1)], q[(i + 1).min(n - 1)]);
            let (tx, ty) = (b[0] - a[0], b[1] - a[1]);
            let l = (tx * tx + ty * ty).sqrt();
            let pull = if l > 0.0 { (tx * dx + ty * dy) / l } else { 0.0 };
            pull.max(0.0).powf(1.6)
        })
        .collect();
    let s = smooth(&s, 2i64.max((n as f64 * 0.025).floor() as i64));
    let w: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / (n as f64 - 1.0);
            let env = g::smoothstep(0.0, o.t0.max(1e-6), t) * g::smoothstep(0.0, o.t1.max(1e-6), 1.0 - t);
            o.hair * (0.55 + 0.45 * env) + (o.shade - o.hair) * s[i] * env
        })
        .collect();
    let last = n as f64 - 1.0;
    g::ribbon(&q, |t| w[(t * last + 0.5).floor() as usize])
}

#[derive(Clone, Copy)]
pub struct Broad {
    pub hair: f64,
    pub nib: f64,
    pub angle: f64,
    pub step: f64,
}

impl Default for Broad {
    fn default() -> Self {
        Broad { hair: 0.15, nib: 1.0, angle: PI / 6.0, step: 0.5 }
    }
}

pub fn broad(pts: &[Pt], o: Broad) -> Vec<Pt> {
    let n = 8f64.max((g::length(pts) / o.step).ceil()) as usize;
    let q = g::resample(pts, n);
    let w: Vec<f64> = (0..n)
        .map(|i| {
            let (a, b) = (q[i.saturating_sub(1)], q[(i + 1).min(n - 1)]);
            let t = i as f64 / (n as f64 - 1.0);
            let env = g::smoothstep(0.0, 0.05, t) * g::smoothstep(0.0, 0.05, 1.0 - t);
            o.hair + (o.nib - o.hair) * (g::angle(a, b) - o.angle).sin().abs() * env
        })
        .collect();
    let last = n as f64 - 1.0;
    g::ribbon(&q, |t| w[(t * last + 0.5).floor() as usize])
}

pub fn hairline(pts: &[Pt], w: f64, finish: f64) -> Vec<Pt> {
    let n = 6f64.max((g::length(pts) / 0.6).ceil()) as usize;
    let q = g::resample(pts, n);
    g::ribbon(&q, |t| w * (1.0 - (1.0 - finish) * t))
}
