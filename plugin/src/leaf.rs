//! Leaves: a blade grown around a curved midrib, painted as a watercolour wash.

use crate::brush;
use crate::color::{self, Rgb};
use crate::geom::{self as g, Cmd, Noise, Pt};
use crate::paint::{self, Paint, MULT, NONE};
use crate::rng::Rng;
use crate::tune::tune;
use crate::wash::{self, HairOpts, WashOpts};
use std::f64::consts::PI;

pub struct Jit<'a> {
    pub phase: &'a Noise,
    pub depth: &'a Noise,
}

pub fn margin(kind: &str, t: f64, teeth: f64, depth: f64, jit: Option<&Jit>) -> f64 {
    let fade = g::smoothstep(0.12, 0.28, t) * (1.0 - g::smoothstep(0.86, 1.0, t));
    let (mut t, mut depth) = (t, depth);
    if let Some(j) = jit {
        depth *= 0.7 + 0.5 * (j.depth.at(t * teeth) + 1.0) / 2.0;
        t += 0.3 * j.phase.at(t * teeth * 0.7) / teeth.max(1.0);
    }
    match kind {
        "serrate" => {
            let u = t * teeth;
            let f = u - u.floor();
            1.0 - depth * fade * (1.0 - f.powf(1.7))
        }
        "acanthus" => {
            let ul = t * teeth;
            let fl = ul - ul.floor();
            let c = 0.62;
            let crest = if fl < c { (PI / 2.0 * fl / c).sin().powf(0.8) } else { 1.0 - ((fl - c) / (1.0 - c)).powf(1.6) };
            1.0 - fade * depth * (1.0 - crest)
        }
        "feather" => {
            let ul = t * teeth;
            let fl = ul - ul.floor();
            let c = 0.78;
            let crest = if fl < c { (PI / 2.0 * fl / c).sin().powf(1.3) } else { 1.0 - ((fl - c) / (1.0 - c)).powf(0.9) };
            1.0 - fade * depth * (1.0 - crest)
        }
        _ => 1.0,
    }
}

const STALK: f64 = 0.07;

pub fn blade(t: f64, kp: f64, ep: f64) -> f64 {
    let u = ((t - STALK) / (1.0 - STALK)).max(0.0);
    0.035 + 0.965 * (PI * u.powf(kp)).sin().powf(ep) * g::smoothstep(0.0, 0.32, u).powf(0.6)
}

#[derive(Clone)]
pub struct Attach {
    pub f: f64,
    pub p: Pt,
    pub seg: Vec<Pt>,
    pub w: f64,
}

#[derive(Clone, Default)]
pub struct LeafOpts {
    pub len: f64,
    pub width: f64,
    pub bend: f64,
    pub hook: f64,
    pub margin: &'static str,
    pub shape: &'static str,
    pub look: &'static str,
    pub veins: Option<i64>,
    pub lightning: bool,
    pub teeth: Option<f64>,
    pub depth: f64,
    pub sides: &'static str,
    pub inner: f64,
    pub sweep: Option<f64>,
    pub striate: bool,
    pub attach: Option<Attach>,
}

pub fn profile(o: &LeafOpts, t: f64) -> f64 {
    if o.shape == "lance" {
        blade(t, 0.55, 1.05)
    } else {
        blade(t, 0.68, 0.85)
    }
}

