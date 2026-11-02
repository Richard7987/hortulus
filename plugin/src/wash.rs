//! Watercolour rendering of a closed shape, as layered glazes.

use crate::brush;
use crate::color::{self, RampInfo, Rgb};
use crate::geom::{self as g, Cmd, Pt};
use crate::lua::sort;
use crate::paint::{self, Blend, Opts, Paint, MULT, NONE};
use crate::rng::Rng;
use crate::tune::tune;
use std::f64::consts::PI;

fn ramp_fill(pt: &mut Paint, rng: &mut Rng, look: &str, base: Pt, tip: Pt, half: f64, part: Option<&str>) -> RampInfo {
    let (stops, info) = color::ramp(rng, look, part);
    pt.ramps += 1;
    pt.fill(&[Cmd::O((base[0] + tip[0]) / 2.0, (base[1] + tip[1]) / 2.0, half)], stops[stops.len() - 1].1, 1.0, NONE);
    let reach = rng.range(0.82, 1.12);
    let start = g::mix(base, tip, -rng.range(0.0, 0.18));
    let stop = g::mix(base, tip, reach);
    pt.shade(&stops, start, stop, half);
    info
}

fn inside_point(rng: &mut Rng, poly: &[Pt]) -> Pt {
    let (x0, y0, x1, y1) = g::bbox(poly);
    for _ in 0..30 {
        let x = rng.range(x0, x1);
        let y = rng.range(y0, y1);
        if g::inside(poly, x, y) {
            return [x, y];
        }
    }
    g::centroid(poly)
}

