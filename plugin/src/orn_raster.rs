//! Raster gilt: an ornament's gold computed pixel by pixel and placed as an image.

use crate::color::Rgb;
use crate::geom::Cmd;
use crate::gilt_look::{self, Look};
use crate::orn_gild;
use crate::paint::{Image, Paint, BP};
use crate::raster::{self, Grid};
use crate::raster_relief::{self, Layers, Shape};
use std::f64::consts::PI;
use std::fmt::Write;

fn lut(stops: &[(f64, Rgb)]) -> Vec<[f64; 3]> {
    (0..256)
        .map(|i| {
            let t = i as f64 / 255.0;
            let mut j = 0;
            while j < stops.len() - 2 && t > stops[j + 1].0 {
                j += 1;
            }
            let (a, b) = (stops[j], stops[j + 1]);
            let u = (t - a.0) / (b.0 - a.0);
            let u = if u < 0.0 {
                0.0
            } else if u > 1.0 {
                1.0
            } else {
                u
            };
            let mut c = [0.0; 3];
            for k in 0..3 {
                c[k] = ((a.1[k] + (b.1[k] - a.1[k]) * u) * 255.0 + 0.5).floor();
            }
            c
        })
        .collect()
}

pub struct Pending {
    shapes: Vec<Shape>,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    shadow: f64,
    calls: i64,
}

fn scale_at(pt: &Paint, x: f64, y: f64) -> f64 {
    if !pt.has_pj() {
        return 1.0;
    }
    let (ox, oy) = pt.pj(x, y);
    let (ax, ay) = pt.pj(x + 1.0, y);
    let (bx, by) = pt.pj(x, y + 1.0);
    ((ax - ox) * (by - oy) - (bx - ox) * (ay - oy)).abs().sqrt()
}

pub struct AddOpts {
    pub bevel: Option<f64>,
    pub shadow: f64,
    pub shade: bool,
    pub width: f64,
    pub whole: bool,
    pub evenodd: bool,
}

