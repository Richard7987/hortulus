//! The border as one plant: a main stem grown by steering, and shoots off it.

use crate::geom::{self as g, Follow, Pt, PursueOpts};
use crate::layout::{GestureField, Margin};
use crate::rng::Rng;
use crate::tune::{tune, PlantCfg};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

pub type Box4 = [f64; 4];

fn clear_of(bx: Option<Box4>, q: Pt, gap: f64, ease: f64) -> Pt {
    let bx = match bx {
        Some(b) => b,
        None => return q,
    };
    let push = g::smoothstep(bx[1] - 12.0 - ease, bx[1] - 12.0, q[1]) * (1.0 - g::smoothstep(bx[3] + 12.0, bx[3] + 12.0 + ease, q[1]));
    let need = bx[2] + gap - q[0];
    if need <= 0.0 || push <= 0.0 {
        return q;
    }
    [q[0] + need * push, q[1]]
}

pub struct Stem {
    pub pts: Vec<Pt>,
    pub cum: Option<Vec<f64>>,
    pub len: f64,
    pub w0: f64,
    pub w1: f64,
    pub forks: Vec<(f64, f64)>,
    pub min: f64,
    pub tip: Option<(Pt, f64)>,
    pub fixed_width: Option<fn(f64) -> f64>,
}

pub type StemRef = Rc<RefCell<Stem>>;

impl Stem {
    /// A stem with a fixed width function, without forks.
    pub fn plain(pts: Vec<Pt>, w: fn(f64) -> f64) -> Stem {
        Stem { pts, cum: None, len: 0.0, w0: 0.0, w1: 0.0, forks: Vec::new(), min: 0.35, tip: None, fixed_width: Some(w) }
    }

    pub fn width(&self, t: f64) -> f64 {
        match self.fixed_width {
            Some(f) => f(t),
            None => width_at(self, t * self.len),
        }
    }
}

pub fn stem(pts: Vec<Pt>, w0: f64, w1: f64) -> Stem {
    let mut cum = vec![0.0; pts.len()];
    let mut total = 0.0;
    for i in 1..pts.len() {
        total += g::dist(pts[i - 1], pts[i]);
        cum[i] = total;
    }
    Stem { pts, cum: Some(cum), len: total.max(1e-9), w0, w1, forks: Vec::new(), min: 0.35, tip: None, fixed_width: None }
}

pub fn width_at(st: &Stem, s: f64) -> f64 {
    let t = g::clamp(s / st.len, 0.0, 1.0);
    let mut w = st.w0 + (st.w1 - st.w0) * t;
    for f in &st.forks {
        let u = g::smoothstep(f.0 - 2.0, f.0 + 6.0, s);
        w *= 1.0 - u + u * f.1;
    }
    w.max(st.min)
}

pub fn fork(st: &mut Stem, t: f64, cw: f64, floor: f64) -> f64 {
    let s = t * st.len;
    let w = width_at(st, s);
    let r = g::clamp(cw / w, 0.0, 0.95);
    st.forks.push((s, floor.max((1.0 - r * r).sqrt())));
    w
}

pub fn at(st: &Stem, s: f64) -> (Pt, f64) {
    let s = g::clamp(s, 0.0, st.len);
    let (pts, cum) = (&st.pts, st.cum.as_ref().unwrap());
    let mut i = 1;
    while i < pts.len() - 1 && cum[i] < s {
        i += 1;
    }
    let (a, b) = (pts[i - 1], pts[i]);
    let u = (s - cum[i - 1]) / (cum[i] - cum[i - 1]).max(1e-9);
    (g::mix(a, b, u), g::angle(a, b))
}

fn avoid_target(t: Option<Pt>, q: Pt, a: f64, bx: Option<Box4>) -> Option<Pt> {
    let t = t?;
    if let Some(b) = bx {
        for d in [15.0, 35.0, 60.0, 100.0] {
            let f = g::polar(q, a, d);
            if f[0] < b[2] + 6.0 && f[1] > b[1] - 6.0 && f[1] < b[3] + 6.0 {
                return Some([b[2] + 16.0, f[1] + 10.0]);
            }
        }
    }
    Some(t)
}

fn guide(rng: &mut Rng, m: &Margin, hero: [f64; 3], w: f64, gest: &GestureField, cfg: &PlantCfg, avoid: Option<Box4>) -> (Vec<Pt>, Pt, Pt) {
    let mw = m.x1 - m.x0;
    let r = mw * 0.32;
    let footy = m.y0 + 6.0;
    let xr = gest.edge;
    let rise_y = footy + r;
    let mut pts = Vec::new();
    let period = rng.range(120.0, 170.0);
    let mut x = w * cfg.root;
    while x > xr + r + 12.0 {
        let ramp = g::smoothstep(xr + r, xr + r + 60.0, x);
        pts.push([x, footy + 4.0 * ramp * (1.0 + 2.0 * PI * x / period).sin()]);
        x -= 20.0;
    }
    for i in 0..=10 {
        let th = -PI / 2.0 - (PI / 2.0) * i as f64 / 10.0;
        pts.push([xr + r + r * th.cos(), rise_y + r * th.sin()]);
    }
    let sway_len = rng.range(300.0, 400.0);
    let amp = mw * rng.range(0.015, 0.03);
    let cap = [hero[0] - hero[2] * 0.1, hero[1] - hero[2] * 0.15];
    let pull_from = cap[1] - (hero[2] * 2.4).max(90.0);
    let mut y = rise_y + 16.0;
    while y < cap[1] - 12.0 {
        let ramp = g::smoothstep(rise_y, rise_y + 80.0, y);
        let sway = (2.0 * PI * (y - rise_y) / sway_len).sin();
        let mut gx = xr + ramp * (amp * sway + gest.bow(y));
        gx += (cap[0] - gx) * g::smoothstep(pull_from, cap[1], y);
        pts.push(clear_of(avoid, [gx, y], 9.0, 60.0));
        y += 12.0;
    }
    pts.push(cap);
    (pts, [xr, rise_y], cap)
}

