//! The ornament kinds: cartouches, marks, pendants, crests, flourishes,
//! tailpieces, corners and the larger gilt.

use crate::color::{self, Rgb};
use crate::geom::{self as g, Cmd, Pt};
use crate::lua::fmod;
use crate::orn_gild::{self as gild, FillOpts};
use crate::orn_kit::{self as k, RayOpts};
use crate::orn_pen::{self as pen, Pointed, Shape};
use crate::ornament::{self as o, Field, Sky, Sparkle};
use crate::ornaments::{DrawOpts, Kind};
use crate::paint::{self, Paint, MULT, NONE};
use crate::rng::Rng;
use std::f64::consts::PI;

const UMBER: Rgb = [0.330, 0.200, 0.090];

pub fn draw(kind: &Kind, pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    match kind.name {
        "night" => o::panel(pt, rng, b[0], b[1], b[2] - b[0], b[3] - b[1]),
        "compass" => o::compass(pt, (b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, (b[2] - b[0]) / 2.0, 1.5, 8),
        "knot" => o::pendant(pt, rng, (b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, b[2] - b[0], b[3] - b[1], d.cord),
        "sun" => sun(pt, rng, b),
        "moon" => moon(pt, rng, b),
        "constellation" => constellation(pt, rng, b),
        "planet" => planet(pt, rng, b),
        "armillary" => armillary(pt, rng, b),
        "scrolled" => scrolled(pt, rng, b),
        "strapwork" => strapwork(pt, rng, b),
        "rosette" => {
            let (x, y, _, _, s) = k::measure(b);
            let n: i64 = rng.weighted(&[("6", 1.0), ("8", 2.0)]).unwrap().parse().unwrap();
            rosette(pt, x, y, s * 0.5, n, PI / 2.0);
        }
        "sunburst" => sunburst(pt, b),
        "quatrefoil" => quatrefoil_mark(pt, b),
        "medallion" => medallion(pt, rng, b, d),
        "zodiac_pendant" => {
            let (x, _, w, h, _) = k::measure(b);
            let r = w.min(h) * 0.48;
            cord(pt, x, b[3], d.cord, 0.45);
            zodiac(pt, x, b[3] - r - 0.5, r, rng);
        }
        "sun_crest" => sun_crest(pt, rng, b),
        "compass_rose" => compass_rose(pt, b),
        "armillary_crest" => {
            let (x, y, _, _, s) = k::measure(b);
            let tilt = rng.range(0.35, 0.5);
            k::armillary(pt, x, y + s * 0.1, s * 0.32, tilt, b[1] + s * 0.04);
        }
        "zodiac" => {
            let (x, y, _, _, s) = k::measure(b);
            zodiac(pt, x, y, s * 0.47, rng);
        }
        "wreath" => wreath(pt, rng, b),
        "fleuron_rule" => fleuron_rule(pt, b),
        "headpiece" => headpiece(pt, rng, b),
        "cul_de_lampe" => cul_de_lampe(pt, rng, b),
        "hedera" => hedera(pt, rng, b),
        "garland" => garland(pt, rng, b),
        "bracket" => bracket(pt, b, d),
        "corner_leaf" => corner_leaf(pt, b, d),
        "copperplate" => copperplate(pt, rng, b),
        "swash" => swash(pt, rng, b),
        "swelled_rule" => swelled_rule(pt, rng, b),
        "fleuronnee" => fleuronnee(pt, rng, b),
        "rinceau" => rinceau(pt, rng, b),
        "acanthus_corner" => acanthus_corner(pt, rng, b, d),
        "cornerpiece" => cornerpiece(pt, rng, b, d),
        "pen_corner" => pen_corner(pt, rng, b, d),
        "fan" => fan(pt, rng, b, d),
        _ => {
            let s = (b[2] - b[0]).min(b[3] - b[1]);
            o::sparkles(pt, &[Sparkle { x: (b[0] + b[2]) / 2.0, y: (b[1] + b[3]) / 2.0, r: s * 0.3, kind: kind.sparkle }]);
        }
    }
}

fn cartouche(
    pt: &mut Paint,
    rng: &mut Rng,
    b: [f64; 4],
    sky: Sky,
    inner: impl FnOnce(&mut Paint, &mut Rng, &Field, f64, f64, f64, f64),
    over: impl FnOnce(&mut Paint, &mut Rng, &Field, f64, f64, f64, f64),
) {
    let f = o::panel_open(pt, rng, b[0], b[1], b[2] - b[0], b[3] - b[1], sky);
    let (w, h) = (f.x1 - f.x0, f.y1 - f.y0);
    let (cx, cy) = ((f.x0 + f.x1) / 2.0, (f.y0 + f.y1) / 2.0);
    pt.clip_begin(&g::polyline(&f.body, true));
    inner(pt, rng, &f, w, h, cx, cy);
    pt.clip_end();
    o::panel_close(pt, &f);
    over(pt, rng, &f, w, h, cx, cy);
}

fn stars(pt: &mut Paint, rng: &Rng, f: &Field) {
    o::starfield(pt, &mut rng.fork("stars"), &[], f.x0, f.y0, f.x1, f.y1);
}

fn sun(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    cartouche(
        pt,
        rng,
        b,
        Sky { pigment: Some(paint::SKY), strength: Some(0.45), light: Some([(b[0] + b[2]) / 2.0, b[1]]), pool: None },
        |pt, _, _, w, h, cx, cy| {
            let r = w.min(h) * 0.2;
            pt.fill_many(&k::ring(cx, cy, r * 2.05, 24, r * 0.05, PI / 2.0), paint::WHITE, 0.7, NONE);
            for (i, rr) in [1.9, 1.5, 1.2].iter().enumerate() {
                pt.fill(&k::dot(cx, cy, r * rr), paint::CREAM, 0.18 + 0.08 * (i + 1) as f64, NONE);
            }
        },
        |pt, rng, _, w, h, cx, cy| {
            let r = w.min(h) * 0.2;
            let rot = rng.range(0.0, PI / 8.0);
            let (lit, shade) = k::rays(cx, cy, r * 1.02, w.min(h) * 0.47, 16, RayOpts { flames: true, rot, ..Default::default() });
            k::gilt(pt, &lit, &shade, 0.95);
            pt.fill(&k::dot(cx, cy, r), paint::GOLD, 0.95, NONE);
            pt.fill(&k::dot(cx - r * 0.18, cy + r * 0.2, r * 0.7), paint::CREAM, 0.55, NONE);
            pt.stroke(&k::dot(cx, cy, r), paint::COPPER, 0.3, 0.9, NONE);
            pt.stroke(&k::dot(cx, cy, r * 0.82), paint::COPPER, 0.18, 0.6, NONE);
        },
    );
}

fn moon(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    cartouche(
        pt,
        rng,
        b,
        Sky::default(),
        |pt, rng, f, w, h, cx, cy| {
            stars(pt, rng, f);
            let r = w.min(h) * 0.3;
            let x = cx + rng.jit(w * 0.06);
            let y = cy + h * 0.06;
            let rot = rng.range(-0.5, 0.2);
            pt.fill(&k::dot(x, y, r), paint::WHITE, 0.08, NONE);
            let m = g::smooth(&k::crescent(x, y, r, 0.62, rot), true);
            pt.fill(&m, paint::WHITE, 0.95, NONE);
            pt.fill(&g::smooth(&k::crescent(x, y, r * 0.98, 0.42, rot), true), f.dilute, 0.16, NONE);
            let mut seas = Vec::new();
            for _ in 0..5 {
                let a = rot + PI + rng.jit(1.1);
                let p = g::polar([x, y], a, r * rng.range(0.62, 0.86));
                seas.push(k::dot(p[0], p[1], r * rng.range(0.05, 0.11)));
            }
            pt.fill_many(&seas, f.dilute, 0.16, NONE);
            pt.fill(&k::dot(x, y, r * 1.35), paint::WHITE, 0.06, NONE);
        },
        |pt, _, f, w, h, cx, _| {
            let s = w.min(h) * 0.08;
            let p = [cx - w * 0.22, f.y0 + h * 0.2];
            pt.fill(&g::polyline(&k::star(p[0], p[1], s, s * 0.38, 5, PI / 2.0), true), paint::GOLD, 0.9, NONE);
        },
    );
}

fn constellation(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    cartouche(
        pt,
        rng,
        b,
        Sky { strength: Some(1.2), ..Default::default() },
        |pt, rng, f, w, h, cx, cy| {
            stars(pt, rng, f);
            let mut band = Vec::new();
            let mut br = rng.fork("band");
            let a = br.range(0.5, 1.1);
            for _ in 0..(w * h * 0.12).floor() as i64 {
                let t = br.range(-0.7, 0.7);
                let u = br.jit(1.0) * br.next();
                let p = g::polar([cx + t * w * a.cos(), cy + t * h * a.sin()], a + PI / 2.0, u * w * 0.18);
                band.push(k::dot(p[0], p[1], br.range(0.08, 0.22)));
            }
            pt.fill_many(&band, paint::WHITE, 0.45, NONE);
        },
        |pt, rng, f, w, h, _, _| {
            let mut cr = rng.fork("figure");
            let mut pts: Vec<Pt> = Vec::new();
            let px = f.x0 + w * cr.range(0.15, 0.3);
            let py = f.y0 + h * cr.range(0.15, 0.3);
            let mut p = [px, py];
            for _ in 0..cr.int(6, 8) {
                pts.push(p);
                let nx = g::clamp(p[0] + cr.range(-0.15, 0.32) * w, f.x0 + w * 0.12, f.x1 - w * 0.12);
                let ny = g::clamp(p[1] + cr.range(0.06, 0.2) * h, f.y0 + h * 0.1, f.y1 - h * 0.1);
                p = [nx, ny];
            }
            let mut lines = vec![g::polyline(&pts, false)];
            let j = cr.int(2, pts.len() as i64 - 1) as usize;
            let spur = [pts[j - 1][0] + w * 0.18, pts[j - 1][1] - h * 0.08];
            lines.push(g::polyline(&[pts[j - 1], spur], false));
            pts.push(spur);
            pt.stroke_many(&lines, paint::GOLD, 0.25, 0.75, NONE);
            let glints: Vec<Vec<Cmd>> = pts
                .iter()
                .enumerate()
                .map(|(i, q)| {
                    let s = w.min(h) * if (i + 1) % 3 == 1 { 0.06 } else { 0.04 };
                    g::polyline(&k::star(q[0], q[1], s, s * 0.32, 4, PI / 2.0), true)
                })
                .collect();
            pt.fill_many(&glints, paint::GOLD, 0.95, NONE);
        },
    );
}

fn planet(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    cartouche(
        pt,
        rng,
        b,
        Sky::default(),
        |pt, rng, f, w, h, cx, cy| {
            stars(pt, rng, f);
            let r = w.min(h) * 0.24;
            let tilt = rng.range(-0.45, -0.2);
            let half = |kk: f64, front: bool| {
                let pts = k::ellipse(cx, cy, r * kk, r * kk * 0.28, tilt, 48);
                let (a, z) = if front { (1, 25) } else { (25, 49) };
                let out: Vec<Pt> = (a..=z).map(|i| pts[(i - 1) % 48]).collect();
                g::polyline(&out, false)
            };
            pt.stroke_many(&[half(2.0, false), half(1.65, false)], paint::GOLD, 0.5, 0.8, NONE);
            let disc = k::dot(cx, cy, r);
            pt.fill(&disc, paint::CREAM, 1.0, NONE);
            pt.clip_begin(&disc);
            for kk in -4i64..=4 {
                let pts = k::ellipse(cx, cy + kk as f64 * r * 0.24, r * 1.3, r * 0.07, tilt * 0.6, 24);
                let col = if kk.rem_euclid(2) == 0 { paint::COPPER } else { paint::GOLD };
                pt.fill(&g::smooth(&pts, true), col, 0.22 + 0.04 * kk.rem_euclid(3) as f64, NONE);
            }
            pt.fill(&g::smooth(&k::crescent(cx, cy, r * 1.02, 0.7, tilt + PI * 0.8), true), f.deep, 0.45, NONE);
            pt.clip_end();
            pt.stroke_many(&[half(2.0, true), half(1.65, true)], paint::GOLD, 0.5, 0.9, NONE);
            let m = g::polar([cx, cy], rng.range(0.6, 1.2), r * 2.15);
            pt.fill(&k::dot(m[0], m[1], r * 0.12), paint::WHITE, 0.9, NONE);
        },
        |_, _, _, _, _, _, _| {},
    );
}

fn armillary(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    cartouche(
        pt,
        rng,
        b,
        Sky { pigment: Some(paint::SKY), strength: Some(0.55), ..Default::default() },
        |pt, rng, f, _, _, _, _| stars(pt, rng, f),
        |pt, rng, f, w, h, cx, cy| {
            let tilt = rng.range(0.35, 0.5);
            k::armillary(pt, cx, cy + h * 0.06, w.min(h) * 0.32, tilt, f.y0 + h * 0.05);
        },
    );
}

fn ground() -> Rgb {
    color::recolor(paint::SKY, color::part_ink(Some("cartouche")))
}

fn vignette(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, s: f64) {
    let kind = rng.weighted(&[("sun", 3.0), ("moon", 2.0), ("star", 2.0), ("rosette", 2.0)]).unwrap();
    match kind {
        "sun" => {
            let (lit, shade) = k::rays(x, y, s * 0.42, s, 12, RayOpts { flames: true, ..Default::default() });
            k::gilt(pt, &lit, &shade, 0.95);
            pt.fill(&k::dot(x, y, s * 0.42), paint::GOLD, 0.95, NONE);
            pt.fill(&k::dot(x - s * 0.08, y + s * 0.08, s * 0.28), paint::CREAM, 0.5, NONE);
        }
        "moon" => {
            let rot = rng.range(-0.4, 0.4);
            pt.fill(&g::smooth(&k::crescent(x, y, s * 0.8, 0.6, rot), true), paint::GOLD, 0.95, NONE);
            let a0 = rng.range(0.0, PI);
            pt.fill_many(&k::ring(x, y, s * 1.05, 5, s * 0.06, a0), paint::GOLD, 0.8, NONE);
        }
        "star" => {
            let (lit, shade) = k::rays(x, y, s * 0.12, s, 8, RayOpts { short: 0.5, width: 0.28, ..Default::default() });
            k::gilt(pt, &lit, &shade, 0.95);
            pt.fill(&k::dot(x, y, s * 0.1), paint::CREAM, 1.0, NONE);
        }
        _ => {
            let petals: Vec<Vec<Cmd>> = (0..8)
                .map(|i| {
                    let a = i as f64 * PI / 4.0;
                    let p = g::polar([x, y], a, s * 0.52);
                    g::smooth(&k::ellipse(p[0], p[1], s * 0.42, s * 0.2, a, 16), true)
                })
                .collect();
            pt.fill_many(&petals, paint::GOLD, 0.7, NONE);
            pt.fill(&k::dot(x, y, s * 0.24), paint::COPPER, 0.9, NONE);
        }
    }
}

fn scroll(pt: &mut Paint, p: Pt, ang: f64, s: f64, dir: f64, w: f64) {
    let pts = k::scroll(p, ang, s, dir, 1.15);
    let shadow: Vec<Pt> = pts.iter().map(|q| [q[0] + w * 0.25, q[1] - w * 0.25]).collect();
    pt.fill(&k::taper(&shadow, w, 0.25), paint::COPPER, 0.7, NONE);
    pt.fill(&k::taper(&pts, w, 0.25), paint::GOLD, 0.95, NONE);
}

fn scrolled(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, s) = k::measure(b);
    let (rx, ry) = (w * 0.27, h * 0.27);
    let oval = g::smooth(&k::ellipse(cx, cy, rx, ry, 0.0, 40), true);
    pt.fill(&oval, ground(), 0.3, NONE);
    pt.fill(&g::smooth(&k::ellipse(cx, cy + ry * 0.15, rx * 0.8, ry * 0.7, 0.0, 32), true), paint::WHITE, 0.35, NONE);
    let sw = s * 0.05;
    for d in [-1.0, 1.0] {
        scroll(pt, [cx + d * rx * 0.25, cy + ry * 0.98], PI / 2.0 + d * -0.9, s * 0.34, -d, sw);
        scroll(pt, [cx + d * rx * 0.25, cy - ry * 0.98], -PI / 2.0 + d * 0.9, s * 0.34, d, sw);
        scroll(pt, [cx + d * rx * 0.98, cy + ry * 0.2], if d > 0.0 { -0.3 } else { PI + 0.3 }, s * 0.26, d, sw * 0.9);
    }
    pt.stroke_many(&[oval, g::smooth(&k::ellipse(cx, cy, rx * 0.9, ry * 0.92, 0.0, 40), true)], paint::GOLD, 0.55, 0.95, NONE);
    let beads: Vec<Vec<Cmd>> = (0..32)
        .map(|i| {
            let a = i as f64 * PI / 16.0;
            k::dot(cx + rx * 0.95 * a.cos(), cy + ry * 0.96 * a.sin(), s * 0.012)
        })
        .collect();
    pt.fill_many(&beads, paint::GOLD, 0.9, NONE);
    pt.fill_many(&k::trefoil(cx, cy + ry + s * 0.17, s * 0.13, PI / 2.0), paint::GOLD, 0.95, NONE);
    pt.fill(&g::smooth(&k::leaf(cx, cy - ry - s * 0.06, s * 0.15, -PI / 2.0, 1.0), true), paint::GOLD, 0.95, NONE);
    vignette(pt, rng, cx, cy, s * 0.17);
}

fn shield(cx: f64, top: f64, w: f64, h: f64, kk: f64) -> Vec<Pt> {
    let pts: Vec<Pt> = (0..=10)
        .map(|i| {
            let t = i as f64 / 10.0;
            [cx + w / 2.0 * kk * (1.0 - t.powf(2.2)), top - h * kk * t]
        })
        .collect();
    let mut out = vec![[cx - w / 2.0 * kk, top]];
    out.extend_from_slice(&pts);
    for i in (0..pts.len() - 1).rev() {
        out.push([2.0 * cx - pts[i][0], pts[i][1]]);
    }
    out
}

fn strapwork(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, s) = k::measure(b);
    let (top, sw, sh) = (cy + h * 0.27, w * 0.62, h * 0.6);
    let outer = g::polyline(&shield(cx, top, sw, sh, 1.0), true);
    let inner = g::polyline(&shield(cx, top - s * 0.05, sw, sh - s * 0.06, 0.84), true);
    pt.fill(&outer, ground(), 0.3, NONE);
    let mut band = Vec::new();
    for d in [-1.0, 1.0] {
        let corner = [cx + d * sw / 2.0, top];
        band.push(g::smooth(&k::ellipse(corner[0] + d * s * 0.07, corner[1] + s * 0.06, s * 0.1, s * 0.07, d * 0.5, 20), true));
        let side = [cx + d * sw * 0.52, top - sh * 0.38];
        band.push(g::smooth(&k::ellipse(side[0] + d * s * 0.05, side[1], s * 0.06, s * 0.11, 0.0, 20), true));
    }
    pt.stroke_many(&band, paint::COPPER, 1.6, 0.55, NONE);
    pt.stroke_many(&band, paint::GOLD, 1.0, 0.95, NONE);
    let both = [outer, inner];
    pt.stroke_many(&both, paint::COPPER, 0.9, 0.5, NONE);
    pt.stroke_many(&both, paint::GOLD, 0.55, 0.95, NONE);
    let mid = shield(cx, top - s * 0.025, sw, sh - s * 0.03, 0.92);
    let rivets: Vec<Vec<Cmd>> = mid.iter().step_by(2).map(|q| k::dot(q[0], q[1], s * 0.016)).collect();
    pt.fill_many(&rivets, paint::CREAM, 0.95, NONE);
    for kk in [-1.0, 0.0, 1.0] {
        pt.fill_many(&k::trefoil(cx + kk * sw * 0.3, top + s * 0.12, s * if kk == 0.0 { 0.12 } else { 0.08 }, PI / 2.0), paint::GOLD, 0.9, NONE);
    }
    vignette(pt, rng, cx, top - sh * 0.42, s * 0.16);
}