pub fn outline(rng: &mut Rng, o: &LeafOpts) -> (Vec<Pt>, Vec<Pt>, Vec<Pt>, Vec<Pt>) {
    let teeth = o.teeth.unwrap_or(0.0);
    let n = 48f64.max(teeth * if o.sweep.is_some() { 16.0 } else { 7.0 }) as usize;
    let spine = g::curve([0.0, 0.0], PI / 2.0, o.len, o.bend, o.hook, n);
    let nz = g::Fbm::new(rng, 32);
    let jlp = Noise::new(rng, 32);
    let jld = Noise::new(rng, 32);
    let jrp = Noise::new(rng, 32);
    let jrd = Noise::new(rng, 32);
    let (jl, jr) = (Jit { phase: &jlp, depth: &jld }, Jit { phase: &jrp, depth: &jrd });
    let al = 1.0 + rng.jit(0.12);
    let ar = 1.0 + rng.jit(0.12);
    let m = spine.len();
    let mut left = Vec::with_capacity(m);
    let mut right = Vec::with_capacity(m);
    let sweep = o.sweep.unwrap_or(0.0);
    for i in 0..m {
        let t = i as f64 / (m as f64 - 1.0);
        let a = spine[i.saturating_sub(1)];
        let b = spine[(i + 1).min(m - 1)];
        let nrm = g::angle(a, b) + PI / 2.0;
        let prof = profile(o, t);
        let w = o.width * prof * (1.0 + 0.06 * nz.at(t * 6.0));
        let dl = if o.sides != "right" { o.depth } else { o.depth * o.inner };
        let dr = if o.sides != "left" { o.depth } else { o.depth * o.inner };
        let ml = margin(o.margin, t, teeth, dl, Some(&jl));
        let mr = margin(o.margin, t + 0.5 / o.teeth.unwrap_or(1.0).max(1.0), teeth, dr, Some(&jr));
        let (rl, rr) = (al * ml * prof, ar * mr * prof);
        let sl = if dl > 0.0 { sweep * rl.min(1.0) } else { 0.0 };
        let sr = if dr > 0.0 { sweep * rr.min(1.0) } else { 0.0 };
        left.push(g::polar(spine[i], nrm - sl, w * al * ml));
        right.push(g::polar(spine[i], nrm + PI + sr, w * ar * mr));
    }
    let mut out = left.clone();
    out.extend(right[1..m - 1].iter().rev());
    (out, spine, left, right)
}

fn broken_lines(pt: &mut Paint, rng: &mut Rng, lines: &[Vec<Pt>], w: f64, gap: f64, lo: f64, hi: f64) {
    let dry = tune().wash.brush_dry * 0.5;
    for line in lines {
        let pieces = brush::broken(rng, line, w, gap, dry);
        let cmds = brush::polygons(&pieces);
        if !cmds.is_empty() {
            let op = rng.range(lo, hi);
            pt.fill_many(&cmds, paint::WHITE, op, NONE);
        }
    }
}

fn veins(pt: &mut Paint, rng: &mut Rng, spine: &[Pt], left: &[Pt], right: &[Pt], size: f64, o: &LeafOpts) {
    let n = spine.len();
    let nf = n as f64;
    let fl = |x: f64| x.floor() as usize;
    let at = |v: &[Pt], i: usize| v[i - 1];
    let mut mid = Vec::new();
    let mut i = 2.max(fl(nf * 0.03));
    while i <= fl(nf * 0.93) {
        mid.push(at(spine, i));
        i += 1;
    }
    let cfg = &tune().wash;
    broken_lines(pt, rng, std::slice::from_ref(&mid), size * 0.03, 0.08, 0.5, 0.75);
    let ink = color::recolor(paint::INK, color::part_ink(Some("leaves")));
    let moved = g::transform(&mid, size * 0.012, -size * 0.004, 0.0, 1.0, 1.0);
    wash::hairlines(pt, &[moved], ink, size * 0.006, 0.35, HairOpts { head: Some(0.02), blend: MULT.blend });

    let (mut lats, mut forks) = (Vec::new(), Vec::new());
    let clampi = |t: f64| g::clamp((t * (nf - 1.0)).floor() + 1.0, 2.0, nf - 1.0) as usize;
    if o.striate {
        let mut fine = Vec::new();
        let count = (o.teeth.unwrap_or(6.0) * 3.0) as i64;
        for side in [left, right] {
            for k in 1..=count {
                let t = 0.1 + 0.82 * (k as f64 - 0.5) / count as f64 + rng.jit(0.01);
                let i = clampi(t);
                let (p0, tip) = (at(spine, i), at(side, (i + fl(nf * 0.05)).min(n)));
                let p1 = g::mix(p0, tip, rng.range(0.6, 0.88));
                let m = g::mix(g::mix(p0, p1, 0.5), at(spine, (i + fl(nf * 0.03)).min(n)), 0.2);
                fine.push(g::spline_points(&[p0, m, p1], 5));
            }
        }
        broken_lines(pt, rng, &fine, size * 0.009, cfg.brush_gap, cfg.vein_contrast[0], cfg.vein_contrast[1]);
    }
    let count = if o.striate { 0 } else { o.veins.unwrap_or(6) };
    for k in 1..=count {
        let t = 0.10 + 0.78 * (k as f64 - 0.5) / count as f64 + rng.jit(0.03);
        let i = clampi(t);
        for side in [left, right] {
            let (p0, tip) = (at(spine, i), at(side, (i + fl(nf * 0.08)).min(n)));
            let p1 = g::mix(p0, tip, rng.range(0.72, 0.9));
            let m = g::mix(p0, p1, 0.5);
            let m = g::mix(m, at(spine, (i + fl(nf * 0.05)).min(n)), 0.25);
            let line = g::spline_points(&[p0, m, p1], 5);
            if o.lightning && rng.chance(0.8) {
                let (q, ang) = g::at(&line, rng.range(0.4, 0.7));
                let dir = ang + rng.jit(0.7);
                let bend = rng.jit(0.5);
                forks.push(g::curve(q, dir, g::dist(p0, p1) * 0.4, bend, 0.0, 6));
            }
            lats.push(line);
        }
    }
    broken_lines(pt, rng, &lats, size * 0.02, cfg.brush_gap, cfg.vein_contrast[0], cfg.vein_contrast[1]);
    if !forks.is_empty() {
        broken_lines(pt, rng, &forks, size * 0.012, cfg.brush_gap, cfg.vein_contrast[0] * 0.7, cfg.vein_contrast[1] * 0.7);
    }

    let mut hatch = Vec::new();
    let back = 1.max(fl(nf * 0.06));
    for side in [left, right] {
        let mut i = fl(nf * 0.08);
        while i <= fl(nf * 0.92) {
            let m = at(side, i);
            let q = g::mix(m, at(spine, 1.max(i.saturating_sub(back))), rng.range(0.18, 0.38));
            hatch.push(vec![m, q]);
            i += 2;
        }
    }
    wash::hairlines(pt, &hatch, ink, size * 0.007, 0.32, HairOpts { head: Some(0.01), blend: MULT.blend });
}

