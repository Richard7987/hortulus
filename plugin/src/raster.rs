//! A small software rasteriser for computed bitmaps (raster gilt).

use crate::geom::Cmd;
use crate::lua::sort;

const K: f64 = 0.5523;
const SUB: i64 = 5;

#[derive(Clone, Copy)]
pub struct Grid {
    pub x0: f64,
    pub y1: f64,
    pub w: i64,
    pub h: i64,
    pub s: f64,
    pub n: usize,
}

pub fn grid(x0: f64, y0: f64, x1: f64, y1: f64, s: f64) -> Grid {
    let (w, h) = ((((x1 - x0) / s).ceil() as i64).max(1), (((y1 - y0) / s).ceil() as i64).max(1));
    Grid { x0, y1: y0 + h as f64 * s, w, h, s, n: (w * h) as usize }
}

fn curve(cur: &mut Vec<f64>, bx: f64, by: f64, cx: f64, cy: f64, dx: f64, dy: f64) {
    let (ax, ay) = (cur[cur.len() - 2], cur[cur.len() - 1]);
    let len = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt()
        + ((cx - bx) * (cx - bx) + (cy - by) * (cy - by)).sqrt()
        + ((dx - cx) * (dx - cx) + (dy - cy) * (dy - cy)).sqrt();
    let n = (len / 1.5).ceil().min(96.0).max(2.0) as i64;
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let u = 1.0 - t;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        cur.push(a * ax + b * bx + c * cx + d * dx);
        cur.push(a * ay + b * by + c * cy + d * dy);
    }
}

/// Path commands as polylines in pixel coordinates (flat x, y lists).
pub fn flatten(g: &Grid, cmds: &[Cmd]) -> Vec<Vec<f64>> {
    let (s, gx, gy) = (g.s, g.x0, g.y1);
    let px = |x: f64, y: f64| ((x - gx) / s, (gy - y) / s);
    let mut polys: Vec<Vec<f64>> = Vec::new();
    let mut open = false;
    for c in cmds {
        match *c {
            Cmd::M(x, y) => {
                let (a, b) = px(x, y);
                polys.push(vec![a, b]);
                open = true;
            }
            Cmd::L(x, y) => {
                let (a, b) = px(x, y);
                let cur = polys.last_mut().unwrap();
                cur.push(a);
                cur.push(b);
            }
            Cmd::C(a, b, c2, d, e, f) => {
                let (bx, by) = px(a, b);
                let (cx, cy) = px(c2, d);
                let (dx, dy) = px(e, f);
                curve(polys.last_mut().unwrap(), bx, by, cx, cy, dx, dy);
            }
            Cmd::Z => {
                if open {
                    let cur = polys.last_mut().unwrap();
                    if cur.len() >= 2 {
                        let (a, b) = (cur[0], cur[1]);
                        cur.push(a);
                        cur.push(b);
                    }
                }
                open = false;
            }
            Cmd::O(x, y, r) => {
                let k = r * K;
                let (a, b) = px(x + r, y);
                let mut cur = vec![a, b];
                for q in [
                    [x + r, y + k, x + k, y + r, x, y + r],
                    [x - k, y + r, x - r, y + k, x - r, y],
                    [x - r, y - k, x - k, y - r, x, y - r],
                    [x + k, y - r, x + r, y - k, x + r, y],
                ] {
                    let (bx, by) = px(q[0], q[1]);
                    let (cx, cy) = px(q[2], q[3]);
                    let (dx, dy) = px(q[4], q[5]);
                    curve(&mut cur, bx, by, cx, cy, dx, dy);
                }
                polys.push(cur);
                open = false;
            }
        }
    }
    polys
}

#[derive(Clone, Copy)]
struct Edge {
    top: f64,
    bot: f64,
    x: f64,
    slope: f64,
    wind: i64,
}

