//! Page planning: the growing region, the plant's strands and the gesture.

use crate::compose::{Bx, Spec};
use crate::geom::{self as g, Pt};
use crate::lua::sort;
use crate::plant::{self, Box4, Stem};
use crate::rng::Rng;
use crate::space::Space;
use crate::sprig;
use crate::tune::tune;
use std::f64::consts::PI;

pub struct Margin {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub foot: f64,
}

pub fn region(spec: &Spec) -> (Space, Margin) {
    let (w, h, t) = (spec.width, spec.height, spec.text);
    let mut s = Space::new(w, h, 3.0);
    let edge = spec.edge;
    let gutter = 9.0;
    let m = Margin { x0: edge, x1: t.x0 - gutter, y0: edge + 4.0, y1: h - edge - 6.0, foot: t.y0 - gutter * 0.8 };
    s.open(m.x0, m.y0, m.x1, m.y1);
    s.open(m.x0, m.y0, w - edge, m.foot);
    s.open(t.x1 + 5.0, m.y0, w - edge * 0.6, t.y0 + (t.y1 - t.y0) * 0.22);
    if let Some(lines) = &spec.lines {
        if !lines.is_empty() {
            if !spec.frame_only {
                open_short_lines(&mut s, spec, lines, gutter);
            }
            for l in lines {
                s.block_text(l.x0 - 4.0, l.y0 - 3.0, l.x1 + 4.0, l.y1 + 3.0);
            }
        }
    }
    if let Some(hy) = spec.header_y {
        s.block_text(t.x0 - 4.0, hy - 4.0, t.x1, hy + 12.0);
    }
    for r in &spec.closed {
        s.close(r[0], r[1], r[2], r[3]);
    }
    (s, m)
}

const CLEARING: f64 = 40.0;

fn open_short_lines(s: &mut Space, spec: &Spec, all: &[Bx], gutter: f64) {
    let (t, pad) = (spec.text, 5.0);
    let mut lines: Vec<Bx> = all.iter().copied().filter(|l| l.x1 > l.x0 && l.y1 > t.y0 - 2.0).collect();
    if lines.is_empty() {
        return;
    }
    sort(&mut lines, |a, b| a.y1 > b.y1);
    let lowest = lines[lines.len() - 1].y0;
    let cap = t.x0 + (t.x1 - t.x0) * 0.42;
    let mut y = t.y0;
    while y <= t.y1 {
        let reach;
        if y < lowest - 14.0 {
            reach = t.x1;
        } else {
            let mut hit: Option<f64> = None;
            for l in &lines {
                if y + s.c > l.y0 - 1.0 && y < l.y1 + 1.0 {
                    hit = Some(hit.unwrap_or(f64::INFINITY).min(l.x0));
                }
            }
            match hit {
                Some(h) => reach = h.min(cap),
                None => {
                    let (mut above, mut below): (Option<Bx>, Option<Bx>) = (None, None);
                    for l in &lines {
                        if l.y0 >= y && above.map_or(true, |a| l.y0 < a.y0) {
                            above = Some(*l);
                        }
                        if l.y1 <= y && below.map_or(true, |b| l.y1 > b.y1) {
                            below = Some(*l);
                        }
                    }
                    let clearing = match (above, below) {
                        (Some(a), Some(b)) => a.y0 - b.y1 >= CLEARING && a.y0 - y >= 10.0 && y + s.c - b.y1 >= 10.0,
                        _ => false,
                    };
                    reach = if clearing { t.x1 + pad } else { above.map_or(t.x0, |a| a.x0).min(below.map_or(t.x0, |b| b.x0)).min(cap) };
                }
            }
        }
        if reach - pad > t.x0 - gutter {
            s.open(t.x0 - gutter, y, reach - pad, y + s.c);
        }
        y += s.c;
    }
}

pub fn cartouche_size(mw: f64, k: f64) -> (f64, f64) {
    (g::clamp(mw * 0.27 * k, 16.0, 48.0), g::clamp(mw * 0.365 * k, 22.0, 66.0))
}

#[derive(Clone)]
pub struct Bow {
    pub y: f64,
    pub amp: f64,
    pub h: f64,
}

#[derive(Clone)]
pub struct GestureField {
    pub edge: f64,
    pub bows: Vec<Bow>,
    pub quiet: Vec<f64>,
    clusters: Vec<[f64; 4]>,
    floor: f64,
}

impl GestureField {
    pub fn bow(&self, y: f64) -> f64 {
        let mut x = 0.0;
        for b in &self.bows {
            let u = (y - b.y) / b.h;
            if u.abs() < 1.0 {
                x += b.amp * 0.5 * (1.0 + (PI * u).cos());
            }
        }
        x
    }

