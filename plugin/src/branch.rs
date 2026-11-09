//! Natural growth: side branches off the main stem, each with its own shorter
//! branches, leaving at a steady angle and bending back towards the light.

use crate::geom::{self as g, Pt};
use crate::layout::Margin;
use crate::plant::{self, Box4, Stem};
use crate::rng::Rng;
use crate::space::Space;
use std::f64::consts::PI;

const UP: f64 = PI / 2.0;
const GAP: f64 = 9.0;

struct Order {
    internode: [f64; 2],
    reach: f64,
    min_len: f64,
    width: f64,
    subs: usize,
}

const ORDERS: [Order; 2] = [
    Order { internode: [62.0, 96.0], reach: 1.25, min_len: 34.0, width: 0.62, subs: 2 },
    Order { internode: [26.0, 40.0], reach: 0.5, min_len: 18.0, width: 0.6, subs: 0 },
];

struct Grove<'a> {
    s: &'a Space,
    m: &'a Margin,
    hero: [f64; 3],
    avoid: Option<Box4>,
    taken: Vec<Pt>,
    divergence: f64,
}

impl Grove<'_> {
    fn clear(&self, q: Pt, origin: Pt) -> bool {
        let m = self.m;
        if q[0] < m.x0 + 3.0 || q[0] > m.x1 - 3.0 || q[1] < m.y0 + 2.0 || !self.s.open_at(q[0], q[1]) {
            return false;
        }
        if g::dist(q, [self.hero[0], self.hero[1]]) < self.hero[2] * 0.95 {
            return false;
        }
        if self.avoid.is_some_and(|b| q[0] <= b[2] + 4.0 && q[1] >= b[1] - 4.0 && q[1] <= b[3] + 4.0) {
            return false;
        }
        // Near its own fork a branch may touch the parent; further out it keeps clear of every stem.
        g::dist(q, origin) < 14.0 || self.taken.iter().all(|p| g::dist(*p, q) >= GAP)
    }

    /// A branch leaving `p` at `a0`, turning back towards upright over its length.
    fn shoot(&self, r: &mut Rng, p: Pt, a0: f64, len: f64) -> Vec<Pt> {
        let lift = g::turn(UP, a0) * r.range(0.55, 0.9);
        let (wobble, ph) = (r.range(0.15, 0.35), r.range(0.0, 2.0 * PI));
        let (pts, _) = g::flow(p, a0, len, |t| (lift * 1.5 * (1.0 - t * t) + wobble * (2.0 * PI * (1.5 * t + ph)).sin()) / len, 0);
        let mut out = vec![pts[0]];
        for q in &pts[1..] {
            if !self.clear(*q, p) {
                break;
            }
            out.push(*q);
        }
        out
    }

    fn branches(&mut self, r: &mut Rng, parent: &mut Stem, from: f64, to: f64, level: usize, budget: f64) -> Vec<Stem> {
        let o = &ORDERS[level];
        let mut out = Vec::new();
        let mut side = if r.chance(0.5) { 1.0 } else { -1.0 };
        let mut s = from + r.range(0.3, 1.0) * o.internode[0];
        let mut made = 0;
        while s < to && (level == 0 || made < o.subs) {
            let (p, ta) = plant::at(parent, s);
            side = -side;
            // Lower branches are older and longer.
            let u = (s - from) / (to - from).max(1.0);
            let len = budget * o.reach * (1.0 - 0.45 * u) * r.range(0.8, 1.15);
            let a0 = ta + side * (self.divergence + r.jit(0.12));
            let mut kr = r.fork(&format!("b{}.{}", level, made));
            let pts = self.shoot(&mut kr, p, a0, len);
            if g::length(&pts) >= o.min_len && pts.len() >= 6 {
                let pw = plant::width_at(parent, s);
                let cw = (pw * o.width).max(0.45);
                plant::fork(parent, s / parent.len, cw, 0.72);
                let tip = (pts[pts.len() - 1], g::angle(pts[pts.len() - 2], pts[pts.len() - 1]));
                self.taken.extend(pts.iter().copied());
                let mut st = plant::stem(pts, cw, 0.3);
                st.min = 0.3;
                st.tip = Some(tip);
                if level + 1 < ORDERS.len() {
                    let l = st.len;
                    let subs = self.branches(&mut kr, &mut st, l * 0.25, l * 0.85, level + 1, l);
                    out.push(st);
                    out.extend(subs);
                } else {
                    out.push(st);
                }
                made += 1;
            }
            s += r.range(o.internode[0], o.internode[1]);
        }
        out
    }
}

/// Side branches along the rising part of the main stem, and branches of those.
pub fn laterals(rng: &mut Rng, s: &Space, main: &mut Stem, others: &[Stem], m: &Margin, hero: [f64; 3], avoid: Option<Box4>) -> Vec<Stem> {
    let mw = m.x1 - m.x0;
    let mut taken: Vec<Pt> = main.pts.clone();
    for st in others {
        taken.extend(st.pts.iter().copied());
    }
    let mut grove = Grove { s, m, hero, avoid, taken, divergence: rng.range(0.6, 0.8) };
    let rise = |st: &Stem, y: f64| {
        let mut d = 0.0;
        while d <= st.len && plant::at(st, d).0[1] < y {
            d += 4.0;
        }
        d
    };
    let from = rise(main, m.y0 + mw * 0.45);
    let mut to = main.len;
    while to > 0.0 && plant::at(main, to).0[1] > hero[1] - hero[2] * 1.3 {
        to -= 4.0;
    }
    grove.branches(rng, main, from, to, 0, mw)
}