fn rosette(pt: &mut Paint, x: f64, y: f64, r: f64, n: i64, rot: f64) {
    let (mut lit, mut shade) = (Vec::new(), Vec::new());
    let nf = n as f64;
    for i in 0..n {
        let a = rot + i as f64 * 2.0 * PI / nf;
        let tip = g::polar([x, y], a, r);
        let l = g::polar([x, y], a + PI / nf * 0.9, r * 0.42);
        let rr = g::polar([x, y], a - PI / nf * 0.9, r * 0.42);
        lit.push(g::smooth_t(&[[x, y], l, tip], true, 0.4));
        shade.push(g::smooth_t(&[[x, y], tip, rr], true, 0.4));
    }
    k::gilt(pt, &lit, &shade, 0.95);
    pt.fill(&k::dot(x, y, r * 0.22), paint::COPPER, 0.95, NONE);
    pt.fill(&k::dot(x - r * 0.04, y + r * 0.04, r * 0.1), paint::CREAM, 0.9, NONE);
}

fn quatrefoil(x: f64, y: f64, r: f64, rot: f64) -> Vec<Cmd> {
    let pts: Vec<Pt> = (0..64)
        .map(|i| {
            let a = i as f64 * 2.0 * PI / 64.0;
            let rr = r * (0.72 + 0.28 * (2.0 * (a - rot)).cos().abs().powf(0.6));
            g::polar([x, y], a, rr)
        })
        .collect();
    g::smooth(&pts, true)
}