    pub fn density(&self, x: f64, y: f64) -> f64 {
        let mut best: f64 = 0.0;
        for c in &self.clusters {
            let (dx, dy) = ((x - c[0]) / c[2], (y - c[1]) / c[2]);
            best = best.max(c[3] * (-(dx * dx + dy * dy)).exp());
        }
        self.floor + (1.0 - self.floor) * best
    }
}

pub fn gesture(rng: &mut Rng, m: &Margin, hero: [f64; 3]) -> GestureField {
    let cfg = &tune().gesture;
    let mw = m.x1 - m.x0;
    let lo = m.y0 + mw * 0.75 + 40.0;
    let hi = hero[1] - hero[2] * 1.2;
    let n = cfg.bows;
    let band = (hi - lo) / n;
    let (mut bows, mut quiet): (Vec<Bow>, Vec<f64>) = (Vec::new(), Vec::new());
    for i in 1..=(n as i64) {
        let y = lo + band * (i as f64 - 0.5 + rng.jit(0.18));
        let amp = mw * rng.rangev(cfg.bow_amp);
        let h = band * rng.rangev(cfg.bow_width);
        bows.push(Bow { y, amp, h });
        if i > 1 {
            quiet.push((bows[bows.len() - 2].y + y) / 2.0);
        }
    }
    let edge = m.x0 + mw * cfg.edge;
    let mut clusters = vec![
        [hero[0], hero[1], hero[2] * cfg.end_reach, 1.0],
        [m.x0 + mw * 0.6, m.y0 + 30.0, 110.0 * cfg.end_reach / 2.2, 1.0],
        [m.x0 + mw * 0.35, m.y0 + mw * 0.9, mw * 0.6, 1.0],
    ];
    for b in &bows {
        clusters.push([edge + b.amp * 0.7, b.y, band * 0.32, cfg.bow_density]);
    }
    GestureField { edge, bows, quiet, clusters, floor: cfg.density_floor }
}

pub fn strands(rng: &Rng, m: &Margin, hero: [f64; 3], w: f64, gest: &GestureField, avoid: Option<Box4>, natural: bool) -> (Vec<Stem>, Pt) {
    let (mut main, corner, _) = plant::main_stem(&mut rng.fork("main"), m, hero, w, gest, avoid);
    let long = if natural { Vec::new() } else { plant::long_shoots(&mut rng.fork("shoots"), &mut main, m, hero, gest, avoid) };
    let foot = plant::foot_shoots(&mut rng.fork("foot shoots"), &mut main, m, corner, avoid);
    let crown = plant::crown(&mut rng.fork("crown"), &mut main, m, hero);
    let mut out = vec![main];
    out.extend(long);
    out.extend(foot);
    out.extend(crown);
    (out, corner)
}

pub fn runners(m: &Margin, root: Pt, w: f64, edge: f64) -> Vec<Stem> {
    let cx = w - edge - 6.0;
    let b = [cx, m.y0 + 70.0];
    let mut stem = g::link(root, 0.05, b, PI / 2.0 + 0.08, 0.6, 0);
    let tip = sprig::tendril(b, PI / 2.0 + 0.08, 46.0, 1.0);
    stem.extend_from_slice(&tip[1..]);
    vec![Stem::plain(stem, |t| 0.8 - 0.6 * t)]
}

pub fn excursion(rng: &mut Rng, spec: &Spec, root: Pt) -> Option<Stem> {
    let t = spec.text;
    let mut lowest = t.y1;
    let lines = spec.lines.as_ref()?;
    for l in lines {
        if l.x1 > t.x0 && l.x0 < t.x1 && l.y0 > t.y0 - 2.0 && l.y1 < t.y1 + 2.0 {
            lowest = lowest.min(l.y0);
        }
    }
    let room = lowest - t.y0;
    if room < 140.0 {
        return None;
    }
    let w = t.x1 - t.x0;
    let ceiling = lowest - 30.0;
    let mut pts = vec![root, [root[0] + 22.0, root[1] + 4.0], [t.x0 - 4.0, root[1] + rng.range(18.0, 30.0)]];
    let n = 5;
    let phase = rng.range(0.0, 2.0 * PI);
    for i in 1..=n {
        let u = i as f64 / n as f64;
        let x = t.x0 + w * (0.08 + 0.78 * u);
        let y = t.y0 + room * (0.18 + 0.55 * u) + room * 0.10 * (phase + u * 5.0).sin();
        pts.push([x, y.min(ceiling)]);
    }
    Some(Stem::plain(g::spline_points(&pts, 6), |t| 1.3 - 0.9 * t))
}