pub fn fill(g: &Grid, polys: &[Vec<f64>], evenodd: bool) -> Vec<f64> {
    let (w, h) = (g.w, g.h);
    let mut cov = vec![0.0; g.n];
    let mut edges: Vec<Edge> = Vec::new();
    for p in polys {
        let n = p.len();
        if n >= 4 {
            let mut i = 0;
            while i < n {
                let (ax, ay) = (p[i], p[i + 1]);
                let j = if i + 2 >= n { 0 } else { i + 2 };
                let (bx, by) = (p[j], p[j + 1]);
                if ay != by {
                    if ay < by {
                        edges.push(Edge { top: ay, bot: by, x: ax, slope: (bx - ax) / (by - ay), wind: 1 });
                    } else {
                        edges.push(Edge { top: by, bot: ay, x: bx, slope: (ax - bx) / (ay - by), wind: -1 });
                    }
                }
                i += 2;
            }
        }
    }
    sort(&mut edges, |a, b| a.top < b.top);
    let ne = edges.len();
    let mut next_e = 0;
    let mut active: Vec<Edge> = Vec::new();
    let (mut xs, mut ws): (Vec<f64>, Vec<i64>) = (Vec::new(), Vec::new());
    let mut d = vec![0.0; (w + 2) as usize];
    let wsub = 1.0 / SUB as f64;
    let r0 = if ne > 0 { (edges[0].top.floor() as i64 + 1).max(1) } else { h + 1 };
    let wf = w as f64;
    for row in r0..=h {
        for v in d.iter_mut() {
            *v = 0.0;
        }
        let base = ((row - 1) * w) as usize;
        for k in 1..=SUB {
            let y = (row - 1) as f64 + (k as f64 - 0.5) * wsub;
            while next_e < ne && edges[next_e].top <= y {
                active.push(edges[next_e]);
                next_e += 1;
            }
            xs.clear();
            ws.clear();
            let mut j = 0;
            while j < active.len() {
                let e = active[j];
                if e.bot <= y {
                    active.swap_remove(j);
                } else {
                    if e.top <= y {
                        let x = e.x + (y - e.top) * e.slope;
                        let mut m = xs.len();
                        xs.push(x);
                        ws.push(e.wind);
                        while m > 0 && xs[m - 1] > x {
                            xs[m] = xs[m - 1];
                            ws[m] = ws[m - 1];
                            m -= 1;
                        }
                        xs[m] = x;
                        ws[m] = e.wind;
                    }
                    j += 1;
                }
            }
            let mut wind = 0;
            for c in 0..xs.len().saturating_sub(1) {
                wind += ws[c];
                let inside = if evenodd { wind.rem_euclid(2) == 1 } else { wind != 0 };
                if inside {
                    let (a, b) = (xs[c].max(0.0), xs[c + 1].min(wf));
                    if b > a {
                        let (ia, ib) = (a.floor() as i64 + 1, b.floor() as i64 + 1);
                        if ia == ib {
                            cov[base + ia as usize - 1] += (b - a) * wsub;
                        } else {
                            cov[base + ia as usize - 1] += (ia as f64 - a) * wsub;
                            if ib <= w {
                                cov[base + ib as usize - 1] += (b - ib as f64 + 1.0) * wsub;
                            }
                            d[ia as usize] += wsub;
                            d[ib as usize - 1] -= wsub;
                        }
                    }
                }
            }
        }
        let mut run = 0.0;
        for i in 0..w as usize {
            run += d[i];
            let v = cov[base + i] + run;
            cov[base + i] = if v > 1.0 {
                1.0
            } else if v < 0.0 {
                0.0
            } else {
                v
            };
        }
        if next_e >= ne && active.is_empty() {
            break;
        }
    }
    cov
}

pub fn stroke(g: &Grid, polys: &[Vec<f64>], r: f64) -> (Vec<f64>, Vec<f64>) {
    let (w, h) = (g.w, g.h);
    let mut cov = vec![0.0; g.n];
    let mut din = vec![-1e9; g.n];
    let reach = r + 1.0;
    for p0 in polys {
        let tmp;
        let p: &[f64] = if p0.len() == 2 {
            tmp = [p0[0], p0[1], p0[0], p0[1]];
            &tmp
        } else {
            p0
        };
        let n = p.len();
        let mut i = 0;
        while i + 3 < n {
            let (ax, ay, bx, by) = (p[i], p[i + 1], p[i + 2], p[i + 3]);
            let (vx, vy) = (bx - ax, by - ay);
            let ll = vx * vx + vy * vy;
            let c0 = ((ax.min(bx) - reach).floor() as i64 + 1).max(1);
            let c1 = ((ax.max(bx) + reach).floor() as i64 + 1).min(w);
            let q0 = ((ay.min(by) - reach).floor() as i64 + 1).max(1);
            let q1 = ((ay.max(by) + reach).floor() as i64 + 1).min(h);
            for row in q0..=q1 {
                let py = row as f64 - 0.5;
                let base = (row - 1) * w;
                for col in c0..=c1 {
                    let px = col as f64 - 0.5;
                    let t = if ll > 0.0 { ((px - ax) * vx + (py - ay) * vy) / ll } else { 0.0 };
                    let t = if t < 0.0 {
                        0.0
                    } else if t > 1.0 {
                        1.0
                    } else {
                        t
                    };
                    let (dx, dy) = (px - ax - t * vx, py - ay - t * vy);
                    let e = r - (dx * dx + dy * dy).sqrt();
                    let k = (base + col - 1) as usize;
                    if e > din[k] {
                        din[k] = e;
                        let c = e + 0.5;
                        cov[k] = if c > 1.0 {
                            1.0
                        } else if c < 0.0 {
                            0.0
                        } else {
                            c
                        };
                    }
                }
            }
            i += 2;
        }
    }
    (cov, din)
}