fn cord(pt: &mut Paint, x: f64, y: f64, len: f64, r: f64) {
    if len < 4.0 {
        return;
    }
    let mut beads = Vec::new();
    let mut d = r * 2.0;
    while d <= len {
        beads.push(k::dot(x, y + d, r));
        d += r * 2.6;
    }
    pt.fill_many(&beads, paint::GOLD, 0.85, NONE);
}

fn sunburst(pt: &mut Paint, b: [f64; 4]) {
    let (x, y, _, _, s) = k::measure(b);
    let (lit, shade) = k::rays(x, y, s * 0.17, s * 0.5, 16, RayOpts::default());
    k::gilt(pt, &lit, &shade, 0.95);
    pt.stroke_many(&[g::polyline(&[[x - s * 0.7, y], [x + s * 0.7, y]], false)], paint::GOLD, 0.2, 0.7, NONE);
    pt.fill(&k::dot(x, y, s * 0.17), paint::GOLD, 0.95, NONE);
    pt.fill(&k::dot(x, y, s * 0.08), paint::CREAM, 0.95, NONE);
}

fn quatrefoil_mark(pt: &mut Paint, b: [f64; 4]) {
    let (x, y, _, _, s) = k::measure(b);
    pt.stroke_many(&[quatrefoil(x, y, s * 0.48, PI / 4.0), quatrefoil(x, y, s * 0.36, PI / 4.0)], paint::GOLD, 0.35, 0.95, NONE);
    let (lit, shade) = k::rays(x, y, s * 0.04, s * 0.26, 4, RayOpts { short: 1.0, width: 0.4, rot: 0.0, flames: false });
    k::gilt(pt, &lit, &shade, 0.95);
    pt.fill_many(&k::ring(x, y, s * 0.42, 4, s * 0.035, PI / 4.0), paint::GOLD, 0.9, NONE);
}

fn medallion(pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    let (x, _, w, h, _) = k::measure(b);
    let r = w.min(h * 0.78) * 0.48;
    let cy = b[3] - r - 0.5;
    cord(pt, x, b[3], d.cord, 0.45);
    pt.stroke_many(&[k::dot(x, cy, r), k::dot(x, cy, r * 0.84)], paint::GOLD, 0.32, 0.9, NONE);
    let rot = rng.range(0.0, PI / 8.0);
    rosette(pt, x, cy, r * 0.62, 8, rot);
    pt.fill(&g::smooth(&k::lozenge(x, cy - r - (h - 2.0 * r) * 0.35, 1f64.max((h - 2.0 * r) * 0.55), -PI / 2.0), true), paint::GOLD, 0.9, NONE);
}

fn zodiac(pt: &mut Paint, x: f64, y: f64, r: f64, rng: &mut Rng) {
    pt.stroke_many(&[k::dot(x, y, r), k::dot(x, y, r * 0.82), k::dot(x, y, r * 0.4)], paint::GOLD, 0.3f64.max(r * 0.03), 0.9, NONE);
    let spokes: Vec<Vec<Cmd>> = (0..12)
        .map(|i| {
            let a = i as f64 * PI / 6.0 + PI / 12.0;
            g::polyline(&[g::polar([x, y], a, r * 0.4), g::polar([x, y], a, r * 0.82)], false)
        })
        .collect();
    pt.stroke_many(&spokes, paint::GOLD, 0.2f64.max(r * 0.018), 0.85, NONE);
    let (mut signs, mut st) = (Vec::new(), Vec::new());
    for i in 0..12i64 {
        let kf = i as f64;
        let p = g::polar([x, y], kf * PI / 6.0 + PI / 2.0, r * 0.61);
        let m = (i + rng.int(0, 3)).rem_euclid(4);
        match m {
            0 => signs.push(k::dot(p[0], p[1], r * 0.06)),
            1 => {
                for dd in [-1.0, 1.0] {
                    let q = g::polar(p, kf * PI / 6.0, dd * r * 0.06);
                    signs.push(k::dot(q[0], q[1], r * 0.035));
                }
            }
            2 => signs.push(g::smooth(&k::crescent(p[0], p[1], r * 0.075, 0.6, kf * PI / 6.0), true)),
            _ => st.push(g::polyline(&k::star(p[0], p[1], r * 0.09, r * 0.035, 5, PI / 2.0), true)),
        }
    }
    pt.fill_many(&signs, paint::GOLD, 0.9, NONE);
    pt.fill_many(&st, paint::COPPER, 0.9, NONE);
    let (lit, shade) = k::rays(x, y, r * 0.17, r * 0.36, 12, RayOpts::default());
    k::gilt(pt, &lit, &shade, 0.95);
    pt.fill(&k::dot(x, y, r * 0.17), paint::GOLD, 0.95, NONE);
    pt.fill_many(&k::ring(x, y, r * 0.91, 36, 0.25f64.max(r * 0.022), PI / 2.0), paint::GOLD, 0.85, NONE);
}

fn sun_crest(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (x, y, _, _, s) = k::measure(b);
    let r = s * 0.5;
    let rot = rng.range(0.0, PI / 16.0);
    let (lit, shade) = k::rays(x, y, r * 0.36, r, 32, RayOpts { flames: true, rot, ..Default::default() });
    k::gilt(pt, &lit, &shade, 0.95);
    pt.fill(&k::dot(x, y, r * 0.36), paint::GOLD, 0.95, NONE);
    pt.fill(&k::dot(x - r * 0.06, y + r * 0.06, r * 0.24), paint::CREAM, 0.45, NONE);
    pt.stroke_many(&[k::dot(x, y, r * 0.36), k::dot(x, y, r * 0.3)], paint::COPPER, 0.35, 0.9, NONE);
    rosette(pt, x, y, r * 0.2, 8, PI / 2.0);
}

fn compass_rose(pt: &mut Paint, b: [f64; 4]) {
    let (x, y, _, _, s) = k::measure(b);
    let r = s * 0.46;
    pt.stroke_many(&[k::dot(x, y, r * 0.78), k::dot(x, y, r * 0.7)], paint::GOLD, 0.4, 0.9, NONE);
    let ticks: Vec<Vec<Cmd>> = (0..32)
        .map(|i| {
            let a = i as f64 * PI / 16.0;
            g::polyline(&[g::polar([x, y], a, r * 0.7), g::polar([x, y], a, r * if i % 2 == 0 { 0.78 } else { 0.74 })], false)
        })
        .collect();
    pt.stroke_many(&ticks, paint::GOLD, 0.25, 0.85, NONE);
    for layer in [(16, r * 0.62, 0.16, PI / 16.0), (8, r * 0.8, 0.2, PI / 8.0), (4, r, 0.26, 0.0)] {
        let (mut lit, mut shade) = (Vec::new(), Vec::new());
        for i in 0..layer.0 {
            let a = PI / 2.0 + layer.3 + i as f64 * 2.0 * PI / layer.0 as f64;
            let tip = g::polar([x, y], a, layer.1);
            let half = layer.1 * layer.2 * 0.5;
            lit.push(g::polyline(&[[x, y], tip, g::polar([x, y], a + PI / 2.0, half)], true));
            shade.push(g::polyline(&[[x, y], tip, g::polar([x, y], a - PI / 2.0, half)], true));
        }
        k::gilt(pt, &lit, &shade, 0.95);
    }
    pt.fill_many(&k::trefoil(x, y + r * 1.02, r * 0.14, PI / 2.0), paint::GOLD, 0.95, NONE);
    pt.fill(&k::dot(x, y, r * 0.06), paint::CREAM, 1.0, NONE);
    pt.stroke(&k::dot(x, y, r * 0.06), paint::COPPER, 0.25, 0.9, NONE);
}