pub fn main_stem(rng: &mut Rng, m: &Margin, hero: [f64; 3], w: f64, gest: &GestureField, avoid: Option<Box4>) -> (Stem, Pt, Pt) {
    let cfg = &tune().plant;
    let (gpts, corner, cap) = guide(rng, m, hero, w, gest, cfg, avoid);
    let kmax = 1.0 / cfg.bend_radius;
    let mut follow = Follow::new(&gpts, cfg.lookahead, 14.0);
    let (mut pts, _) = g::pursue(
        gpts[0],
        g::angle(gpts[0], gpts[1]),
        |q, _s, a| avoid_target(follow.target(q), q, a, avoid),
        &PursueOpts { len: g::length(&gpts) * 1.3, kmax, dk: kmax / cfg.bend_ease, step: 2.0, k0: 0.0 },
        |_| false,
    );
    let tail = pts[pts.len() - 1];
    let a = g::angle(pts[pts.len() - 2], tail);
    let link = g::link(tail, a, cap, g::angle(cap, [hero[0], hero[1]]), 0.45, 0);
    pts.extend_from_slice(&link[1..]);
    let mut st = stem(pts, cfg.root_width, cfg.root_width * 0.8);
    st.min = cfg.tip_width;
    (st, corner, cap)
}

pub fn crown(rng: &mut Rng, main: &mut Stem, m: &Margin, hero: [f64; 3]) -> Vec<Stem> {
    let cfg = &tune().plant;
    let r0 = hero[2];
    let hc = [hero[0], hero[1]];
    let mut s0 = main.len;
    let mut s = main.len;
    while s >= 0.0 {
        if at(main, s).0[1] < hero[1] - r0 * 1.5 {
            s0 = s;
            break;
        }
        s -= 2.0;
    }
    let mut out = Vec::new();
    for (k, side) in [(1, -1.0), (2, 1.0)] {
        let mut r = rng.fork(&format!("crown{}", k));
        let s = (s0 - r.range(0.0, r0 * 0.9)).max(0.0);
        let (p0, a0) = at(main, s);
        let ta = (if side < 0.0 { PI } else { 0.0 }) - side * r.range(0.15, 0.55);
        let mut to = g::polar(hc, ta, r0 * r.range(0.85, 0.95));
        to[0] = g::clamp(to[0], m.x0 + 4.0, m.x1 - 4.0);
        let pts = g::link(p0, a0 - side * 0.55, to, PI / 2.0 - side * 0.5, 0.6, 0);
        let clear = pts.iter().all(|q| g::dist(*q, hc) >= r0 * 0.7);
        if clear && pts.len() >= 6 {
            let w = fork(main, s / main.len, cfg.root_width * cfg.fork_ratio * 0.6, 0.75);
            let tip = (pts[pts.len() - 1], g::angle(pts[pts.len() - 2], pts[pts.len() - 1]));
            let mut st = stem(pts, w * 0.7, 0.35);
            st.min = 0.3;
            st.tip = Some(tip);
            out.push(st);
        }
    }
    out
}