fn blot(rng: &mut Rng, c: Pt, r: f64) -> Vec<Cmd> {
    let pts: Vec<Pt> = (0..24)
        .map(|k| {
            let a = k as f64 * PI / 12.0;
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect();
    g::smooth(&g::deform(&pts, rng, r * 0.28), true)
}

fn variegate(pt: &mut Paint, rng: &mut Rng, poly: &[Pt], size: f64, info: &RampInfo, lifts: f64) {
    let cfg = &tune().wash;
    let n = rng.int(cfg.blotches[0] as i64, cfg.blotches[1] as i64);
    for _ in 0..n {
        let c = inside_point(rng, poly);
        let h = info.h + rng.jit(cfg.blotch_hue);
        let l = rng.range(0.35f64.max(info.l - 0.05), 0.82);
        let col = color::oklch(l, info.share * rng.range(0.6, 1.0) * color::max_chroma(l, h), h);
        let r = size * rng.range(0.16, 0.40);
        let b = blot(rng, c, r);
        let op = rng.rangev(cfg.blotch_opacity);
        pt.fill(&b, col, op, MULT);
    }
    let n = (rng.int(cfg.lifts[0] as i64, cfg.lifts[1] as i64) as f64 * lifts).floor() as i64;
    for _ in 0..n {
        let c = inside_point(rng, poly);
        let r = size * rng.range(0.10, 0.22);
        let b = blot(rng, c, r);
        let op = rng.rangev(cfg.lift_opacity);
        pt.fill(&b, paint::WHITE, op, NONE);
        let edge = color::oklch(info.l + 0.15, 0.6 * color::max_chroma(info.l + 0.15, info.h), info.h);
        pt.stroke(&b, edge, size * 0.012, 0.22, MULT);
    }
    if rng.chance(cfg.tideline_chance) {
        let n = poly.len() as i64;
        let i0 = rng.int(1, n);
        let m = (n as f64 * rng.range(0.25, 0.5)).floor() as i64;
        let run: Vec<Pt> = (0..=m).map(|k| poly[(i0 + k - 1).rem_euclid(n) as usize]).collect();
        if run.len() >= 3 {
            let col = color::oklch(info.l * 0.85, color::max_chroma(info.l * 0.85, info.h) * 0.8, info.h);
            pt.stroke(&g::smooth(&run, false), col, size * 0.03, 0.35, MULT);
        }
    }
}

pub fn taper(line: &[Pt], wmax: f64, head: f64) -> Vec<Pt> {
    g::ribbon(line, |t| wmax * g::smoothstep(0.0, head, t) * (1.0 - t).powf(1.1) + 0.02)
}

#[derive(Clone, Copy, Default)]
pub struct HairOpts {
    pub head: Option<f64>,
    pub blend: Option<Blend>,
}

pub fn hairlines(pt: &mut Paint, lines: &[Vec<Pt>], rgb: Rgb, wmax: f64, opacity: f64, o: HairOpts) {
    if lines.is_empty() {
        return;
    }
    let head = o.head.unwrap_or(0.12);
    let paths: Vec<Vec<Cmd>> = lines.iter().map(|l| g::smooth(&taper(l, wmax, head), true)).collect();
    pt.fill_many(&paths, rgb, opacity, Opts { blend: o.blend, evenodd: false });
}

pub struct GrainOpts {
    pub cap: f64,
    pub scale: f64,
    pub clump: f64,
    pub mottle: f64,
    pub rim: f64,
    pub dark: f64,
    pub dark_op: f64,
    pub pale_op: f64,
    pub dark_rgb: Rgb,
}

impl Default for GrainOpts {
    fn default() -> Self {
        GrainOpts { cap: 1600.0, scale: 1.0, clump: 0.0, mottle: 0.0, rim: 0.6, dark: 0.6, dark_op: 0.38, pale_op: 0.45, dark_rgb: paint::DEEP }
    }
}

pub fn grain(pt: &mut Paint, rng: &mut Rng, poly: &[Pt], density: f64, o: &GrainOpts) {
    let (x0, y0, x1, y1) = g::bbox(poly);
    let area = g::area(poly).abs();
    let n = (area * density).floor().min(o.cap);
    let c = g::centroid(poly);
    let reach = (x1 - x0).max(y1 - y0) / 2.0;
    let scale = o.scale;
    let clump = o.clump;
    let sample = g::span_sampler(poly, 0.3f64.max((y1 - y0) / 150.0));
    let budget = if sample.is_some() { n * 4.0 * area / ((x1 - x0) * (y1 - y0)).max(1e-9) } else { 0.0 };
    let mottle = o.mottle;
    if let (Some(s), true) = (&sample, mottle > 0.0) {
        let (mut bd, mut bp) = (Vec::new(), Vec::new());
        let m = 60f64.min((area / 30.0 * mottle + 0.5).floor()) as i64;
        for _ in 0..m {
            let (x, y) = s.sample(rng).unwrap();
            let r = rng.range(0.8, 2.6) * scale;
            let b = blot(rng, [x, y], r);
            if rng.chance(0.6) {
                bd.push(b);
            } else {
                bp.push(b);
            }
        }
        if !bd.is_empty() {
            pt.fill_many(&bd, o.dark_rgb, 0.10 * mottle.min(1.5), MULT);
        }
        if !bp.is_empty() {
            pt.fill_many(&bp, paint::WHITE, 0.10 * mottle.min(1.5), NONE);
        }
    }
    let mut dark: Vec<Vec<Cmd>> = Vec::new();
    let mut pale: Vec<Vec<Cmd>> = Vec::new();
    let mut tries = 0.0;
    if let Some(s) = &sample {
        while ((dark.len() + pale.len()) as f64) < n && tries < budget {
            tries += 1.0;
            let (x, y) = s.sample(rng).unwrap();
            let edge = g::dist(c, [x, y]) / reach;
            if rng.next() < 0.35 + o.rim * edge {
                let is_dark = rng.next() < o.dark;
                let dot = |dark: &mut Vec<Vec<Cmd>>, pale: &mut Vec<Vec<Cmd>>, x: f64, y: f64, rng: &mut Rng| {
                    let r = rng.range(0.10, 0.32) * scale;
                    if is_dark {
                        dark.push(vec![Cmd::O(x, y, r)]);
                    } else {
                        pale.push(vec![Cmd::O(x, y, r * 0.9)]);
                    }
                };
                if clump > 0.0 && rng.next() < clump / (5.0 - 4.0 * clump.min(1.0)) {
                    let k = rng.int(3, 7);
                    let cr = rng.range(0.5, 1.4) * scale;
                    let m = k.min(n as i64 - (dark.len() + pale.len()) as i64);
                    for _ in 0..m {
                        let dx = x + rng.jit(1.0) * cr;
                        let dy = y + rng.jit(1.0) * cr;
                        dot(&mut dark, &mut pale, dx, dy, rng);
                    }
                } else {
                    dot(&mut dark, &mut pale, x, y, rng);
                }
            }
        }
    }
    if !dark.is_empty() {
        pt.fill_many(&dark, o.dark_rgb, o.dark_op, MULT);
    }
    if !pale.is_empty() {
        pt.fill_many(&pale, paint::WHITE, o.pale_op, NONE);
    }
}

pub fn fan(rng: &mut Rng, poly: &[Pt], from: Pt, count: i64, reach0: f64, reach1: f64) -> Vec<Vec<Pt>> {
    let mut lines = Vec::with_capacity(count.max(0) as usize);
    for _ in 0..count {
        let v = poly[(rng.int(1, poly.len() as i64) - 1) as usize];
        let t0 = rng.range(0.02, 0.18);
        let t1 = rng.range(reach0, reach1);
        let a = g::mix(from, v, t0);
        let b = g::mix(from, v, t1);
        let m = g::mix(a, b, 0.5);
        let bow = rng.jit(0.06) * g::dist(a, b);
        let ang = g::angle(a, b) + PI / 2.0;
        let m = g::polar(m, ang, bow);
        lines.push(g::spline_points(&[a, m, b], 6));
    }
    lines
}

type Runs = Vec<Vec<Pt>>;

pub fn edge_runs(rng: &mut Rng, poly: &[Pt], pale_at: &dyn Fn(Pt) -> f64, share: f64) -> Option<(Runs, Runs)> {
    if share <= 0.0 {
        return None;
    }
    const N: usize = 96;
    let mut ring = poly.to_vec();
    ring.push(poly[0]);
    let mut pts = g::resample(&ring, N + 1);
    pts.pop();
    let nz = g::Noise::new(rng, 6);
    let ph = rng.int(0, 5) as f64;
    let score: Vec<f64> = pts
        .iter()
        .enumerate()
        .map(|(i, p)| 0.3 * pale_at(*p) + 0.7 * (0.5 + 0.5 * nz.at(ph + i as f64 / N as f64 * 6.0)))
        .collect();
    let mut sorted = score.clone();
    sort(&mut sorted, |a, b| a < b);
    let k = (N as f64 * (1.0 - share) + 0.5).floor() as i64;
    if k >= N as i64 {
        return None;
    }
    let cut = if k >= 1 { sorted[(k - 1) as usize] } else { f64::NEG_INFINITY };
    let lost: Vec<bool> = score.iter().map(|s| *s > cut).collect();
    let mut start = None;
    for i in 0..N {
        if lost[i] && !lost[(i + N - 1) % N] {
            start = Some(i);
            break;
        }
    }
    let start = start?;
    let (mut found_runs, mut lost_runs): (Runs, Runs) = (Vec::new(), Vec::new());
    let prev = pts[(start + N - 1) % N];
    let mut run = vec![prev];
    let mut state = true;
    for j in 0..N {
        let i = (start + j) % N;
        if lost[i] != state {
            let last = run[run.len() - 1];
            if state {
                lost_runs.push(run);
            } else {
                found_runs.push(run);
            }
            run = vec![last];
            state = lost[i];
        }
        run.push(pts[i]);
    }
    run.push(prev);
    if state {
        lost_runs.push(run);
    } else {
        found_runs.push(run);
    }
    Some((found_runs, lost_runs))
}

fn band(run: &[Pt], w: f64, taper: f64, rng: Option<(&mut Rng, f64)>, scale_at: Option<&dyn Fn(Pt) -> f64>) -> Vec<Cmd> {
    let line = g::resample(run, 4.max(run.len()));
    let last = line.len() as f64 - 1.0;
    let poly = g::ribbon(&line, |t| {
        let s = match scale_at {
            Some(f) => f(line[(t * last + 0.5).floor() as usize]),
            None => 1.0,
        };
        w * s * g::smoothstep(0.0, taper, t) * g::smoothstep(0.0, taper, 1.0 - t)
    });
    match rng {
        Some((r, wobble)) => g::smooth(&g::deform(&poly, r, wobble), true),
        None => g::smooth(&poly, true),
    }
}

fn bands(runs: &[Vec<Pt>], w: f64, taper: f64, mut rng: Option<(&mut Rng, f64)>, scale_at: Option<&dyn Fn(Pt) -> f64>) -> Vec<Vec<Cmd>> {
    let mut out = Vec::with_capacity(runs.len());
    for r in runs {
        if r.len() >= 2 {
            let pass = rng.as_mut().map(|(x, wb)| (&mut **x, *wb));
            out.push(band(r, w, taper, pass, scale_at));
        }
    }
    out
}

pub struct Rays {
    pub count: i64,
    pub reach: [f64; 2],
    pub contrast: [f64; 2],
}

#[derive(Default)]
pub struct WashOpts<'a> {
    pub size: f64,
    pub strength: Option<f64>,
    pub part: Option<&'a str>,
    pub ink: Option<&'a str>,
    pub pigment: Option<Rgb>,
    pub pool: Option<Rgb>,
    pub light: Option<Pt>,
    pub knockout: Option<f64>,
    pub bleed: Option<f64>,
    pub rings: Option<i64>,
    pub lift: Option<f64>,
    pub streaks: f64,
    pub dark_streaks: i64,
    pub rays: Option<Rays>,
    pub grain: f64,
    pub grain_scale: Option<f64>,
    pub outline: f64,
    pub outline_w: Option<f64>,
    pub lost: Option<f64>,
    pub eye_fade: Option<f64>,
    pub lifts: Option<f64>,
    pub pale_scale: Option<f64>,
    pub fade_scale: Option<f64>,
    pub simple: bool,
    pub ramp: Option<&'a str>,
    pub axis: Option<(Pt, Pt)>,
    pub inner: Option<&'a mut dyn FnMut(&mut Paint, &mut Rng)>,
    pub brush_lines: Option<&'a dyn Fn(&mut Rng, i64) -> Vec<Vec<Pt>>>,
}

pub fn wash(pt: &mut Paint, poly: &[Pt], rng: &mut Rng, o: WashOpts) {
    let size = o.size;
    let s = o.strength.unwrap_or(1.0);
    let part = o.part.or_else(|| o.ramp.and_then(color::part_of));
    let tint = o.ink.or_else(|| color::part_ink(part));
    let pig = color::recolor(o.pigment.unwrap_or(paint::BLUE), tint);
    let mut pool = color::recolor(o.pool.unwrap_or(paint::DEEP), tint);
    let mut line = color::recolor(paint::INK, tint);
    let mut grain_rgb = color::recolor(paint::DEEP, tint);
    let light = o.light.unwrap_or_else(|| g::centroid(poly));
    let body = g::smooth(poly, true);
    let cfg = &tune().wash;
    let axis = o.axis;
    let mut far = 1e-9f64;
    let ramp_axis = if o.ramp.is_some() { axis } else { None };
    if ramp_axis.is_none() {
        for p in poly {
            far = far.max(g::dist(*p, light));
        }
    }
    let pale_at = move |p: Pt| -> f64 {
        match ramp_axis {
            Some((a, b)) => {
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let dd = (dx * dx + dy * dy).max(1e-9);
                1.0 - g::clamp(((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / dd, 0.0, 1.0)
            }
            None => 1.0 - g::dist(p, light) / far,
        }
    };
    let mut erng = Rng::new(rng.int(1, 1 << 30) as f64);
    let runs = edge_runs(&mut erng, poly, &pale_at, o.lost.unwrap_or(cfg.lost_edges));

    let ko = o.knockout.unwrap_or(0.9);
    if ko > 0.0 {
        pt.fill(&body, paint::PAPER, ko, NONE);
    }
    let bleed = o.bleed.unwrap_or(size * 0.022);
    for _ in 0..2 {
        let d = g::smooth(&g::deform(poly, rng, bleed), true);
        pt.fill(&d, pig, 0.10 * s, MULT);
    }
    let has_ramp = o.ramp.is_some();
    if let Some(ramp) = o.ramp {
        pt.clip_begin(&body);
        let (a0, a1) = axis.expect("axis");
        let info = ramp_fill(pt, rng, ramp, a0, a1, size * 1.5, o.part);
        if let Some(ink) = info.ink {
            pool = color::recolor(o.pool.unwrap_or(paint::DEEP), Some(ink));
            line = color::recolor(paint::INK, Some(ink));
            grain_rgb = color::recolor(paint::DEEP, Some(ink));
        }
        let k = g::clamp(size / 30.0, 0.3, 1.3);
        let nd = (rng.rangev(cfg.brush_bands) * k + 0.5).floor() as i64;
        let np = (rng.rangev(cfg.brush_pale) * k * o.pale_scale.unwrap_or(1.0) + 0.5).floor() as i64;
        let (dark, pale) = match o.brush_lines {
            Some(f) => {
                let d = f(rng, nd);
                let p = f(rng, np);
                (d, p)
            }
            None => {
                let d = fan(rng, poly, light, nd, 0.6, 1.0);
                let p = fan(rng, poly, light, np, 0.5, 0.98);
                (d, p)
            }
        };
        if !o.simple {
            brush::paint(pt, rng, &dark, &pale, size, &info, cfg);
            variegate(pt, rng, poly, size, &info, o.lifts.unwrap_or(1.0));
        }
    } else {
        pt.fill(&body, pig, (0.85 * s).min(0.95), MULT);
        pt.clip_begin(&body);
        let rings = o.rings.unwrap_or(12);
        for k in 1..=rings {
            let sc = 1.0 - (k as f64 / (rings as f64 + 1.0)).powf(0.75) * 0.97;
            let layer = g::deform(&g::toward(poly, light[0], light[1], sc), rng, size * 0.018 * sc);
            pt.fill(&g::smooth(&layer, true), paint::WHITE, o.lift.unwrap_or(0.10), NONE);
        }
    }

    let (op_a, op_b) = if has_ramp { (0.12, 0.26) } else { (0.16, 0.30) };
    if let Some((found, _)) = &runs {
        let b = bands(found, size * 0.08, 0.2, None, None);
        pt.fill_many(&b, pool, op_a * s, MULT);
        let b = bands(found, size * 0.025, 0.2, None, None);
        pt.fill_many(&b, pool, op_b * s, MULT);
    } else {
        pt.stroke(&body, pool, size * 0.16, op_a * s, MULT);
        pt.stroke(&body, pool, size * 0.05, op_b * s, MULT);
    }

    let streaks = (o.streaks * if has_ramp { cfg.streak_scale } else { 1.0 }).floor() as i64;
    if streaks > 0 {
        let lines = fan(rng, poly, light, streaks, 0.55, 0.95);
        let half = lines.len() / 2;
        hairlines(pt, &lines[..half], paint::WHITE, size * 0.022, 0.75, HairOpts::default());
        hairlines(pt, &lines[half..], paint::WHITE, size * 0.013, 0.55, HairOpts::default());
    }
    if let Some(r) = &o.rays {
        if r.count > 0 {
            let lines = fan(rng, poly, light, r.count, r.reach[0], r.reach[1]);
            let (lo, hi) = (r.contrast[0], r.contrast[1]);
            let thirds = 1.max(lines.len() / 3);
            for k in 0..3usize {
                let from = (k * thirds).min(lines.len());
                let to = if k == 2 { lines.len() } else { ((k + 1) * thirds).min(lines.len()) };
                let part: &[Vec<Pt>] = if from < to { &lines[from..to] } else { &[] };
                hairlines(pt, part, paint::WHITE, size * (0.007 + 0.004 * k as f64), lo + (hi - lo) * k as f64 / 2.0, HairOpts::default());
            }
        }
    }
    if o.dark_streaks > 0 {
        let lines = fan(rng, poly, light, o.dark_streaks, 0.4, 0.98);
        hairlines(pt, &lines, pool, size * 0.010, cfg.vein_opacity * s, HairOpts { head: None, blend: Some(Blend::Multiply) });
    }

    let mut inner = o.inner;
    if let Some(f) = inner.as_mut() {
        f(pt, rng);
    }

    if o.grain > 0.0 {
        grain(
            pt,
            rng,
            poly,
            o.grain,
            &GrainOpts {
                scale: o.grain_scale.unwrap_or(1.0),
                dark_op: 0.30 * s + 0.08,
                clump: cfg.grain_clump,
                mottle: cfg.grain_mottle,
                dark_rgb: grain_rgb,
                ..Default::default()
            },
        );
    }

    if let (Some((_, lost)), true) = (&runs, cfg.lost_fade > 0.0) {
        let mut ring = poly.to_vec();
        ring.push(poly[0]);
        let thick = 2.0 * g::area(poly).abs() / g::length(&ring).max(1e-9);
        let d = size.min(1.6 * thick) * cfg.lost_fade * o.fade_scale.unwrap_or(1.0);
        let ef = o.eye_fade.unwrap_or(0.0);
        let scale = |p: Pt| 1.0 - ef + ef * g::smoothstep(0.35, 0.9, pale_at(p));
        let scale_at: Option<&dyn Fn(Pt) -> f64> = if ef > 0.0 { Some(&scale) } else { None };
        let layers = if o.simple { 3 } else { 7 };
        for k in 1..=layers {
            let w = d * (0.36 - 0.045 * (k - 1) as f64 * 7.0 / layers as f64);
            let b = bands(lost, w, 0.4 - 0.03 * k as f64, Some((&mut erng, w * 0.25)), scale_at);
            pt.fill_many(&b, paint::WHITE, 0.22, NONE);
        }
    }

    pt.clip_end();

    if o.outline > 0.0 {
        let w = o.outline_w.unwrap_or(0.28);
        if let Some((found, _)) = &runs {
            let b = bands(found, w / 2.0, 0.25, None, None);
            pt.fill_many(&b, line, o.outline, MULT);
        } else {
            pt.stroke(&body, line, w, o.outline, MULT);
        }
    }
}