fn wreath(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (x, y, _, _, s) = k::measure(b);
    let r = s * 0.38;
    let (mut leaves, mut veins) = (Vec::new(), Vec::new());
    for d in [-1.0, 1.0] {
        let stem: Vec<Pt> = (0..=24)
            .map(|i| {
                let a = -PI / 2.0 + d * (0.22 + i as f64 / 24.0 * (PI - 0.62));
                g::polar([x, y], a, r)
            })
            .collect();
        veins.push(g::smooth(&stem, false));
        let mut i = 2;
        while i <= 24 {
            let kf = i as f64;
            let a = -PI / 2.0 + d * (0.22 + kf / 24.0 * (PI - 0.62));
            let p = g::polar([x, y], a, r);
            let along = a + d * PI / 2.0;
            let size = s * 0.075 * (1.0 - kf / 60.0);
            for side in [-1.0, 1.0] {
                let e = along + side * 0.7;
                leaves.push(g::smooth(&k::ellipse(p[0] + e.cos() * size * 0.9, p[1] + e.sin() * size * 0.9, size, size * 0.38, e, 14), true));
            }
            i += 2;
        }
    }
    pt.stroke_many(&veins, paint::COPPER, 0.5, 0.8, NONE);
    pt.fill_many(&leaves, paint::GOLD, 0.9, NONE);
    for d in [-1.0, 1.0] {
        pt.fill(&g::smooth(&k::ellipse(x + d * s * 0.06, y - r - s * 0.01, s * 0.06, s * 0.03, d * 0.4, 16), true), paint::COPPER, 0.85, NONE);
        pt.fill(&k::taper(&[[x, y - r], [x + d * s * 0.05, y - r - s * 0.08], [x + d * s * 0.09, y - r - s * 0.13]], s * 0.025, 0.4), paint::COPPER, 0.85, NONE);
    }
    let rot = PI / 2.0 + rng.jit(0.05);
    let (lit, shade) = k::rays(x, y, s * 0.03, s * 0.2, 8, RayOpts { short: 0.45, width: 0.3, rot, flames: false });
    k::gilt(pt, &lit, &shade, 0.95);
}

fn fleuron_rule(pt: &mut Paint, b: [f64; 4]) {
    let (cx, cy, w, h, _) = k::measure(b);
    let s = h * 0.38;
    let leaves = [g::smooth(&k::lozenge(cx - s * 0.62, cy, s, PI), true), g::smooth(&k::lozenge(cx + s * 0.62, cy, s, 0.0), true)];
    let rules: Vec<Vec<Cmd>> = [-1.0, 1.0].iter().map(|d: &f64| k::taper(&[[cx + d * s * 1.45, cy], [cx + d * w * 0.47, cy]], h * 0.05, 0.1)).collect();
    pt.fill_many(&rules, paint::GOLD, 0.9, NONE);
    pt.fill_many(&leaves, paint::COPPER, 0.6, NONE);
    pt.fill(&g::smooth(&k::lozenge(cx - s * 0.6, cy + s * 0.04, s * 0.92, PI), true), paint::GOLD, 0.95, NONE);
    pt.fill(&g::smooth(&k::lozenge(cx + s * 0.6, cy + s * 0.04, s * 0.92, 0.0), true), paint::GOLD, 0.95, NONE);
    pt.fill(&k::dot(cx, cy, s * 0.14), paint::COPPER, 0.95, NONE);
    let mut dots = Vec::new();
    for d in [-1.0, 1.0] {
        for i in 0..3 {
            let kf = i as f64;
            dots.push(k::dot(cx + d * (w * 0.485 - kf * h * 0.12), cy, h * (0.045 - kf * 0.01)));
        }
    }
    pt.fill_many(&dots, paint::GOLD, 0.9, NONE);
}

fn headpiece(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (_, cy, _, h, _) = k::measure(b);
    let bh = h * 0.5;
    let (x0, x1) = (b[0] + h * 0.35, b[2] - h * 0.35);
    let n = 5f64.max(((x1 - x0) / (bh * 0.9)).floor()) as i64;
    let (mut up, mut down) = (Vec::new(), Vec::new());
    for i in 0..n {
        let x = x0 + (i as f64 + 0.5) * (x1 - x0) / n as f64;
        let even = i % 2 == 0;
        let path = g::smooth(&k::lozenge(x, cy - (if even { 0.32 } else { -0.32 }) * bh, bh * 0.58, if even { PI / 2.0 } else { -PI / 2.0 }), true);
        if even {
            up.push(path);
        } else {
            down.push(path);
        }
    }
    pt.fill_many(&up, paint::GOLD, 0.92, NONE);
    pt.fill_many(&down, paint::COPPER, 0.8, NONE);
    let rules: Vec<Vec<Cmd>> = [cy + bh * 0.62, cy - bh * 0.62].iter().map(|y| g::polyline(&[[x0, *y], [x1, *y]], false)).collect();
    pt.stroke_many(&rules, paint::GOLD, 0.45, 0.95, NONE);
    for x in [b[0] + h * 0.3, b[2] - h * 0.3] {
        let rot = rng.range(0.0, PI / 8.0);
        rosette(pt, x, cy, h * 0.3, 8, rot);
    }
}

fn cul_de_lampe(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, _, w, h, _) = k::measure(b);
    let rows = [7, 5, 3, 1];
    let rh = h * 0.18;
    let s = (w / 8.0).min(rh) * 0.95;
    let (mut gold, mut copper) = (Vec::new(), Vec::new());
    for (ri, n) in rows.iter().enumerate() {
        let i = ri as i64 + 1;
        let y = b[3] - rh * (i as f64 - 0.5);
        for kk in 0..*n {
            let x = cx + (kk as f64 - (*n as f64 - 1.0) / 2.0) * s * 1.05;
            if (kk + i) % 2 == 0 {
                gold.extend(k::trefoil(x, y, s * 0.82, -PI / 2.0));
            } else {
                copper.push(g::smooth(&k::leaf(x, y, s * 0.6, -PI / 2.0, 1.0), true));
            }
        }
    }
    pt.fill_many(&gold, paint::GOLD, 0.92, NONE);
    pt.fill_many(&copper, paint::COPPER, 0.85, NONE);
    let tip = b[3] - rh * rows.len() as f64;
    pt.fill(&g::smooth(&k::leaf(cx, tip - h * 0.08, h * 0.12, -PI / 2.0, 1.0), true), paint::GOLD, 0.95, NONE);
    pt.fill(&k::dot(cx, b[1] + h * 0.04, h * 0.025), paint::GOLD, 0.9, NONE);
    pt.stroke(&g::polyline(&[[cx, tip - h * 0.2], [cx, b[1] + h * 0.07]], false), paint::GOLD, 0.3, 0.8, NONE);
    if rng.chance(0.5) {
        pt.stroke(&g::polyline(&[[cx - s * 4.0, b[3] + h * 0.01], [cx + s * 4.0, b[3] + h * 0.01]], false), paint::GOLD, 0.35, 0.85, NONE);
    }
}

fn hedera(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, _) = k::measure(b);
    let s = (w * 0.3).min(h * 0.5);
    let cy = cy + s * 0.4;
    let leaf = k::leaf(cx, cy - s * 0.2, s, -PI / 2.0, 1.0);
    let shadow: Vec<Pt> = leaf.iter().map(|q| [q[0] + s * 0.03, q[1] - s * 0.03]).collect();
    pt.fill(&g::smooth(&shadow, true), paint::COPPER, 0.7, NONE);
    pt.fill(&g::smooth(&leaf, true), paint::GOLD, 0.95, NONE);
    let d = if rng.chance(0.5) { 1.0 } else { -1.0 };
    scroll(pt, [cx, cy - s * 0.2 + s * 0.15], PI / 2.0 + d * 0.3, s * 1.1, -d, s * 0.08);
    pt.stroke(&g::polyline(&[[cx, cy - s * 0.1], [cx, cy - s * 0.85]], false), paint::COPPER, 0.35, 0.8, NONE);
    pt.fill_many(&[k::dot(cx - s * 1.1, cy - s * 0.4, s * 0.06), k::dot(cx + s * 1.1, cy - s * 0.4, s * 0.06)], paint::GOLD, 0.9, NONE);
}

fn garland(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (_, cy, w, h, _) = k::measure(b);
    let (x0, x1) = (b[0] + w * 0.12, b[2] - w * 0.12);
    let (top, sag) = (cy + h * 0.25, h * 0.45);
    let cord_pts: Vec<Pt> = (0..=24)
        .map(|i| {
            let t = i as f64 / 24.0;
            [x0 + (x1 - x0) * t, top - sag * (PI * t).sin()]
        })
        .collect();
    pt.stroke(&g::smooth(&cord_pts, false), paint::COPPER, 0.4, 0.8, NONE);
    let (mut gold, mut copper) = (Vec::new(), Vec::new());
    for i in 1..=9i64 {
        let t = i as f64 / 10.0;
        let (x, y) = (x0 + (x1 - x0) * t, top - sag * (PI * t).sin());
        let along = (-sag * PI * (PI * t).cos()).atan2(x1 - x0);
        let size = h * 0.17 * (0.65 + 0.45 * (PI * t).sin());
        for sd in [-1.0, 1.0] {
            let leaf = g::smooth(&k::leaf(x, y, size, along + sd * 1.05, 0.55), true);
            if (i + if sd > 0.0 { 1 } else { 0 }) % 2 == 0 {
                gold.push(leaf);
            } else {
                copper.push(leaf);
            }
        }
    }
    pt.fill_many(&copper, paint::COPPER, 0.8, NONE);
    pt.fill_many(&gold, paint::GOLD, 0.92, NONE);
    for x in [x0, x1] {
        let rot = rng.range(0.0, PI / 8.0);
        rosette(pt, x, top, h * 0.13, 8, rot);
        pt.fill(&g::smooth(&k::leaf(x, top - h * 0.3, h * 0.12, -PI / 2.0, 1.0), true), paint::COPPER, 0.85, NONE);
    }
}