pub fn options(r: &mut Rng, len: f64, ang: f64) -> LeafOpts {
    let up = if ang.cos() < 0.0 { -1.0 } else { 1.0 };
    let kind = r.weighted(&[("feather", 6.0), ("acanthus", 1.5), ("lance", 2.5)]).unwrap();
    let kind = if len < 45.0 { "lance" } else { kind };
    let droop = r.chance(0.5);
    let deep = r.chance(0.5);
    let lw = tune().compose.leaf_width;
    let mut o = LeafOpts { sides: "both", shape: "lance", ..Default::default() };
    o.len = len * r.range(0.85, 1.0);
    o.width = len * r.range(lw[0], lw[1]);
    o.bend = up * r.range(0.6, 1.3);
    o.hook = (if droop { -up } else { up }) * r.range(0.5, 1.4);
    o.margin = if kind == "lance" { "entire" } else { kind };
    o.look = r.weighted(&[("vivid", 3.0), ("soft", 2.0)]).unwrap();
    o.veins = Some(r.int(5, 7));
    o.lightning = r.chance(0.25);
    if kind == "acanthus" {
        o.teeth = Some(r.int(3, 6) as f64);
        o.depth = if deep { r.range(0.5, 0.72) } else { r.range(0.18, 0.32) };
        o.sides = "both";
        o.width *= 1.25;
    } else if kind == "feather" {
        o.teeth = Some(r.int(5, 8) as f64);
        o.depth = if deep { r.range(0.6, 0.82) } else { r.range(0.32, 0.45) };
        o.sides = if up > 0.0 { "right" } else { "left" };
        o.inner = if r.chance(0.6) { r.range(0.2, 0.45) } else { 0.0 };
        o.sweep = Some(r.range(0.5, 0.75));
        o.bend *= 1.25;
        o.width *= 0.85;
        o.striate = true;
    } else {
        o.width *= 0.75;
        o.bend *= 0.5;
        if r.chance(0.3) {
            o.hook *= 1.6;
        } else {
            o.hook *= 0.6;
        }
    }
    o
}