pub fn long_shoots(rng: &mut Rng, main: &mut Stem, m: &Margin, hero: [f64; 3], gest: &GestureField, avoid: Option<Box4>) -> Vec<Stem> {
    let cfg = &tune().plant;
    let mw = m.x1 - m.x0;
    let mut out = Vec::new();
    let mut bow_max: f64 = 1e-9;
    for b in &gest.bows {
        bow_max = bow_max.max(b.amp);
    }
    let mut rise0 = main.len;
    let mut s = 0.0;
    while s <= main.len {
        if at(main, s).0[1] > m.y0 + mw * 0.5 {
            rise0 = s;
            break;
        }
        s += 4.0;
    }
    let mut rise1 = main.len;
    let mut s = main.len;
    while s >= 0.0 {
        if at(main, s).0[1] < hero[1] - hero[2] * 0.7 {
            rise1 = s;
            break;
        }
        s -= 4.0;
    }
    let span = (rise1 - rise0).max(1.0);
    let n = cfg.long_shoots;
    for k in 1..=(n as i64) {
        let kf = k as f64;
        let mut r = rng.fork(&format!("long{}", k));
        let (ss, se) = (cfg.shoot_start, cfg.shoot_end);
        let s0 = (main.len * 0.15).max(rise0 + span * r.range(ss[0], ss[1]));
        let f = g::clamp((kf - 0.5 + r.jit(0.3)) / n + 0.5 / n, 0.0, 1.0);
        let s1 = rise1.min((s0 + 60.0).max(rise0 + span * (se[0] + (se[1] - se[0]) * f)));
        let amp = mw * r.rangev(cfg.shoot_reach);
        let wave = r.rangev(cfg.shoot_wave);
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let ph = r.range(0.0, 0.5);
        let pv = at(main, s0 + (s1 - s0) * 0.65).0;
        let veer = if pv[0] - m.x0 > m.x1 - pv[0] { 1.0 } else { -1.0 };
        let mut gpts = Vec::new();
        let mut s = s0 + 6.0;
        while s <= s1 {
            let (p, a) = at(main, s);
            let u = s - s0;
            let env = 0.5 + 0.5 * g::clamp(gest.bow(p[1]) / bow_max, 0.0, 1.0);
            let mut off = side * amp * env * g::smoothstep(0.0, 50.0, u) * (2.0 * PI * (ph + u / wave)).sin();
            off += veer * amp * 1.4 * g::smoothstep(0.65, 1.0, u / (s1 - s0));
            let upright = g::smoothstep(0.3, 0.7, a.sin().abs());
            let dx = (if (a + PI / 2.0).cos() >= 0.0 { 1.0 } else { -1.0 }) * off * upright;
            let q = clear_of(avoid, [p[0] + dx, p[1]], 6.0, 150.0);
            gpts.push([g::clamp(q[0], m.x0 + 6.0, m.x1 - 6.0), q[1]]);
            s += 8.0;
        }
        if gpts.len() >= 4 {
            let (p0, a0) = at(main, s0);
            let kmax = 1.0 / (cfg.bend_radius * 0.6);
            let ahead = (cfg.lookahead * 0.7).min(wave * 0.12);
            let mut follow = Follow::new(&gpts, ahead, 10.0);
            let (pts, a) = g::pursue(
                p0,
                a0 + side * 0.08,
                |q, _s, a| avoid_target(follow.target(q), q, a, avoid),
                &PursueOpts { len: (s1 - s0) * 1.3, kmax, dk: kmax / (cfg.bend_ease * 0.7), step: 2.0, k0: 0.0 },
                |q| q[0] > m.x1 - 3.0 || q[0] < m.x0 + 2.0,
            );
            if pts.len() >= 6 {
                let w = fork(main, s0 / main.len, cfg.root_width * cfg.fork_ratio, 0.75);
                let tip = (pts[pts.len() - 1], a);
                let mut st = stem(pts, w * cfg.fork_ratio, 0.35);
                st.min = 0.3;
                st.tip = Some(tip);
                out.push(st);
            }
        }
    }
    out
}

pub fn foot_shoots(rng: &mut Rng, main: &mut Stem, m: &Margin, corner: Pt, avoid: Option<Box4>) -> Vec<Stem> {
    let cfg = &tune().plant;
    let mut out = Vec::new();
    let inside = [corner[0] + (m.x1 - m.x0) * 0.6, corner[1] + 60.0];
    let s_end = main.len * 0.3;
    for k in 1..=(cfg.foot_shoots as i64) {
        let mut r = rng.fork(&format!("foot{}", k));
        let s0 = s_end * (k as f64 - 0.6 + r.jit(0.25)) / cfg.foot_shoots;
        let (p0, a0) = at(main, s0);
        let (nx, ny) = ((a0 + PI / 2.0).cos(), (a0 + PI / 2.0).sin());
        let sign = if nx * (inside[0] - p0[0]) + ny * (inside[1] - p0[1]) >= 0.0 { 1.0 } else { -1.0 };
        let len = r.rangev(cfg.foot_reach);
        let target = g::polar(p0, a0 + sign * r.range(0.55, 0.95), len);
        let kmax = 1.0 / cfg.shoot_bend;
        let (pts, a) = g::pursue(
            p0,
            a0 + sign * 0.1,
            |q, _s, _a| if g::dist(q, target) < 4.0 { None } else { Some(target) },
            &PursueOpts { len: len * 1.6, kmax, dk: kmax / 20.0, step: 1.5, k0: 0.0 },
            |q| {
                (q[0] > m.x1 - 4.0 && q[1] > m.foot - 8.0)
                    || q[0] < m.x0 + 3.0
                    || q[1] < m.y0 - 2.0
                    || avoid.map_or(false, |b| q[0] <= b[2] + 2.0 && q[1] >= b[1] - 2.0 && q[1] <= b[3] + 2.0)
            },
        );
        if pts.len() >= 8 {
            let w = fork(main, s0 / main.len, 1.0, 0.9);
            let tip = (pts[pts.len() - 1], a);
            let mut st = stem(pts, 1.1f64.min(w * 0.4), 0.3);
            st.min = 0.3;
            st.tip = Some(tip);
            out.push(st);
        }
    }
    out
}