fn bracket(pt: &mut Paint, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let w = s * 0.05;
        pt.fill_many(
            &[k::taper(&[[x0 + s * 0.08, y0 + s * 0.08], [x0 + s * 0.85, y0 + s * 0.08]], w, 0.2), k::taper(&[[x0 + s * 0.08, y0 + s * 0.08], [x0 + s * 0.08, y0 + s * 0.85]], w, 0.2)],
            paint::GOLD,
            0.92,
            NONE,
        );
        scroll(pt, [x0 + s * 0.6, y0 + s * 0.08], PI / 2.0, s * 0.32, 1.0, w * 0.8);
        scroll(pt, [x0 + s * 0.08, y0 + s * 0.6], 0.0, s * 0.32, -1.0, w * 0.8);
        rosette(pt, x0 + s * 0.08, y0 + s * 0.08, s * 0.08, 8, PI / 2.0);
        pt.fill_many(&[k::dot(x0 + s * 0.93, y0 + s * 0.08, s * 0.025), k::dot(x0 + s * 0.08, y0 + s * 0.93, s * 0.025)], paint::GOLD, 0.9, NONE);
    });
}

fn corner_leaf(pt: &mut Paint, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let c = [x0 + s * 0.42, y0 + s * 0.42];
        let leaf = k::leaf(c[0], c[1], s * 0.36, -3.0 * PI / 4.0, 1.0);
        pt.fill(&g::smooth(&leaf, true), paint::GOLD, 0.95, NONE);
        scroll(pt, g::polar(c, PI / 4.0, s * 0.08), PI / 4.0 + 0.9, s * 0.42, 1.0, s * 0.04);
        scroll(pt, g::polar(c, PI / 4.0, s * 0.08), PI / 4.0 - 0.9, s * 0.42, -1.0, s * 0.04);
        pt.fill(&k::dot(x0 + s * 0.06, y0 + s * 0.06, s * 0.035), paint::GOLD, 0.9, NONE);
    });
}

fn ink(pt: &mut Paint, polys: &[Vec<Pt>], dots: &[[f64; 3]], gilt: bool, shade: f64) {
    let mut paths: Vec<Vec<Cmd>> = polys.iter().map(|p| g::polyline(p, true)).collect();
    for d in dots {
        paths.push(k::dot(d[0], d[1], d[2]));
    }
    if gilt {
        gild::fill_o(pt, &paths, FillOpts { bevel: Some(0.1f64.max(shade * 0.22)), key: Some(0.06), shadow: Some(0.0), ..Default::default() });
    } else {
        pt.fill_many(&paths, paint::INK, 0.9, MULT);
    }
}

fn copperplate_half(rng: &mut Rng) -> Vec<Pt> {
    let l1 = rng.range(0.3, 0.5);
    let t1 = rng.range(0.02, 0.06);
    let l2 = rng.range(0.3, 0.5);
    let t2 = -rng.range(0.02, 0.06);
    let (lead, a) = pen::path([0.08, 0.0], 0.0, &[(l1, t1, Shape::Bell), (l2, t2, Shape::Bell)]);
    let side = if rng.chance(0.7) { 1.0 } else { -1.0 };
    let rx = rng.range(0.5, 0.75);
    let ry = rng.range(0.26, 0.38);
    let drift = rng.range(0.45, 0.8);
    let lp = pen::looped(lead[lead.len() - 1], a, rx, ry, drift, side);
    let mut segs = Vec::new();
    let waves = rng.int(1, 2);
    let len = rng.range(0.8, 1.2) / waves as f64;
    for i in 1..=waves {
        let amp = rng.range(0.04, 0.07) * if i % 2 == 1 { -side } else { side };
        segs.push((len, amp, Shape::Bell));
        segs.push((len, -amp, Shape::Bell));
    }
    let curl = if rng.chance(0.6) { 1.0 } else { -1.0 };
    let cl = rng.range(0.25, 0.4);
    segs.push((cl, curl * 0.08, Shape::C));
    let il = rng.range(0.8, 1.1);
    let it = curl * rng.range(1.0, 1.4);
    segs.push((il, it, Shape::In));
    let (tail, _) = pen::path(lp[lp.len() - 1], a, &segs);
    let mut pts: Vec<Pt> = Vec::new();
    for part in [&lead, &lp, &tail] {
        let from = if pts.is_empty() { 0 } else { 1 };
        pts.extend_from_slice(&part[from..]);
    }
    pts
}

fn copperplate(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let half = copperplate_half(rng);
    let form = rng.weighted(&[("mirror", 3.0), ("turn", 2.0)]).unwrap();
    let mut strokes: Vec<(Vec<Pt>, bool)> = Vec::new();
    let mut extra: Vec<Pt> = Vec::new();
    if form == "mirror" {
        strokes.push((half.clone(), false));
        strokes.push((pen::mirror(&half, 0.0, -1.0), false));
        strokes.push((vec![[-0.06, 0.0], [0.0, 0.12], [0.06, 0.0], [0.0, -0.12]], true));
        if rng.chance(0.5) {
            extra = vec![[0.0, 0.24], [0.0, -0.24]];
        }
    } else {
        let left = pen::rotate(&half, 0.0, 0.0);
        let mut whole: Vec<Pt> = left[1..].iter().rev().copied().collect();
        whole.extend_from_slice(&half);
        let a = g::angle(whole[0], whole[whole.len() - 1]);
        strokes.push((g::transform(&whole, 0.0, 0.0, -a, 1.0, 1.0), false));
    }
    let refs: Vec<&[Pt]> = strokes.iter().map(|s| &s.0[..]).collect();
    let ext = pen::extent(&refs);
    let fit = pen::fit(ext, b, (b[2] - b[0]).min(b[3] - b[1]) * 0.08);
    let shade = g::clamp(fit.2 * 0.03, 0.45, 1.8);
    let polys: Vec<Vec<Pt>> = strokes
        .iter()
        .map(|(st, solid)| {
            let f = pen::apply_fit(fit, st);
            if *solid {
                f
            } else {
                pen::pointed(&f, Pointed { hair: 0.16f64.max(shade * 0.12), shade, t0: 0.02, t1: 0.08, ..Default::default() })
            }
        })
        .collect();
    let dots: Vec<[f64; 3]> = pen::apply_fit(fit, &extra).iter().map(|p| [p[0], p[1], shade * 0.9]).collect();
    let gilt = rng.chance(0.5);
    ink(pt, &polys, &dots, gilt, shade);
}

fn swash(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, _) = k::measure(b);
    let (u, v) = (w / 2.0, h / 2.0);
    let lift = rng.range(0.6, 1.0);
    let wave = rng.range(0.3, 0.5);
    let curl = rng.range(0.85, 1.15);
    let mut polys = Vec::new();
    for d in [-1.0, 1.0] {
        let base = pen::mirror(&[[cx + u * 0.04, cy], [cx + u * 0.16, cy + v * 0.5 * lift], [cx + u * 0.38, cy - v * wave], [cx + u * 0.62, cy + v * wave * 0.7 * lift], [cx + u * 0.8, cy]], cx, d);
        let mut pts = g::spline_points(&base, 10);
        let tail = k::scroll(pts[pts.len() - 1], if d > 0.0 { -0.4 } else { PI + 0.4 }, h * 0.95 * curl, d, 1.15);
        pts.extend_from_slice(&tail[1..]);
        polys.push(pen::pointed(&pts, Pointed { hair: 0.2, shade: h * 0.035, t0: 0.03, t1: 0.1, ..Default::default() }));
    }
    let gilt = rng.chance(0.4);
    polys.push(vec![[cx, cy + v * 0.3], [cx + v * 0.18, cy], [cx, cy - v * 0.3], [cx - v * 0.18, cy]]);
    ink(pt, &polys, &[[cx - u * 0.94, cy, h * 0.04], [cx + u * 0.94, cy, h * 0.04]], gilt, h * 0.035);
}

fn lens(x0: f64, x1: f64, y: f64, t: f64, pw: f64) -> Vec<Pt> {
    let top: Vec<Pt> = (0..=40)
        .map(|i| {
            let f = i as f64 / 40.0;
            [x0 + (x1 - x0) * f, y + t / 2.0 * (PI * f).sin().powf(pw) + 0.06]
        })
        .collect();
    let mut out = top.clone();
    for i in (1..top.len() - 1).rev() {
        out.push([top[i][0], 2.0 * y - top[i][1]]);
    }
    out
}