pub fn halo_runs(poly: &[Pt], base: Pt, size: f64) -> Vec<Vec<Pt>> {
    let n = poly.len();
    let (mut i0, mut best) = (0, f64::INFINITY);
    for (i, p) in poly.iter().enumerate() {
        let d = g::dist(*p, base);
        if d < best {
            i0 = i;
            best = d;
        }
    }
    let mut runs = Vec::new();
    let mut run: Option<Vec<Pt>> = None;
    for j in 1..=n {
        let p = poly[(i0 + j) % n];
        if g::dist(p, base) > size * 0.18 {
            run.get_or_insert_with(Vec::new).push(p);
        } else {
            if let Some(r) = run.take() {
                if r.len() >= 3 {
                    runs.push(r);
                }
            }
        }
    }
    if let Some(r) = run {
        if r.len() >= 3 {
            runs.push(r);
        }
    }
    runs
}

fn halo(pt: &mut Paint, poly: &[Pt], base: Pt, size: f64) {
    let runs: Vec<Vec<Cmd>> = halo_runs(poly, base, size).iter().map(|r| g::smooth(r, false)).collect();
    if runs.is_empty() {
        return;
    }
    for w in [3.4, 2.2, 1.2] {
        pt.stroke_many(&runs, paint::PAPER, w, 0.2, NONE);
    }
}

fn leaf_look(name: &str) -> (Rgb, Rgb, f64) {
    match name {
        "deep" => (paint::DEEP, paint::INK, 1.0),
        "soft" => (paint::SKY, paint::DEEP, 0.95),
        _ => (paint::BLUE, paint::DEEP, 1.0),
    }
}

pub fn leaf(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, ang: f64, o: &LeafOpts) -> Vec<Pt> {
    let (local, spine, left, right) = outline(rng, o);
    let rot = ang - PI / 2.0;
    let poly = g::transform(&local, x, y, rot, 1.0, 1.0);
    let spine = g::transform(&spine, x, y, rot, 1.0, 1.0);
    let left = g::transform(&left, x, y, rot, 1.0, 1.0);
    let right = g::transform(&right, x, y, rot, 1.0, 1.0);
    let (pig, pool, strength) = leaf_look(o.look);
    let size = o.len;

    let mut lit = spine.clone();
    lit.extend(right.iter().rev());

    halo(pt, &poly, spine[0], size);
    let lines = |r: &mut Rng, n: i64| -> Vec<Vec<Pt>> {
        let mut out = Vec::with_capacity(n.max(0) as usize);
        for _ in 0..n {
            let edge = if r.chance(0.5) { &left } else { &right };
            let s = r.range(0.15, 0.85);
            out.push(spine.iter().enumerate().map(|(i, p)| g::mix(*p, edge[i.min(edge.len() - 1)], s)).collect());
        }
        out
    };
    let mut inner = |p: &mut Paint, r: &mut Rng| {
        p.fill(&g::smooth(&lit, true), paint::WHITE, 0.08, NONE);
        veins(p, r, &spine, &left, &right, size, o);
    };
    wash::wash(
        pt,
        &poly,
        rng,
        WashOpts {
            size,
            ramp: Some(if o.look == "soft" { "soft" } else { "leaf" }),
            axis: Some((spine[0], spine[spine.len() - 1])),
            light: Some(g::at(&spine, 0.25).0),
            pigment: Some(pig),
            pool: Some(pool),
            strength: Some(strength),
            rings: Some(8),
            lift: Some(0.035),
            bleed: Some(o.width * 0.05),
            pale_scale: Some(0.2),
            lifts: Some(0.5),
            fade_scale: Some(0.5),
            brush_lines: Some(&lines),
            grain: 0.45,
            grain_scale: Some(0.9),
            outline: 0.85,
            outline_w: Some(0.28f64.max(size * 0.008)),
            inner: Some(&mut inner),
            ..Default::default()
        },
    );
    if let Some(a) = &o.attach {
        attach(pt, &poly, &spine, size, a);
    }
    spine
}

pub const FRONT: f64 = 0.35;

