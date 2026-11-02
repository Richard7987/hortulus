//! Spray helpers for growth, and sparkles in leftover space.

use crate::geom::{self as g, Pt};
use crate::ornament::Sparkle;
use crate::ornaments::{self, Slot};
use crate::mix::Weights;
use crate::rng::Rng;
use crate::space::Space;
use crate::sprig::{self, Batch};
use std::f64::consts::PI;

pub fn grow(b: &mut Batch, rng: &mut Rng, kind: &str, s: Pt, tangent: f64, d: f64, x: f64, y: f64, r: f64, scale: f64, lean: f64) -> Vec<Pt> {
    let target = [x, y];
    let chord = g::angle(s, target);
    let dir0 = g::depart(s, tangent, target, 0.35);
    let a = chord + g::turn(chord, dir0) * 0.6 + lean;
    let base = g::polar(s, chord, (d - r * 0.55).max(3.0));
    let stalk = g::link(s, dir0, base, a, 0.5, 0);
    b.line(&stalk, if d > 30.0 { 2 } else { 3 });
    let r = r * scale;
    match kind {
        "gyps" => sprig::gypsophila(b, rng, base, a, r * 0.5, if r >= 14.0 { 5 } else if r >= 8.0 { 4 } else { 3 }),
        "cyme" => {
            let dir = if rng.chance(0.5) { 1.0 } else { -1.0 };
            sprig::cyme(b, rng, base, a, r * 1.25, dir);
        }
        "umbel" => sprig::umbel(b, rng, base, a, r * 0.85),
        "bud" => {
            for k in [-1.0, 0.0, 1.0] {
                let bend = rng.jit(0.4);
                let stem = g::curve(base, a + k * 0.45, r * 0.6, bend, 0.0, 4);
                b.line(&stem, 3);
                let e = stem[stem.len() - 1];
                b.bud(e[0], e[1], a + k * 0.45, r * 0.32);
            }
        }
        _ => {
            let rot = rng.range(0.0, 2.0 * PI);
            let tint = rng.int(1, 3) as usize;
            b.floret(x, y, r * 0.35, rot, tint, false);
        }
    }
    stalk
}

pub fn fits(s: &Space, b: &Batch) -> bool {
    b.all_points(|x, y, r| s.open_at(x, y) && s.open_at(x - r, y) && s.open_at(x + r, y) && s.open_at(x, y - r) && s.open_at(x, y + r))
}

struct GiltPicker {
    old: Vec<(&'static str, f64)>,
    new: Vec<(&'static str, f64)>,
    so: f64,
    sn: f64,
    alt: Rng,
}

fn gilt_picker(rng: &Rng, weights: Option<&Weights>) -> Option<GiltPicker> {
    let (mut old, mut new, mut so, mut sn) = (Vec::new(), Vec::new(), 0.0, 0.0);
    for k in ornaments::of(Slot::Gilt) {
        let w = ornaments::weight(weights, k.name);
        if k.classic {
            old.push((k.sparkle, w));
            so += w;
        } else {
            new.push((k.sparkle, w));
            sn += w;
        }
    }
    if so + sn == 0.0 {
        return None;
    }
    Some(GiltPicker { old, new, so, sn, alt: rng.fork("gilt kinds") })
}

impl GiltPicker {
    fn pick(&mut self, rng: &mut Rng) -> &'static str {
        let mut kind = if self.so > 0.0 { rng.weighted(&self.old) } else { None };
        if self.sn > 0.0 && (kind.is_none() || self.alt.chance(self.sn / (self.so + self.sn))) {
            kind = self.alt.weighted(&self.new);
        }
        kind.unwrap_or("plain")
    }
}

pub fn sparkles(rng: &mut Rng, s: &mut Space, n: i64, weights: Option<&Weights>) -> Vec<Sparkle> {
    let cells = s.free_cells(2);
    let mut picker = gilt_picker(rng, weights);
    let mut items: Vec<Sparkle> = Vec::new();
    let put = |s: &mut Space, items: &mut Vec<Sparkle>, x: f64, y: f64, size: f64, kind: &'static str| {
        let reach = (if kind == "cross" { 1.55 } else { 1.0 }) * size;
        items.push(Sparkle { x, y, r: size, kind });
        s.reserve(x, y, reach * 2.2);
    };
    let nf = n as f64;
    let (mut big, mut tries) = (0.0, 0.0);
    while picker.is_some() && big < nf / 3.0 && tries < nf * 30.0 && !cells.is_empty() {
        tries += 1.0;
        let c = cells[(rng.int(1, cells.len() as i64) - 1) as usize];
        let r = s.clearance(c[0], c[1], 14.0);
        if r >= 9.0 {
            let size = rng.range(4.2, 7.5f64.min(r * 0.5));
            let kind = picker.as_mut().unwrap().pick(rng);
            put(s, &mut items, c[0], c[1], size, kind);
            big += 1.0;
        }
    }
    tries = 0.0;
    while (items.len() as i64) < n && tries < nf * 20.0 && !cells.is_empty() {
        tries += 1.0;
        let c = cells[(rng.int(1, cells.len() as i64) - 1) as usize];
        let r = s.clearance(c[0], c[1], 8.0);
        if r >= 3.0 {
            let size = rng.range(1.4, 3.4f64.min(r * 0.8));
            let kind = rng.weighted(&[("plain", 5.0), ("dotted", 3.0), ("lozenge", 2.0)]).unwrap();
            put(s, &mut items, c[0], c[1], size, kind);
        }
    }
    items
}