fn swelled_rule(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, _) = k::measure(b);
    let form = rng.weighted(&[("plain", 2.0), ("broken", 3.0), ("oxford", 2.0), ("triple", 2.0)]).unwrap();
    let t = h * rng.range(0.11, 0.16);
    let (x0, x1) = (b[0] + w * 0.04, b[2] - w * 0.04);
    let mut paths = Vec::new();
    let mut dots: Vec<Pt> = Vec::new();
    match form {
        "plain" => paths.push(g::polyline(&lens(x0, x1, cy, t, 1.4), true)),
        "broken" => {
            let s = h * 0.26;
            paths.push(g::polyline(&lens(x0, cx - s * 1.5, cy, t, 1.4), true));
            paths.push(g::polyline(&lens(cx + s * 1.5, x1, cy, t, 1.4), true));
            paths.push(g::smooth(&k::lozenge(cx - s * 0.7, cy, s * 1.4, 0.0), true));
            dots = vec![[cx - s * 1.15, cy], [cx + s * 1.15, cy]];
        }
        "oxford" => {
            paths.push(g::polyline(&lens(x0, x1, cy + t * 0.55, t, 1.4), true));
            paths.push(g::polyline(&lens(x0 + w * 0.06, x1 - w * 0.06, cy - t * 0.75, t * 0.22, 0.3), true));
        }
        _ => {
            let gp = w * 0.035;
            let side = (x1 - x0) * 0.17;
            paths.push(g::polyline(&lens(x0 + side + gp, x1 - side - gp, cy, t, 1.4), true));
            paths.push(g::polyline(&lens(x0, x0 + side, cy, t * 0.55, 1.4), true));
            paths.push(g::polyline(&lens(x1 - side, x1, cy, t * 0.55, 1.4), true));
            dots = vec![[x0 + side + gp / 2.0, cy], [x1 - side - gp / 2.0, cy]];
        }
    }
    gild::fill_o(pt, &paths, FillOpts { bevel: Some(t * 0.12), key: Some(0.15), shadow: Some(t * 0.12), ..Default::default() });
    if !dots.is_empty() {
        gild::dots(pt, &dots, t * 0.3);
    }
}

fn fleuronnee(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let (cx, cy, w, h, _) = k::measure(b);
    let (mut line, mut accent) = (paint::INK, paint::COPPER);
    if rng.chance(0.35) {
        line = paint::COPPER;
        accent = paint::INK;
    }
    let hw = g::clamp(h * 0.006, 0.16, 0.3);
    let mut hairs: Vec<Vec<Cmd>> = Vec::new();
    let (mut beads, mut rings): (Vec<[f64; 3]>, Vec<[f64; 3]>) = (Vec::new(), Vec::new());
    let hair = |hairs: &mut Vec<Vec<Cmd>>, pts: &[Pt], f: f64| hairs.push(g::polyline(&pen::hairline(pts, hw, f), true));
    let reach = w * 0.47;
    let bud = h * rng.range(0.22, 0.28);
    let n = rng.int(4, 6);
    let turns = rng.range(0.9, 1.25);
    let wave = rng.range(0.0, 0.05);
    let size = rng.range(0.4, 0.5);
    let gap = h * rng.range(0.05, 0.07);
    for d in [-1.0, 1.0] {
        let start = [cx + d * bud * 1.05, cy];
        let mut spine = g::spline_points(&[start, [cx + d * reach * 0.35, cy + h * wave], [cx + d * reach * 0.7, cy - h * wave], [cx + d * reach * 0.84, cy]], 10);
        let curl = pen::sprig(spine[spine.len() - 1], if d > 0.0 { 0.0 } else { PI }, h * 0.55, d, turns);
        spine.extend_from_slice(&curl[1..]);
        hair(&mut hairs, &spine, 0.6);
        let ce = pen::echo(&curl, -d * gap);
        let cut = &ce[..(ce.len() as f64 * 0.7).floor() as usize];
        hair(&mut hairs, cut, 0.3);
        let tip = curl[curl.len() - 1];
        rings.push([tip[0], tip[1], h * 0.045]);
        for kk in 1..=n {
            let f = 0.06 + 0.68 * (kk as f64 - 0.5) / n as f64;
            let (p, along) = g::at(&spine, f);
            let side = if kk % 2 == 1 { 1.0 } else { -1.0 };
            let l = h * size * rng.range(0.85, 1.1);
            let tn = rng.range(0.95, 1.25);
            let sp = pen::sprig(p, along + side * d * 0.85, l, -side * d, tn);
            hair(&mut hairs, &sp, 0.5);
            let e = pen::echo(&sp, side * d * gap);
            let m55 = (e.len() as f64 * 0.55).floor() as usize;
            let part: Vec<Pt> = if m55 >= 2 { e[1..m55].to_vec() } else { Vec::new() };
            if part.len() > 2 {
                hair(&mut hairs, &part, 0.2);
            }
            let q = sp[sp.len() - 1];
            rings.push([q[0], q[1], h * 0.035]);
            let m = g::at(&spine, f + 0.34 / n as f64).0;
            let off = -side * h * 0.17;
            for j in [-1.0, 0.0, 1.0] {
                beads.push([m[0] + j * h * 0.045, m[1] + off + if j == 0.0 { -side * h * 0.045 } else { 0.0 }, h * 0.022]);
            }
        }
        for j in [-1.0, 1.0] {
            let f = pen::sprig([cx + d * bud * 0.9, cy + j * bud * 0.3], (if d > 0.0 { 0.0 } else { PI }) + j * d * 0.7, h * 0.3, -j * d, 0.7);
            hair(&mut hairs, &f, 0.4);
        }
    }
    let core = k::lozenge(cx - bud * 0.6, cy, bud * 1.2, 0.0);
    for sc in [1.4, 1.8] {
        let mut e = g::toward(&core, cx, cy, sc);
        e.push(e[0]);
        hair(&mut hairs, &e, 1.0);
    }
    pt.fill_many(&hairs, line, 0.92, MULT);
    let circles: Vec<Vec<Cmd>> = rings.iter().map(|r| g::polyline(&pen::hairline(&k::ellipse(r[0], r[1], r[2], r[2], 0.0, 20), hw, 1.0), true)).collect();
    pt.fill_many(&circles, line, 0.92, MULT);
    let mut dots: Vec<Vec<Cmd>> = beads.iter().map(|q| k::dot(q[0], q[1], q[2])).collect();
    dots.extend(rings.iter().map(|r| k::dot(r[0], r[1], r[2] * 0.45)));
    pt.fill_many(&dots, accent, 0.92, MULT);
    pt.fill(&g::smooth(&core, true), accent, 0.9, MULT);
    pt.fill(&k::dot(cx, cy, bud * 0.13), paint::CREAM, 0.9, NONE);
}

#[derive(Clone, Copy)]
struct LeafO {
    curl: f64,
    lobes: i64,
    wide: f64,
}

fn acanthus_leaf(base: Pt, ang: f64, len: f64, o: LeafO) -> (Vec<Pt>, Vec<Vec<Pt>>, Vec<Vec<Pt>>) {
    let rib = g::resample(&pen::path(base, ang, &[(len * 0.5, o.curl * 0.2, Shape::C), (len * 0.5, o.curl * 0.8, Shape::In)]).0, 160);
    let (n, wd) = (o.lobes as f64, len * o.wide);
    let env = |t: f64| wd * (PI * (0.06 + 0.94 * t.powf(0.8))).sin().powf(0.85) * (1.0 - 0.3 * t);
    let lobe = |t: f64, phase: f64| {
        let u = fmod(t * n + phase, 1.0);
        let swell = 0.18 + 0.82 * (PI * u.powf(0.8)).sin().powf(0.55);
        let teeth = 0.72 + 0.28 * fmod(u * 3.0, 1.0).powf(2.2);
        swell * teeth
    };
    let m = rib.len();
    let (mut left, mut right) = (Vec::with_capacity(m), Vec::with_capacity(m));
    for i in 0..m {
        let t = i as f64 / (m as f64 - 1.0);
        let a = g::angle(rib[i.saturating_sub(1)], rib[(i + 1).min(m - 1)]);
        let stalk = g::smoothstep(0.02, 0.14, t);
        let wl = env(t) * (0.25 + 0.75 * stalk * lobe(t, 0.0));
        let wr = env(t) * (0.25 + 0.75 * stalk * lobe(t, 0.5));
        left.push(g::polar(rib[i], a + PI / 2.0, wl));
        right.push(g::polar(rib[i], a - PI / 2.0, wr));
    }
    let mut out = left.clone();
    out.extend(right[1..m - 1].iter().rev());
    let mut veins: Vec<Vec<Pt>> = vec![rib[..(m as f64 * 0.88).floor() as usize].to_vec()];
    let mut eyes = Vec::new();
    let mf = m as f64;
    for (side, pts) in [(0.0, &left), (0.5, &right)] {
        for kk in 0..o.lobes {
            let t1 = (kk as f64 + 0.92 - side) / n;
            if t1 > 0.12 && t1 < 0.9 {
                let i1 = (t1 * (mf - 1.0)).floor() as usize + 1;
                let i0 = 1.max(i1 as i64 - (mf * 0.12).floor() as i64) as usize;
                veins.push(vec![rib[i0 - 1], g::mix(rib[i1 - 1], pts[i1 - 1], 0.7)]);
            }
            let tn = (kk as f64 + 1.0 - side) / n;
            if tn > 0.15 && tn < 0.85 {
                let j = (tn * (mf - 1.0)).floor() as usize + 1;
                let q = g::mix(rib[j - 1], pts[j - 1], 0.55);
                let r = env(tn) * 0.12;
                let ang2 = g::angle(rib[(j - 1).saturating_sub(1)], rib[j.min(m - 1)]);
                eyes.push(k::ellipse(q[0], q[1], r * 1.6, r, ang2, 12));
            }
        }
    }
    (out, veins, eyes)
}

#[derive(Default, Clone)]
struct Bunch {
    leaves: Vec<Vec<Pt>>,
    veins: Vec<Vec<Pt>>,
    eyes: Vec<Vec<Pt>>,
}