pub fn attach(pt: &mut Paint, poly: &[Pt], spine: &[Pt], size: f64, at: &Attach) {
    let tone = brush::stem_tone(&tune().plant);
    let base = spine[0];
    let body = g::smooth(poly, true);
    if at.f.abs() < FRONT {
        let from = g::mix(at.p, base, at.w / 2.0 / g::dist(at.p, base).max(1e-9));
        let to = g::polar(base, g::angle(spine[0], spine[1]), 3.0);
        for k in 1..=4 {
            let kf = k as f64 / 4.0;
            let line = g::resample(&[from, g::mix(from, to, kf)], 6);
            let rb = g::ribbon(&line, |t| at.w * (0.3 - 0.1 * t * kf));
            pt.fill(&g::smooth(&rb, true), tone, 0.22, MULT);
        }
    }
    pt.clip_begin(&body);
    let reach = size * if at.f > FRONT { 0.24 } else { 0.16 };
    for k in 1..=6 {
        pt.fill(&[Cmd::O(base[0], base[1], reach * (1.0 - (k - 1) as f64 / 6.0))], tone, 0.08, MULT);
    }
    if at.f > FRONT {
        let mut rib = Vec::new();
        for i in 0..spine.len() {
            if g::length(&spine[..=i]) > size * 0.22 {
                break;
            }
            rib.push(spine[i]);
        }
        if rib.len() >= 2 {
            pt.fill(&g::smooth(&wash::taper(&rib, at.w * 0.85, 0.001), true), tone, 0.55, MULT);
        }
    }
    if at.seg.len() >= 2 {
        let band = |k: f64| g::smooth(&g::ribbon(&at.seg, |_| at.w * k / 2.0), true);
        if at.f < -FRONT {
            pt.fill(&band(2.6), color::recolor(paint::DEEP, color::part_ink(Some("leaves"))), 0.14, MULT);
            pt.fill(&band(1.0), tone, 0.6, NONE);
        } else {
            pt.fill(&band(1.0), tone, g::clamp(0.45 - 0.35 * at.f, 0.15, 0.45), MULT);
        }
    }
    pt.clip_end();
}

pub fn leaflets(pt: &mut Paint, rng: &mut Rng, items: &[[f64; 4]], rgb: Option<Rgb>, opacity: f64) {
    let (mut bodies, mut ribs) = (Vec::new(), Vec::new());
    for it in items {
        let (x, y, ang, len) = (it[0], it[1], it[2], it[3]);
        let bend = rng.jit(0.35);
        let wd = len * rng.range(0.17, 0.23);
        let kp = rng.range(0.62, 0.74);
        let ep = rng.range(0.8, 0.95);
        let al = 1.0 + rng.jit(0.12);
        let ar = 1.0 + rng.jit(0.12);
        let mut pts = Vec::with_capacity(32);
        for i in 0..=16 {
            let t = i as f64 / 16.0;
            pts.push([-wd * al * blade(t, kp, ep) + bend * len * t * t, len * t]);
        }
        for i in (1..=15).rev() {
            let t = i as f64 / 16.0;
            pts.push([wd * ar * blade(t, kp, ep) + bend * len * t * t, len * t]);
        }
        let rot = ang - PI / 2.0;
        let poly = g::transform(&pts, x, y, rot, 1.0, 1.0);
        let rib = g::transform(&[[0.0, len * 0.12], [bend * len * 0.25, len * 0.7]], x, y, rot, 1.0, 1.0);
        if rgb.is_none() {
            let tip = g::transform(&[[bend * len, len]], x, y, rot, 1.0, 1.0)[0];
            let mut inner = |p: &mut Paint, _: &mut Rng| {
                // The Lua nests this path one list too deep, so its stroke is empty.
                p.stroke(&[], paint::WHITE, 0.18f64.max(len * 0.03), 0.55, NONE);
            };
            wash::wash(
                pt,
                &poly,
                rng,
                WashOpts {
                    size: len,
                    ramp: Some("leaf"),
                    part: Some("leaflets"),
                    axis: Some(([x, y], tip)),
                    light: Some(g::mix([x, y], tip, 0.3)),
                    pigment: Some(paint::BLUE),
                    pool: Some(paint::DEEP),
                    strength: Some(1.1),
                    rings: Some(4),
                    lift: Some(0.02),
                    bleed: Some(len * 0.03),
                    fade_scale: Some(0.5),
                    simple: true,
                    grain: 0.5,
                    grain_scale: Some(0.5),
                    outline: 0.75,
                    outline_w: Some(0.2f64.max(len * 0.012)),
                    inner: Some(&mut inner),
                    ..Default::default()
                },
            );
        } else {
            bodies.push(g::smooth(&poly, true));
            ribs.push(g::polyline(&rib, false));
        }
    }
    if let Some(c) = rgb {
        pt.fill_many(&bodies, c, opacity, MULT);
        pt.stroke_many(&ribs, paint::WHITE, 0.18, 0.45, NONE);
    }
}
