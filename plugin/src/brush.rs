//! Broken, directional brushwork: pigment bands and pale striations.

use crate::color::{self, RampInfo, Rgb};
use crate::geom::{self as g, Cmd, Noise, Pt};
use crate::paint::{self, Opts, Paint, MULT, NONE};
use crate::rng::Rng;
use crate::tune::{palette, tune, PlantCfg, Wash};
use std::f64::consts::PI;

pub fn polygon(poly: &[Pt]) -> Vec<Cmd> {
    let mut cmds = Vec::with_capacity(poly.len() + 1);
    cmds.push(Cmd::M(poly[0][0], poly[0][1]));
    for p in &poly[1..] {
        cmds.push(Cmd::L(p[0], p[1]));
    }
    cmds.push(Cmd::Z);
    cmds
}

fn slice(pts: &[Pt], t0: f64, t1: f64) -> Vec<Pt> {
    let n = pts.len() as i64;
    let i0 = 1.max((t0 * (n - 1) as f64).floor() as i64 + 1);
    let i1 = n.min((t1 * (n - 1) as f64).ceil() as i64 + 1);
    let mut out = Vec::new();
    let mut i = i0;
    while i <= i1 {
        out.push(pts[(i - 1) as usize]);
        i += 1;
    }
    out
}

fn offset(pts: &[Pt], d: f64) -> Vec<Pt> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let a = g::angle(pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]) + PI / 2.0;
            g::polar(pts[i], a, d)
        })
        .collect()
}

pub fn broken(rng: &mut Rng, line: &[Pt], wmax: f64, gap: f64, dry: f64) -> Vec<Vec<Pt>> {
    let pts = g::resample(line, 32);
    let nz = Noise::new(rng, 16);
    let phase = rng.range(0.0, 16.0);
    let mut out = Vec::new();
    let mut t = rng.range(0.0, 0.12);
    while t < 0.95 {
        let t1 = (t + rng.range(0.14, 0.5) * (1.0 - 0.6 * gap)).min(1.0);
        let seg = slice(&pts, t, t1);
        if seg.len() >= 2 {
            let seg = g::resample(&seg, 10);
            let (a, b) = (t, t1);
            let w = |u: f64| {
                let tt = a + (b - a) * u;
                let taper = g::smoothstep(0.0, 0.12, tt) * (1.0 - tt).powf(0.7);
                let ends = (PI * g::clamp(u, 0.0, 1.0)).sin().powf(0.6);
                let swell = 0.5 + 0.5 * nz.at(phase + tt * 9.0);
                wmax * taper * ends * (0.2 + swell)
            };
            if rng.chance(dry) {
                let k = rng.int(2, 3);
                for j in 1..=k {
                    let d = (j as f64 - (k as f64 + 1.0) / 2.0) * wmax * 0.9;
                    out.push(g::ribbon(&offset(&seg, d), |u| w(u) * rng.range(0.25, 0.45)));
                }
            } else {
                out.push(g::ribbon(&seg, w));
            }
        }
        t = t1 + rng.range(0.02, 0.12) * gap * 2.5;
    }
    out
}

pub fn polygons(pieces: &[Vec<Pt>]) -> Vec<Vec<Cmd>> {
    pieces.iter().map(|p| polygon(p)).collect()
}

pub fn paint(pt: &mut Paint, rng: &mut Rng, dark: &[Vec<Pt>], pale: &[Vec<Pt>], size: f64, info: &RampInfo, cfg: &Wash) {
    for line in dark {
        let w = size * rng.range(0.025, 0.06) * cfg.brush_width;
        let pieces = broken(rng, line, w, cfg.brush_gap, cfg.brush_dry);
        if !pieces.is_empty() {
            let l = g::clamp(info.l + rng.range(0.0, 0.25), 0.25, 0.9);
            let h = info.h + rng.jit(6.0);
            let col = color::oklch(l, info.share * color::max_chroma(l, h), h);
            let cmds = polygons(&pieces);
            let op = rng.rangev(cfg.band_opacity);
            pt.fill_many(&cmds, col, op, MULT);
        }
    }
    for line in pale {
        let w = size * rng.range(0.012, 0.036) * cfg.brush_width;
        let pieces = broken(rng, line, w, cfg.brush_gap, cfg.brush_dry);
        if !pieces.is_empty() {
            let cmds = polygons(&pieces);
            let op = rng.rangev(cfg.pale_contrast);
            pt.fill_many(&cmds, paint::WHITE, op, NONE);
        }
    }
}

fn offset_by(pts: &[Pt], wfn: &dyn Fn(f64) -> f64, f: f64) -> Vec<Pt> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let a = g::angle(pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]) + PI / 2.0;
            g::polar(pts[i], a, f * wfn(i as f64 / 1f64.max(n as f64 - 1.0)))
        })
        .collect()
}

pub fn stem_tone(cfg: &PlantCfg) -> Rgb {
    let h = tune().color.hue;
    if let Some(ink) = color::ink_first(palette("stems")) {
        return color::ink_rgb(ink, cfg.stem_l, 1.0);
    }
    color::oklch(cfg.stem_l, 0.85 * color::max_chroma(cfg.stem_l, h), h)
}

pub fn stem(pt: &mut Paint, rng: &mut Rng, pts: &[Pt], wfn: &dyn Fn(f64) -> f64, cfg: &PlantCfg) {
    let h = tune().color.hue;
    let l = cfg.stem_l;
    let ink = color::ink_for(rng, palette("stems"));
    let tone = |l: f64, k: f64| match ink {
        Some(i) => color::ink_rgb(i, l, 1.0),
        None => color::oklch(l, k * color::max_chroma(l, h), h),
    };
    let body = tone(l, 0.85);
    let outline = g::smooth(&g::ribbon(pts, |t| wfn(t) / 2.0), true);
    pt.fill(&outline, body, 0.55 + 0.35 * (1.0 - cfg.stem_brush), MULT);
    if cfg.stem_brush <= 0.0 {
        return;
    }
    let line = g::resample(pts, 8f64.max((g::length(pts) / 3.0).floor()) as usize);
    let mut wmax: f64 = 0.0;
    for i in 0..=10 {
        wmax = wmax.max(wfn(i as f64 / 10.0));
    }
    pt.clip_begin(&outline);
    let dark = tone(l * 0.7, 0.9);
    let draw = |pt: &mut Paint, rng: &mut Rng, f: f64, w: f64, gap: f64, dry: f64, col: Rgb, op: f64, o: Opts| {
        let pieces = broken(rng, &offset_by(&line, wfn, f), w, gap, dry);
        let cmds = polygons(&pieces);
        if !cmds.is_empty() {
            pt.fill_many(&cmds, col, op * cfg.stem_brush, o);
        }
    };
    draw(pt, rng, -0.22, wmax * 0.32, 0.3, 0.2, dark, 0.65, MULT);
    draw(pt, rng, 0.24, wmax * 0.16, 0.45, 0.3, paint::WHITE, 0.55, NONE);
    draw(pt, rng, 0.5, wmax * 0.3, 0.75, 0.0, paint::WHITE, 0.6, NONE);
    pt.clip_end();
    draw(pt, rng, -0.5, 0.25f64.max(wmax * 0.09), 0.2, 0.0, paint::INK, 0.7, MULT);
}