pub fn add(pt: &mut Paint, stroke: bool, paths: &[Vec<Cmd>], o: AddOpts) {
    if pt.gilt_pending.is_none() {
        pt.gilt_pending = Some(Pending { shapes: Vec::new(), x0: f64::INFINITY, y0: f64::INFINITY, x1: f64::NEG_INFINITY, y1: f64::NEG_INFINITY, shadow: 0.0, calls: 0 });
    }
    let page: Vec<Vec<Cmd>> = if pt.has_pj() { paths.iter().map(|c| pt.projected(c)).collect() } else { paths.to_vec() };
    let b = orn_gild::bbox(&page);
    if b[0] > b[2] {
        return;
    }
    let k = scale_at(pt, (b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0);
    let half = if stroke { o.width * k / 2.0 } else { 0.0 };
    let p = pt.gilt_pending.as_mut().unwrap();
    p.x0 = p.x0.min(b[0] - half);
    p.y0 = p.y0.min(b[1] - half);
    p.x1 = p.x1.max(b[2] + half);
    p.y1 = p.y1.max(b[3] + half);
    p.shadow = p.shadow.max(o.shadow * k);
    p.calls += 1;
    let call = p.calls;
    let bevel = o.bevel.map(|e| e * k);
    let tone = if o.shade { 1.0 } else { 0.0 };
    if stroke || o.whole {
        p.shapes.push(Shape { stroke, paths: page, evenodd: o.evenodd, half, call, bevel, tone });
    } else {
        for cmds in page {
            p.shapes.push(Shape { stroke, paths: vec![cmds], evenodd: o.evenodd, half, call, bevel, tone });
        }
    }
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = (x - a) / (b - a);
    let t = if t < 0.0 {
        0.0
    } else if t > 1.0 {
        1.0
    } else {
        t
    };
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn studio(l: &Look, lx: f64, ly: f64, rx: f64, ry: f64, rz: f64) -> f64 {
    let hl = (rx * rx + ry * ry).sqrt();
    let a = if hl > 1e-6 { (rx * lx + ry * ly) / hl } else { 0.0 };
    let horizon = l.horizon - l.horizon_dip * a;
    let e = rz - horizon;
    let mut sky = l.sky + l.sky_glow * (1.0 - rz) * (1.0 - rz) + (if a > 0.0 { l.sky_lit } else { l.sky_far }) * a * hl;
    if a > 0.0 {
        let span = l.window_top - l.window_foot;
        let wnd = a * a * if rz > l.window_top { 0.0 } else if rz < l.window_foot { 1.0 } else { (l.window_top - rz) / span };
        sky += l.window * wnd;
    }
    let ground = l.ground + l.ground_fall * (-rz) + l.ground_lit * a;
    if e >= l.horizon_soft {
        return sky;
    }
    if e <= -l.horizon_soft {
        return ground;
    }
    let u = (e + l.horizon_soft) / (l.horizon_soft + l.horizon_soft);
    let u = u * u * (3.0 - 2.0 * u);
    ground + (sky - ground) * u
}

#[inline]
fn mirror(l: &Look, lx: f64, ly: f64, rx: f64, ry: f64) -> f64 {
    let u = rx * lx + ry * ly;
    let mut v = l.ground + l.ground_far * smooth(-0.3, -1.0, u);
    v += (l.sky + l.sky_glow * smooth(0.25, 0.95, u) - v) * smooth(l.horizon - l.horizon_soft, l.horizon + l.horizon_soft, u);
    v += l.band * (-((u - l.horizon - 0.06) / l.band_width).powf(2.0)).exp();
    l.front + (v - l.front) * smooth(0.05, 0.6, (rx * rx + ry * ry).sqrt())
}

fn shade(g: &Grid, r0: &Layers, shadow_px: f64, light: f64, l: &Look) -> (Vec<u8>, Vec<u8>) {
    let (gl, cl) = (lut(l.gold), lut(l.copper));
    let (w, hh) = (g.w, g.h);
    let (lx, ly) = (light.cos(), light.sin());
    let (hlt, cut, hs) = (l.highlight, 1.0 - l.highlight_size, l.highlight_size);
    let (grain, gs, spark_rate, glint) = (l.grain, l.grain_scale, l.sparkle, l.sparkle_glint);
    let shadow_k = l.shadow;
    let le = 0.64;
    let (hx, hy, hz) = (lx * le, ly * le, 1.0 + 0.77);
    let hn = (hx * hx + hy * hy + hz * hz).sqrt();
    let (hx, hy, hz) = (hx / hn, hy / hn, hz / hn);
    let mut a_buf = vec![0.0; g.n];
    let (sx, sy) = ((-lx * shadow_px + 0.5).floor() as i64, (ly * shadow_px + 0.5).floor() as i64);
    if shadow_px > 0.0 {
        for row in 1..=hh {
            let rs = row - sy;
            if rs >= 1 && rs <= hh {
                for col in 1..=w {
                    let cs = col - sx;
                    if cs >= 1 && cs <= w {
                        a_buf[((row - 1) * w + col - 1) as usize] = r0.c[((rs - 1) * w + cs - 1) as usize];
                    }
                }
            }
        }
        raster::blur(g, &mut a_buf, 2);
    }
    let (s, gx, gy) = (g.s, g.x0, g.y1);
    let (ecx, ecy) = (w as f64 / 2.0, hh as f64 / 2.0);
    let eye = l.eye * ((w * w + hh * hh) as f64).sqrt();
    let (bc, bs) = ((light + PI / 2.0).cos(), (light + PI / 2.0).sin());
    let mut rgb = Vec::with_capacity(g.n * 3);
    let mut alpha = Vec::with_capacity(g.n);
    let um = l.umber;
    for row in 1..=hh {
        let base = (row - 1) * w;
        let y = gy - (row as f64 - 0.5) * s;
        for col in 1..=w {
            let k = (base + col - 1) as usize;
            let c = r0.c[k];
            let sh = a_buf[k] * shadow_k * (1.0 - c);
            let a = c + sh;
            let (mut r, mut gg, mut b) = (0.0, 0.0, 0.0);
            if c > 0.002 {
                let x = gx + (col as f64 - 0.5) * s;
                let z = r0.z[k];
                let len = (if z < 2.5 { 0.1 + 0.32 * z } else { 0.9 }) * gs;
                let (u, v) = ((x * bc + y * bs) / len, (-x * bs + y * bc) / (0.11 * gs));
                let tex = grain * (0.6 * raster::noise(u, v, 7) + 0.4 * raster::noise(x / (0.35 * gs), y / (0.35 * gs), 11));
                let (cx, cy) = ((x / 0.3).floor(), (y / 0.3).floor());
                let spark = (raster::hash(cx as i64, cy as i64, 23).rem_euclid(1000) as f64) < spark_rate
                    && (((cx + 0.5) * 0.3 - gx) / s).floor() == (col - 1) as f64
                    && ((gy - (cy + 0.5) * 0.3) / s).floor() == (row - 1) as f64;
                let wa = r0.w[k];
                let (mut sxk, mut syk, mut t) = (r0.sx[k], r0.sy[k], r0.t[k]);
                let passes = if wa < 0.999 { 2 } else { 1 };
                for pass in 1..=passes {
                    if pass == 2 {
                        sxk = r0.bx[k];
                        syk = r0.by[k];
                        t = r0.bt[k];
                    }
                    let (nx, ny) = (-sxk, syk);
                    let nn = (nx * nx + ny * ny + 1.0).sqrt();
                    let (nx, ny, nz) = (nx / nn, ny / nn, 1.0 / nn);
                    let (vx, vy, vz) = (ecx - col as f64, row as f64 - ecy, eye);
                    let vn = (vx * vx + vy * vy + vz * vz).sqrt();
                    let (vx, vy, vz) = (vx / vn, vy / vn, vz / vn);
                    let nv = 2.0 * (nx * vx + ny * vy + nz * vz);
                    let (rx, ry, rz) = (nv * nx - vx, nv * ny - vy, nv * nz - vz);
                    let mut i = if l.studio { studio(l, lx, ly, rx, ry, rz) } else { mirror(l, lx, ly, rx, ry) };
                    let sp = nx * hx + ny * hy + nz * hz;
                    if sp > cut {
                        let sp = (sp - cut) / hs;
                        i += hlt * sp.powf(6.0);
                    }
                    i += tex;
                    if spark && nz > 0.9 {
                        i += glint;
                    }
                    let idx = ((if i < 0.0 { 0.0 } else if i > 1.0 { 1.0 } else { i }) * 255.0 + 0.5).floor() as usize;
                    let (g1, c1) = (gl[idx], cl[idx]);
                    let wt = if pass == 1 { wa } else { 1.0 - wa };
                    r += (g1[0] + (c1[0] - g1[0]) * t) * wt;
                    gg += (g1[1] + (c1[1] - g1[1]) * t) * wt;
                    b += (g1[2] + (c1[2] - g1[2]) * t) * wt;
                }
                if wa >= 0.999 && wa < 1.0 {
                    r /= wa;
                    gg /= wa;
                    b /= wa;
                }
                if sh > 0.0 {
                    let wc = c / a;
                    r = r * wc + um[0] * 255.0 * (1.0 - wc);
                    gg = gg * wc + um[1] * 255.0 * (1.0 - wc);
                    b = b * wc + um[2] * 255.0 * (1.0 - wc);
                }
            } else if sh > 0.0 {
                r = um[0] * 255.0;
                gg = um[1] * 255.0;
                b = um[2] * 255.0;
            }
            rgb.push((r + 0.5).floor() as u8);
            rgb.push((gg + 0.5).floor() as u8);
            rgb.push((b + 0.5).floor() as u8);
            alpha.push(((if a > 1.0 { 1.0 } else { a }) * 255.0 + 0.5).floor() as u8);
        }
    }
    (rgb, alpha)
}

/// Paint the shapes pt has collected as one image, and forget them.
pub fn flush(pt: &mut Paint) {
    let p = match pt.gilt_pending.take() {
        Some(p) => p,
        None => return,
    };
    if p.shapes.is_empty() {
        return;
    }
    let look = pt.gilt_env.clone().unwrap_or_else(|| "studio".to_string());
    let l = gilt_look::resolve(&look, &pt.gilt_look).expect("look checked by settings");
    let s = 72.27 / pt.gilt_dpi;
    let shadow = p.shadow * l.shadow_offset;
    let pad = shadow + 3.0 * s;
    let g = raster::grid(p.x0 - pad, p.y0 - pad, p.x1 + pad, p.y1 + pad, s);
    let r = raster_relief::layers(&g, &p.shapes, &l);
    let (rgb, alpha) = shade(&g, &r, shadow / s, l.light.to_radians(), &l);
    let (ix0, iy0, ix1, iy1) = (g.x0, g.y1 - g.h as f64 * s, g.x0 + g.w as f64 * s, g.y1);
    let name = format!("flimg{}", pt.xobjects.len() + 1);
    pt.xobjects.push((name.clone(), Image { w: g.w as usize, h: g.h as usize, rgb, alpha }));
    let mut e = String::new();
    write!(e, "q {:.4} 0 0 {:.4} {:.4} {:.4} cm /{} Do Q", (ix1 - ix0) * BP, (iy1 - iy0) * BP, ix0 * BP, iy0 * BP, name).unwrap();
    pt.emit(&e);
    pt.count += 1;
}