impl Bunch {
    fn gather(&mut self, l: (Vec<Pt>, Vec<Vec<Pt>>, Vec<Vec<Pt>>)) {
        self.leaves.push(l.0);
        self.veins.extend(l.1);
        self.eyes.extend(l.2);
    }

    fn remap(&self, f: &dyn Fn(&[Pt]) -> Vec<Pt>) -> Bunch {
        Bunch {
            leaves: self.leaves.iter().map(|x| f(x)).collect(),
            veins: self.veins.iter().map(|x| f(x)).collect(),
            eyes: self.eyes.iter().map(|x| f(x)).collect(),
        }
    }
}

fn paint_leaves(pt: &mut Paint, b: &Bunch, size: f64) {
    let paths: Vec<Vec<Cmd>> = b.leaves.iter().map(|l| g::smooth(l, true)).collect();
    gild::fill_o(pt, &paths, FillOpts { bevel: Some(g::clamp(size * 0.012, 0.12, 0.45)), key: Some(0.14), shadow: Some(size * 0.01), ..Default::default() });
    let vp: Vec<Vec<Cmd>> = b.veins.iter().map(|v| g::smooth(v, false)).collect();
    pt.stroke_many(&vp, UMBER, g::clamp(size * 0.006, 0.12, 0.3), 0.55, NONE);
    let ep: Vec<Vec<Cmd>> = b.eyes.iter().map(|e| g::smooth(e, true)).collect();
    if !ep.is_empty() {
        pt.fill_many(&ep, UMBER, 0.6, NONE);
    }
}

fn stems(pt: &mut Paint, list: &[(Vec<Pt>, Option<f64>)], w: f64) {
    let paths: Vec<Vec<Cmd>> = list.iter().map(|(p, sw)| k::taper(p, sw.unwrap_or(w), 0.3)).collect();
    gild::fill_o(pt, &paths, FillOpts { bevel: Some(w * 0.18), key: Some(0.12), shadow: Some(w * 0.2), ..Default::default() });
}

struct RinceauHalf {
    stem: Vec<Pt>,
    bunch: Bunch,
    tendril: Vec<Pt>,
    berries: Vec<Pt>,
}

fn rinceau_half(rng: &mut Rng) -> RinceauHalf {
    let up = if rng.chance(0.5) { 1.0 } else { -1.0 };
    let a0 = up * rng.range(0.15, 0.35);
    let swing = rng.range(0.08, 0.14);
    let s1 = rng.range(1.2, 1.5);
    let s2 = rng.range(1.0, 1.3);
    let s3 = rng.range(0.3, 0.45);
    let s4 = rng.range(1.5, 1.8);
    let s4t = -up * rng.range(1.3, 1.6);
    let stem = pen::path([0.18, -0.12], a0, &[(s1, -up * swing, Shape::Bell), (s2, up * swing * 0.6, Shape::Bell), (s3, -up * 0.05, Shape::C), (s4, s4t, Shape::In)]).0;
    let mut bn = Bunch::default();
    let leaf = |rng: &mut Rng, bn: &mut Bunch, at: f64, turn: f64, len: f64, curl: f64| {
        let (p, a) = g::at(&stem, at);
        let lobes = rng.int(3, 4);
        let wide = rng.range(0.2, 0.25);
        bn.gather(acanthus_leaf(p, a + turn, len, LeafO { curl, lobes, wide }));
    };
    let l1 = rng.range(0.7, 0.85);
    let c1 = -up * rng.range(0.2, 0.3);
    leaf(rng, &mut bn, 0.02, up * 0.4, l1, c1);
    let at2 = rng.range(0.58, 0.64);
    let l2 = rng.range(0.6, 0.75);
    let c2 = up * rng.range(0.2, 0.3);
    leaf(rng, &mut bn, at2, -up * 0.5, l2, c2);
    let (p, a) = g::at(&stem, rng.range(0.3, 0.4));
    let tl = rng.range(0.45, 0.6);
    let tt = rng.range(0.9, 1.2);
    let tend = pen::sprig(p, a + up * 0.9, tl, -up, tt);
    let tip = tend[tend.len() - 1];
    let berries: Vec<Pt> = (0..3).map(|i| g::polar(tip, a + up * (0.6 + i as f64 * 0.7), 0.07)).collect();
    RinceauHalf { stem, bunch: bn, tendril: tend, berries }
}

fn rinceau(pt: &mut Paint, rng: &mut Rng, b: [f64; 4]) {
    let half = rinceau_half(rng);
    let mut calyx = Bunch::default();
    for d in [-1.0, 1.0] {
        calyx.gather(acanthus_leaf([d * 0.04, -0.3], PI / 2.0 - d, 0.6, LeafO { curl: -d * 0.25, lobes: 3, wide: 0.23 }));
    }
    calyx.gather(acanthus_leaf([0.0, -0.3], PI / 2.0, 0.5, LeafO { curl: 0.0, lobes: 3, wide: 0.2 }));
    let mirrored_stem = pen::mirror(&half.stem, 0.0, -1.0);
    let mirrored: Vec<Vec<Pt>> = half.bunch.leaves.iter().map(|l| pen::mirror(l, 0.0, -1.0)).collect();
    let mut lists: Vec<&[Pt]> = vec![&half.stem, &mirrored_stem];
    for (l, m) in half.bunch.leaves.iter().zip(&mirrored) {
        lists.push(l);
        lists.push(m);
    }
    for l in &calyx.leaves {
        lists.push(l);
    }
    let fit = pen::fit(pen::extent(&lists), b, (b[2] - b[0]).min(b[3] - b[1]) * 0.06);
    let s = fit.2;
    let right = |l: &[Pt]| pen::apply_fit(fit, l);
    let left = |l: &[Pt]| pen::apply_fit(fit, &pen::mirror(l, 0.0, -1.0));
    stems(pt, &[(right(&half.stem), None), (left(&half.stem), None), (right(&half.tendril), Some(s * 0.03)), (left(&half.tendril), Some(s * 0.03))], s * 0.07);
    let mut all = Bunch::default();
    for bn in [half.bunch.remap(&right), half.bunch.remap(&left), calyx.remap(&right)] {
        all.leaves.extend(bn.leaves);
        all.veins.extend(bn.veins);
        all.eyes.extend(bn.eyes);
    }
    paint_leaves(pt, &all, s);
    let mut dots = right(&half.berries);
    dots.extend(left(&half.berries));
    gild::dots(pt, &dots, s * 0.04);
    let e = [half.stem[half.stem.len() - 1]];
    for q in [right(&e)[0], left(&e)[0]] {
        let rot = rng.range(0.0, PI / 8.0);
        rosette(pt, q[0], q[1], s * 0.13, 8, rot);
    }
}

fn acanthus_corner(pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let u = |x: f64, y: f64| [x0 + x * s, y0 + y * s];
        let mut list = Vec::new();
        let mut bn = Bunch::default();
        let turn = rng.range(1.2, 1.5);
        for (a, dd) in [(0.0, 1.0), (PI / 2.0, -1.0)] {
            let stem = pen::path(u(0.08, 0.08), a, &[(s * 0.4, dd * 0.02, Shape::C), (s * 0.55, dd * turn, Shape::In)]).0;
            let (p, ta) = g::at(&stem, 0.28);
            bn.gather(acanthus_leaf(p, ta + dd * 0.45, s * 0.34, LeafO { curl: -dd * 0.22, lobes: 3, wide: 0.23 }));
            list.push((stem, None));
        }
        let sign = if rng.chance(0.5) { 1.0 } else { -1.0 };
        let curl = sign * rng.range(0.18, 0.28);
        let len = s * rng.range(0.62, 0.72);
        bn.gather(acanthus_leaf(u(0.1, 0.1), PI / 4.0, len, LeafO { curl, lobes: 4, wide: 0.24 }));
        stems(pt, &list, s * 0.05);
        paint_leaves(pt, &bn, s);
        rosette(pt, x0 + s * 0.08, y0 + s * 0.08, s * 0.09, 8, PI / 8.0);
    });
}

fn arc(c: Pt, r: f64, a0: f64, a1: f64, n: usize) -> Vec<Pt> {
    (0..=n).map(|i| g::polar(c, a0 + (a1 - a0) * i as f64 / n as f64, r)).collect()
}

