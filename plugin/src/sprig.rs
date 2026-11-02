//! Line-work plants: stems, tendrils, umbels, cymes and baby's-breath sprays.

use crate::color::{self, Rgb};
use crate::dist;
use crate::flower::{self, Floret};
use crate::geom::{self as g, Cmd, Pt};
use crate::paint::{self, Paint, MULT};
use crate::rng::Rng;
use crate::tune::tune;
use std::f64::consts::PI;

#[derive(Default)]
pub struct Batch {
    pub ribbons: Vec<Vec<Cmd>>,
    pub lines: [Vec<Vec<Cmd>>; 3],
    pub florets: Vec<Floret>,
    pub buds: Vec<[f64; 4]>,
    pub stars: Vec<[f64; 3]>,
}

impl Batch {
    pub fn stem(&mut self, pts: &[Pt], w0: f64, w1: f64) {
        self.ribbons.push(g::smooth(&g::ribbon(pts, |t| (w0 + (w1 - w0) * t) / 2.0), true));
    }

    pub fn stem_by(&mut self, pts: &[Pt], w: &dyn Fn(f64) -> f64) {
        self.ribbons.push(g::smooth(&g::ribbon(pts, |t| w(t) / 2.0), true));
    }

    pub fn line(&mut self, pts: &[Pt], class: usize) {
        self.lines[class - 1].push(g::smooth(pts, false));
    }

    pub fn floret(&mut self, x: f64, y: f64, r: f64, rot: f64, tint: usize, soft: bool) {
        self.florets.push(Floret { x, y, r, rot, tint, soft });
    }

    pub fn bud(&mut self, x: f64, y: f64, ang: f64, len: f64) {
        self.buds.push([x, y, ang, len]);
    }

    /// Does every point lie where `ok(x, y, r)` holds?
    pub fn all_points(&self, mut ok: impl FnMut(f64, f64, f64) -> bool) -> bool {
        let list = |l: &[Vec<Cmd>], ok: &mut dyn FnMut(f64, f64, f64) -> bool| -> bool {
            for c in l {
                for op in c {
                    let pass = match *op {
                        Cmd::M(x, y) | Cmd::L(x, y) => ok(x, y, 0.0),
                        Cmd::C(_, _, _, _, e, f) => ok(e, f, 0.0),
                        _ => true,
                    };
                    if !pass {
                        return false;
                    }
                }
            }
            true
        };
        if !list(&self.ribbons, &mut ok) {
            return false;
        }
        for i in 0..3 {
            if !list(&self.lines[i], &mut ok) {
                return false;
            }
        }
        self.florets.iter().all(|f| ok(f.x, f.y, f.r)) && self.buds.iter().all(|b| ok(b[0], b[1], b[3])) && self.stars.iter().all(|s| ok(s[0], s[1], s[2]))
    }

    pub fn merge(&mut self, other: Batch) {
        self.ribbons.extend(other.ribbons);
        let [a, b, c] = other.lines;
        self.lines[0].extend(a);
        self.lines[1].extend(b);
        self.lines[2].extend(c);
        self.florets.extend(other.florets);
        self.buds.extend(other.buds);
        self.stars.extend(other.stars);
    }

    pub fn paint(&self, pt: &mut Paint, ink: Option<Rgb>) {
        let tint = color::part_ink(Some("sprays"));
        let ink = color::recolor(ink.unwrap_or(paint::INK), tint);
        if !self.ribbons.is_empty() {
            pt.fill_many(&self.ribbons, ink, 0.92, MULT);
        }
        let widths = [0.32, 0.22, 0.14];
        for i in 0..3 {
            if !self.lines[i].is_empty() {
                pt.stroke_many(&self.lines[i], ink, widths[i], 0.85, MULT);
            }
        }
        for b in &self.buds {
            let c = if tint.is_some() { color::recolor(paint::DEEP, tint) } else { paint::DEEP };
            flower::bud(pt, b[0], b[1], b[2], b[3], c);
        }
        if !self.florets.is_empty() {
            flower::florets(pt, &self.florets);
        }
        if !self.stars.is_empty() {
            let paths: Vec<Vec<Cmd>> = self
                .stars
                .iter()
                .map(|s| {
                    let (x, y, r) = (s[0], s[1], s[2]);
                    let k = r * 0.22;
                    vec![
                        Cmd::M(x, y + r), Cmd::L(x + k, y + k), Cmd::L(x + r, y), Cmd::L(x + k, y - k),
                        Cmd::L(x, y - r), Cmd::L(x - k, y - k), Cmd::L(x - r, y), Cmd::L(x - k, y + k), Cmd::Z,
                    ]
                })
                .collect();
            pt.fill_many(&paths, color::recolor(paint::DEEP, tint), 0.85, MULT);
        }
    }
}

pub fn tendril(p: Pt, ang: f64, len: f64, dir: f64) -> Vec<Pt> {
    let c = g::curl_coefficient(len, 1.05, 2.5);
    g::flow(p, ang, len, |t| dir * (0.25 / len + c * t.powf(2.5)).min(0.25), 0).0
}

pub fn flourish(p: Pt, ang: f64, len: f64, dir: f64) -> Vec<Pt> {
    let split = 0.55;
    let looped = 2.0 * PI * 1.05 * 2.0 / (split * len);
    let curl = g::curl_coefficient((1.0 - split) * len, 1.2, 2.5);
    g::flow(
        p,
        ang,
        len,
        |t| {
            if t < split {
                dir * looped * (PI * t / split).sin().powf(2.0)
            } else {
                -dir * (curl * ((t - split) / (1.0 - split)).powf(2.5)).min(0.25)
            }
        },
        0,
    )
    .0
}