pub fn segment_distance(g: &Grid, segs: &[[f64; 4]], band: f64, d2: &mut [f64]) {
    let (w, h) = (g.w, g.h);
    for s in segs {
        let (ax, ay, bx, by) = (s[0], s[1], s[2], s[3]);
        let (vx, vy) = (bx - ax, by - ay);
        let ll = vx * vx + vy * vy;
        let c0 = ((ax.min(bx) - band).floor() as i64 + 1).max(1);
        let c1 = ((ax.max(bx) + band).floor() as i64 + 1).min(w);
        let q0 = ((ay.min(by) - band).floor() as i64 + 1).max(1);
        let q1 = ((ay.max(by) + band).floor() as i64 + 1).min(h);
        for row in q0..=q1 {
            let py = row as f64 - 0.5;
            let base = (row - 1) * w;
            for col in c0..=c1 {
                let px = col as f64 - 0.5;
                let t = if ll > 0.0 { ((px - ax) * vx + (py - ay) * vy) / ll } else { 0.0 };
                let t = if t < 0.0 {
                    0.0
                } else if t > 1.0 {
                    1.0
                } else {
                    t
                };
                let (dx, dy) = (px - ax - t * vx, py - ay - t * vy);
                let e = dx * dx + dy * dy;
                let k = (base + col - 1) as usize;
                if e < d2[k] {
                    d2[k] = e;
                }
            }
        }
    }
}

fn dt1(f: &mut [f64], o: usize, step: usize, n: usize, v: &mut [usize], z: &mut [f64], tmp: &mut [f64]) {
    for i in 0..n {
        tmp[i] = f[o + i * step];
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = -1e30;
    z[1] = 1e30;
    for q in 1..n {
        let qf = q as f64;
        let fq = tmp[q] + qf * qf;
        let mut p = v[k];
        let mut s = (fq - (tmp[p] + (p * p) as f64)) / (2.0 * (q - p) as f64);
        while s <= z[k] {
            k -= 1;
            p = v[k];
            s = (fq - (tmp[p] + (p * p) as f64)) / (2.0 * (q - p) as f64);
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = 1e30;
    }
    k = 0;
    for q in 0..n {
        while z[k + 1] < q as f64 {
            k += 1;
        }
        let p = v[k];
        let dq = q as f64 - p as f64;
        f[o + q * step] = dq * dq + tmp[p];
    }
}

pub fn inside_distance(g: &Grid, cov: &[f64]) -> Vec<f64> {
    let (w, h, n) = (g.w as usize, g.h as usize, g.n);
    const INF: f64 = 1e20;
    let mut f: Vec<f64> = cov.iter().map(|c| if *c >= 0.5 { INF } else { 0.0 }).collect();
    let m = w.max(h) + 2;
    let (mut v, mut z, mut tmp) = (vec![0usize; m], vec![0.0; m], vec![0.0; m]);
    for col in 0..w {
        dt1(&mut f, col, w, h, &mut v, &mut z, &mut tmp);
    }
    for row in 0..h {
        dt1(&mut f, row * w, 1, w, &mut v, &mut z, &mut tmp);
    }
    for i in 0..n {
        let c = cov[i];
        if c < 0.5 {
            f[i] = c - 0.5;
        } else {
            let dd = if f[i] >= INF { ((w + h) * (w + h)) as f64 } else { f[i] };
            f[i] = if dd <= 1.0 { c - 0.5 } else { dd.sqrt() - 0.5 };
        }
    }
    f
}

pub fn blur(g: &Grid, a: &mut [f64], passes: usize) {
    let (w, h) = (g.w as usize, g.h as usize);
    let mut t = vec![0.0; g.n];
    for _ in 0..passes {
        for row in 0..h {
            let base = row * w;
            for col in 0..w {
                let l = if col > 0 { a[base + col - 1] } else { a[base + col] };
                let r = if col + 1 < w { a[base + col + 1] } else { a[base + col] };
                t[base + col] = (l + a[base + col] + r) / 3.0;
            }
        }
        for row in 0..h {
            let base = row * w;
            let up = if row > 0 { base - w } else { base };
            let dn = if row + 1 < h { base + w } else { base };
            for col in 0..w {
                a[base + col] = (t[up + col] + t[base + col] + t[dn + col]) / 3.0;
            }
        }
    }
}

#[inline]
pub fn hash(x: i64, y: i64, seed: i64) -> i64 {
    let hh = (x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263)).wrapping_add(seed.wrapping_mul(2246822519))) & 0xffffffff;
    let hh = ((hh ^ (hh >> 13)).wrapping_mul(1274126177)) & 0xffffffff;
    hh ^ (hh >> 16)
}

#[inline]
pub fn noise(x: f64, y: f64, seed: i64) -> f64 {
    let (fxi, fyi) = (x.floor(), y.floor());
    let (ix, iy) = (fxi as i64, fyi as i64);
    let (fx, fy) = (x - fxi, y - fyi);
    let (fx, fy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (a, b) = (hash(ix, iy, seed) as f64, hash(ix + 1, iy, seed) as f64);
    let (c, d) = (hash(ix, iy + 1, seed) as f64, hash(ix + 1, iy + 1, seed) as f64);
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    (top + (bot - top) * fy) / 2147483647.5 - 1.0
}