fn cornerpiece(pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let c = [x0 + s * 0.04, y0 + s * 0.04];
        let r0 = s * rng.range(0.66, 0.74);
        let lobes = rng.int(2, 3);
        let finial = rng.range(0.1, 0.16);
        let edge = |t: f64, scale: f64| {
            let th = t * PI / 2.0;
            let u = fmod(t * (lobes * 2 + 1) as f64, 1.0);
            let mut r = r0 * (0.86 + 0.14 * (PI * u).sin().powf(0.5));
            r += r0 * finial * (1.0 - (t - 0.5).abs() * 9.0).max(0.0).powf(1.5);
            g::polar(c, th, r * scale)
        };
        let mut band: Vec<Pt> = (0..=120).map(|i| edge(i as f64 / 120.0, 1.0)).collect();
        band.extend((0..=120).rev().map(|i| edge(i as f64 / 120.0, 0.86)));
        let mut pieces = vec![g::polyline(&band, true)];
        for (a, big) in [(PI / 8.0, false), (PI / 4.0, true), (3.0 * PI / 8.0, false)] {
            let tip = g::polar(c, a, r0 * 0.46);
            pieces.push(g::smooth(&k::lozenge(tip[0], tip[1], r0 * if big { 0.32 } else { 0.24 }, a), true));
        }
        let mut hub = vec![c];
        for i in 0..=48 {
            let t = i as f64 / 48.0;
            hub.push(g::polar(c, t * PI / 2.0, r0 * (0.2 + 0.03 * (PI * fmod(t * 4.0, 1.0)).sin().powf(0.6))));
        }
        pieces.push(g::polyline(&hub, true));
        gild::fill_o(pt, &pieces, FillOpts { bevel: Some(s * 0.008), key: Some(0.18), shadow: Some(s * 0.006), ..Default::default() });
        let rings: Vec<Vec<Cmd>> = [0.36, 0.42].iter().map(|r: &f64| g::smooth(&arc(c, r0 * r, 0.02, PI / 2.0 - 0.02, 24), false)).collect();
        gild::line(pt, &rings, g::clamp(s * 0.007, 0.18, 0.45));
        let m = lobes * 12;
        let dots: Vec<Pt> = (1..m).map(|i| edge(i as f64 / m as f64, 1.07)).collect();
        gild::dots(pt, &dots, s * 0.009);
        let inner: Vec<Pt> = (1..m).step_by(2).map(|i| edge(i as f64 / m as f64, 0.78)).collect();
        gild::dots(pt, &inner, s * 0.008);
        let mut lines = Vec::new();
        for dd in [0.012, 0.035] {
            lines.push(g::polyline(&[[c[0] + r0 * 0.95, y0 + s * dd], [x0 + s * 0.98, y0 + s * dd]], false));
            lines.push(g::polyline(&[[x0 + s * dd, c[1] + r0 * 0.95], [x0 + s * dd, y0 + s * 0.98]], false));
        }
        gild::line(pt, &lines, g::clamp(s * 0.008, 0.2, 0.5));
        let spokes: Vec<Vec<Cmd>> = (1..=3).map(|i| g::polyline(&[g::polar(c, i as f64 * PI / 8.0, r0 * 0.05), g::polar(c, i as f64 * PI / 8.0, r0 * 0.17)], false)).collect();
        pt.stroke_many(&spokes, UMBER, g::clamp(s * 0.005, 0.12, 0.3), 0.6, NONE);
    });
}

fn pen_corner(pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let (mut line, mut accent) = (paint::INK, paint::COPPER);
        if rng.chance(0.35) {
            line = paint::COPPER;
            accent = paint::INK;
        }
        let hw = g::clamp(s * 0.004, 0.15, 0.28);
        let mut hairs: Vec<Vec<Cmd>> = Vec::new();
        let (mut beads, mut rings): (Vec<[f64; 3]>, Vec<[f64; 3]>) = (Vec::new(), Vec::new());
        let hair = |hairs: &mut Vec<Vec<Cmd>>, pts: &[Pt], f: f64| hairs.push(g::polyline(&pen::hairline(pts, hw, f), true));
        let k0 = s * 0.1;
        let mut rules = Vec::new();
        for e in [[1.0, 0.0], [0.0, 1.0]] {
            let at = |t: f64, off: f64| [x0 + e[0] * t + e[1] * off, y0 + e[1] * t + e[0] * off];
            rules.push(pen::hairline(&[at(k0 * 1.3, k0 * 0.5), at(s * 0.92, k0 * 0.5)], hw * 2.6, 0.2));
            hair(&mut hairs, &[at(k0 * 1.3, k0 * 0.8), at(s * 0.8, k0 * 0.8)], 0.4);
            let n = rng.int(2, 3);
            for kk in 1..=n {
                let t = k0 * 1.5 + (s * 0.7 - k0 * 1.5) * (kk as f64 - 0.5) / n as f64;
                let p = at(t, k0 * 0.8);
                let away = e[0].atan2(e[1]);
                let dir = if e[0] == 1.0 { 1.0 } else { -1.0 };
                let l = s * rng.range(0.28, 0.34);
                let tn = rng.range(0.9, 1.2);
                let sp = pen::sprig(p, away + dir * 0.6, l, -dir, tn);
                hair(&mut hairs, &sp, 0.5);
                let e2 = pen::echo(&sp, -dir * s * 0.025);
                let m5 = (e2.len() as f64 * 0.5).floor() as usize;
                let part: Vec<Pt> = if m5 >= 2 { e2[1..m5].to_vec() } else { Vec::new() };
                if part.len() > 2 {
                    hair(&mut hairs, &part, 0.2);
                }
                let q = sp[sp.len() - 1];
                rings.push([q[0], q[1], s * 0.018]);
                let m = at(t + s * 0.08, k0 * 0.8 + s * 0.06);
                beads.push([m[0], m[1], s * 0.012]);
            }
            let tipdir = if e[0] == 1.0 { 0.0 } else { PI / 2.0 };
            let curl = pen::sprig(at(s * 0.8, k0 * 0.8), tipdir, s * 0.2, if e[0] == 1.0 { 1.0 } else { -1.0 }, 1.1);
            hair(&mut hairs, &curl, 0.5);
        }
        let core = k::lozenge(x0 + k0 * 0.2, y0 + k0 * 0.2, k0 * 1.1, PI / 4.0);
        let mid = g::centroid(&core);
        for sc in [1.45, 1.9] {
            let mut e = g::toward(&core, mid[0], mid[1], sc);
            e.push(e[0]);
            hair(&mut hairs, &e, 1.0);
        }
        for dd in [-1.0, 1.0] {
            let tn = rng.range(1.0, 1.3);
            let sp = pen::sprig(g::polar(mid, PI / 4.0, k0 * 1.3), PI / 4.0 + dd * 0.35, s * 0.3, -dd, tn);
            hair(&mut hairs, &sp, 0.5);
            let q = sp[sp.len() - 1];
            rings.push([q[0], q[1], s * 0.02]);
        }
        let far = g::polar(mid, PI / 4.0, s * 0.48);
        for j in [-1.0, 0.0, 1.0] {
            let q = g::polar(far, PI / 4.0 + PI / 2.0, j * s * 0.03);
            let off = if j == 0.0 { s * 0.02 } else { 0.0 };
            beads.push([q[0] + off, q[1] + off, s * 0.013]);
        }
        pt.fill(&g::polyline(&rules[0], true), line, 0.9, MULT);
        pt.fill(&g::polyline(&rules[1], true), line, 0.9, MULT);
        pt.fill_many(&hairs, line, 0.92, MULT);
        let (mut circles, mut dots) = (Vec::new(), Vec::new());
        for r in &rings {
            circles.push(g::polyline(&pen::hairline(&k::ellipse(r[0], r[1], r[2], r[2], 0.0, 16), hw, 1.0), true));
            dots.push(k::dot(r[0], r[1], r[2] * 0.45));
        }
        for q in &beads {
            dots.push(k::dot(q[0], q[1], q[2]));
        }
        pt.fill_many(&circles, line, 0.92, MULT);
        pt.fill_many(&dots, accent, 0.92, MULT);
        pt.fill(&g::smooth(&core, true), accent, 0.9, MULT);
    });
}

fn fan(pt: &mut Paint, rng: &mut Rng, b: [f64; 4], d: &DrawOpts) {
    k::corner(pt, b, d.flip, |pt, x0, y0, s| {
        let c = [x0 + s * 0.04, y0 + s * 0.04];
        let n = rng.int(5, 7) * 2;
        let rim = s * 0.8;
        let inner = rim * rng.range(0.8, 0.84);
        let step = PI / 2.0 / n as f64;
        let mut band = Vec::new();
        for i in 0..n {
            let (a0, a1) = (i as f64 * step, (i + 1) as f64 * step);
            for j in 0..=10 {
                let t = j as f64 / 10.0;
                let a = a0 + (a1 - a0) * t;
                band.push(g::polar(c, a, rim * (0.95 + 0.06 * (PI * t).sin().powf(0.7))));
            }
        }
        for i in (0..=40).rev() {
            band.push(g::polar(c, i as f64 / 40.0 * PI / 2.0, inner * 1.04));
        }
        gild::fill_o(pt, &[g::polyline(&band, true)], FillOpts { bevel: Some(s * 0.01), key: Some(0.18), shadow: Some(s * 0.006), ..Default::default() });
        let mut rays = Vec::new();
        let dot_r = inner * 0.78;
        for i in 0..=n * 2 {
            let a = i as f64 * step / 2.0;
            let long = i % 2 == 0;
            let r1 = if long { inner * 1.02 } else { dot_r * 0.94 };
            let half = (if long { 0.5 } else { 0.36 }) * step * s * 0.18;
            let base = g::polar(c, a, s * 0.16);
            let tip = g::polar(c, a, r1);
            let mid = g::polar(c, a, s * 0.16 + (r1 - s * 0.16) * 0.35);
            rays.push(g::smooth_t(&[base, g::polar(mid, a + PI / 2.0, half), tip, g::polar(mid, a - PI / 2.0, half)], true, 0.8));
        }
        gild::fill_o(pt, &rays, FillOpts { bevel: Some(s * 0.004), key: Some(0.1), shadow: Some(0.0), ..Default::default() });
        let dots: Vec<Pt> = (0..n).map(|i| g::polar(c, (i as f64 + 0.5) * step, dot_r)).collect();
        gild::dots(pt, &dots, s * 0.014);
        gild::line(pt, &[g::smooth(&arc(c, inner * 0.97, 0.0, PI / 2.0, 30), false)], g::clamp(s * 0.004, 0.15, 0.3));
        let mut hub = vec![c];
        hub.extend(arc(c, s * 0.17, 0.0, PI / 2.0, 20));
        gild::fill_o(pt, &[g::polyline(&hub, true)], FillOpts { bevel: Some(s * 0.01), key: Some(0.15), shadow: Some(0.0), ..Default::default() });
        pt.stroke(&g::smooth(&arc(c, s * 0.11, 0.05, PI / 2.0 - 0.05, 12), false), UMBER, 0.25, 0.6, NONE);
        pt.fill(&k::dot(c[0], c[1], s * 0.05), paint::COPPER, 0.95, NONE);
    });
}