pub fn umbel(b: &mut Batch, rng: &mut Rng, base: Pt, ang: f64, r0: f64) {
    let m = rng.int(13, 19);
    let spread = rng.range(2.0, 2.6);
    for j in 1..=m {
        let u = (j as f64 - 0.5) / m as f64;
        let a = ang - spread / 2.0 + spread * u + rng.jit(0.05);
        let len = r0 * (0.72 + 0.28 * ((u - 0.5) * PI).cos()) * rng.range(0.9, 1.05);
        let ray = g::curve(base, a, len, (u - 0.5) * 0.35, 0.0, 6);
        b.line(&ray, 3);
        let tip = ray[ray.len() - 1];
        let k = rng.int(4, 7);
        for i in 1..=k {
            let a2 = a - 1.1 + 2.2 * (i as f64 - 0.5) / k as f64 + rng.jit(0.08);
            let l = r0 * rng.range(0.13, 0.22);
            let bend = rng.jit(0.3);
            let sub = g::curve(tip, a2, l, bend, 0.0, 3);
            b.line(&sub, 3);
            let e = sub[sub.len() - 1];
            let fr = r0 * rng.range(0.020, 0.030);
            let rot = rng.range(0.0, 2.0 * PI);
            let tint = if rng.chance(0.6) { 2 } else { 3 };
            b.floret(e[0], e[1], fr, rot, tint, false);
        }
    }
}

pub fn cyme(b: &mut Batch, rng: &mut Rng, base: Pt, ang: f64, len: f64, dir: f64) -> Vec<Pt> {
    let axis = tendril(base, ang, len, dir);
    b.line(&axis, 2);
    let n = axis.len();
    let count = rng.int(7, 11);
    for i in 1..=count {
        let t = (i as f64 - 0.5) / count as f64;
        let idx = 2.max((t * (n as f64 - 1.0)).floor() as usize + 1);
        let (p, q) = (axis[idx - 2], axis[idx - 1]);
        let a = g::angle(p, q) + (if i % 2 == 0 { 1.0 } else { -1.0 }) * rng.range(0.6, 1.1);
        let size = len * 0.075 * (1.15 - 0.8 * t);
        let bend = rng.jit(0.4);
        let ped = g::curve(q, a, size * 1.4, bend, 0.0, 3);
        b.line(&ped, 3);
        let e = ped[ped.len() - 1];
        if t < 0.72 {
            let rot = rng.range(0.0, 2.0 * PI);
            let tint = if rng.chance(0.35) { 3 } else { 2 };
            b.floret(e[0], e[1], size, rot, tint, false);
        } else {
            b.bud(e[0], e[1], a, size * 1.5);
        }
    }
    axis
}

fn branch(b: &mut Batch, rng: &mut Rng, depth: i64, p: Pt, a: f64, l: f64, d: i64) {
    let bend = rng.jit(0.3);
    let pts = g::curve(p, a, l, bend, 0.0, 4);
    b.line(&pts, if d >= depth - 1 { 2 } else { 3 });
    let e = pts[pts.len() - 1];
    let ea = g::angle(pts[pts.len() - 2], e);
    if d == 0 {
        let n = if rng.chance(0.7) { 1 } else { 2 };
        for k in 1..=n {
            let q = if k == 1 {
                e
            } else {
                let dir = ea + rng.jit(1.6);
                g::polar(e, dir, rng.range(2.8, 3.8))
            };
            if rng.chance(0.08) {
                let r = rng.range(0.7, 1.1);
                b.stars.push([q[0], q[1], r]);
            } else {
                let r = rng.range(0.9, 1.5);
                let rot = rng.range(0.0, 2.0 * PI);
                let tint = rng.weighted(&[("1", 0.4), ("2", 3.0), ("3", 5.0)]).unwrap();
                b.floret(q[0], q[1], r, rot, tint.parse().unwrap(), true);
            }
        }
        return;
    }
    let spread = 0.25 + 0.45 * (1.0 - d as f64 / depth as f64);
    let kids = if rng.chance(0.4) { 3 } else { 2 };
    for k in 1..=kids {
        let off = if kids == 2 {
            (if k == 1 { -1.0 } else { 1.0 }) * spread * rng.range(0.7, 1.2)
        } else {
            (k - 2) as f64 * spread * rng.range(0.8, 1.2)
        };
        let nl = l * rng.range(0.55, 0.72);
        branch(b, rng, depth, e, ea + off, nl, d - 1);
    }
    if d <= 2 && rng.chance(0.5) {
        let dir = ea + rng.jit(1.2);
        let q = g::polar(g::mix(p, e, 0.6), dir, rng.range(1.5, 2.5));
        let r = rng.range(0.5, 0.8);
        b.floret(q[0], q[1], r, 0.0, 3, true);
    }
}

pub fn gypsophila(b: &mut Batch, rng: &mut Rng, base: Pt, ang: f64, len: f64, depth: i64) {
    branch(b, rng, depth, base, ang, len, depth);
}

pub fn leaflet_items(rng: &mut Rng, pts: &[Pt], every: [f64; 2], len: f64) -> Vec<[f64; 4]> {
    let mut items = Vec::new();
    let total = g::length(pts);
    let (lo, hi) = (every[0], every[1]);
    let mut d = 0.0;
    let mut i = 0;
    loop {
        d += rng.range(lo, hi);
        if d > total - lo * 0.3 {
            break;
        }
        i += 1;
        let (p, a) = g::at(pts, (d + rng.jit(0.2) * lo) / total);
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let ang = a + side * rng.range(0.5, 0.9);
        let size = len * dist::sample(rng, &tune().sizes.leaflet);
        items.push([p[0], p[1], ang, size]);
    }
    items
}
