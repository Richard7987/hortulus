//! The relief of raster gilt: every painted piece's surface slope, laid in painting order.

use crate::geom::Cmd;
use crate::gilt_look::Look;
use crate::lua::sort;
use crate::raster::{self, Grid};
use std::f64::consts::PI;

fn profile(dd: f64, rr: f64, r2: f64, tilt: f64, cushion: f64) -> f64 {
    if dd <= 0.0 {
        return 0.0;
    }
    let t = dd.min(rr) / rr;
    let t2 = dd.min(r2) / r2;
    tilt * rr * (t * PI / 2.0).sin() + cushion * r2 * (t2 * PI / 2.0).sin()
}

#[derive(Clone, Copy)]
struct Win {
    c0: i64,
    r0: i64,
    w: i64,
    h: i64,
}

fn window(g: &Grid, x0: f64, y0: f64, x1: f64, y1: f64) -> Win {
    let (c0, r0) = ((x0.floor() as i64 - 1).max(0), (y0.floor() as i64 - 1).max(0));
    let (c1, r1) = ((x1.floor() as i64 + 2).min(g.w), (y1.floor() as i64 + 2).min(g.h));
    Win { c0, r0, w: (c1 - c0).max(1), h: (r1 - r0).max(1) }
}

fn grid_of(g: &Grid, w: &Win) -> Grid {
    Grid { x0: g.x0 + w.c0 as f64 * g.s, y1: g.y1 - w.r0 as f64 * g.s, w: w.w, h: w.h, s: g.s, n: (w.w * w.h) as usize }
}

fn shifted(polys: &[Vec<f64>], w: &Win) -> Vec<Vec<f64>> {
    polys
        .iter()
        .map(|p| {
            let mut q = p.clone();
            let mut j = 0;
            while j + 1 < q.len() {
                q[j] -= w.c0 as f64;
                q[j + 1] -= w.r0 as f64;
                j += 2;
            }
            q
        })
        .collect()
}

fn simplify(p: &[f64], i: usize, j: usize, tol: f64, out: &mut Vec<usize>) {
    let (ax, ay, bx, by) = (p[2 * i - 2], p[2 * i - 1], p[2 * j - 2], p[2 * j - 1]);
    let (vx, vy) = (bx - ax, by - ay);
    let ll = vx * vx + vy * vy;
    let (mut far, mut at) = (-1.0, None);
    for k in i + 1..j {
        let (px, py) = (p[2 * k - 2] - ax, p[2 * k - 1] - ay);
        let t = if ll > 0.0 { (px * vx + py * vy) / ll } else { 0.0 };
        let t = if t < 0.0 {
            0.0
        } else if t > 1.0 {
            1.0
        } else {
            t
        };
        let (dx, dy) = (px - t * vx, py - t * vy);
        let e = dx * dx + dy * dy;
        if e > far {
            far = e;
            at = Some(k);
        }
    }
    match at {
        Some(a) if far > tol * tol => {
            simplify(p, i, a, tol, out);
            simplify(p, a, j, tol, out);
        }
        _ => out.push(j),
    }
}

fn segments(polys: &[Vec<f64>]) -> Vec<[f64; 4]> {
    let mut segs = Vec::new();
    for p in polys {
        let n = p.len() / 2;
        if n >= 2 {
            let mut keep = vec![1];
            simplify(p, 1, n, 0.05, &mut keep);
            for k in 0..keep.len() - 1 {
                let (a, b) = (keep[k], keep[k + 1]);
                segs.push([p[2 * a - 2], p[2 * a - 1], p[2 * b - 2], p[2 * b - 1]]);
            }
            if p[2 * n - 2] != p[0] || p[2 * n - 1] != p[1] {
                segs.push([p[2 * n - 2], p[2 * n - 1], p[0], p[1]]);
            }
        }
    }
    segs
}

fn on_edge(segs: &[[f64; 4]], x: f64, y: f64) -> bool {
    for s in segs {
        let (vx, vy) = (s[2] - s[0], s[3] - s[1]);
        let ll = vx * vx + vy * vy;
        let t = if ll > 0.0 { ((x - s[0]) * vx + (y - s[1]) * vy) / ll } else { 0.0 };
        let t = if t < 0.0 {
            0.0
        } else if t > 1.0 {
            1.0
        } else {
            t
        };
        let (dx, dy) = (x - s[0] - t * vx, y - s[1] - t * vy);
        if dx * dx + dy * dy < 1e-6 {
            return true;
        }
    }
    false
}

fn inside(segs: &[[f64; 4]], x: f64, y: f64) -> bool {
    let mut odd = false;
    for s in segs {
        if (s[1] > y) != (s[3] > y) && x < s[0] + (y - s[1]) * (s[2] - s[0]) / (s[3] - s[1]) {
            odd = !odd;
        }
    }
    odd
}

struct Piece {
    fill: bool,
    tone: f64,
    call: i64,
    bevel: Option<f64>,
    half: f64,
    win: Win,
    order: usize,
    cov: Vec<f64>,
    din: Vec<f64>,
    segs: Vec<[f64; 4]>,
    shared: Vec<bool>,
}

fn share(a: &Piece, b: &Piece) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, s) in a.segs.iter().enumerate() {
        let (mx, my) = ((s[0] + s[2]) / 2.0, (s[1] + s[3]) / 2.0);
        if on_edge(&b.segs, s[0], s[1]) && on_edge(&b.segs, s[2], s[3]) && on_edge(&b.segs, mx, my) {
            let (vx, vy) = (s[2] - s[0], s[3] - s[1]);
            let l = (vx * vx + vy * vy).sqrt();
            if l > 0.0 {
                let (nx, ny) = (-vy / l * 0.25, vx / l * 0.25);
                let (ap, an) = (inside(&a.segs, mx + nx, my + ny), inside(&a.segs, mx - nx, my - ny));
                if ap != an && inside(&b.segs, mx + nx, my + ny) == an && inside(&b.segs, mx - nx, my - ny) == ap {
                    out.push(i);
                }
            }
        }
    }
    out
}

fn overlap(a: &Piece, b: &Piece) -> bool {
    a.win.c0 < b.win.c0 + b.win.w && b.win.c0 < a.win.c0 + a.win.w && a.win.r0 < b.win.r0 + b.win.h && b.win.r0 < a.win.r0 + a.win.h
}

struct PairRec {
    i: usize,
    j: usize,
    len: f64,
    ab: Vec<usize>,
    ba: Vec<usize>,
}

fn groups(pieces: &mut [Piece]) -> Vec<Vec<usize>> {
    let n = pieces.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(parent: &[usize], mut i: usize) -> usize {
        while parent[i] != i {
            i = parent[i];
        }
        i
    }
    let mut pairs: Vec<PairRec> = Vec::new();
    for i in 0..n {
        if !pieces[i].fill {
            continue;
        }
        for j in 0..n {
            if pieces[j].call != pieces[i].call + 1 {
                continue;
            }
            let (a, b) = (&pieces[i], &pieces[j]);
            if b.fill && a.tone != b.tone && overlap(a, b) {
                let (ab, ba) = (share(a, b), share(b, a));
                if !ab.is_empty() && !ba.is_empty() {
                    let mut len = 0.0;
                    for k in &ab {
                        let s = a.segs[*k];
                        len += ((s[2] - s[0]) * (s[2] - s[0]) + (s[3] - s[1]) * (s[3] - s[1])).sqrt();
                    }
                    pairs.push(PairRec { i, j, len, ab, ba });
                }
            }
        }
    }
    sort(&mut pairs, |x, y| {
        if x.len != y.len {
            x.len > y.len
        } else if x.i != y.i {
            x.i < y.i
        } else {
            x.j < y.j
        }
    });
    let mut taken = vec![false; n];
    for q in &pairs {
        if !taken[q.i] && !taken[q.j] {
            taken[q.i] = true;
            taken[q.j] = true;
            for k in &q.ab {
                pieces[q.i].shared[*k] = true;
            }
            for k in &q.ba {
                pieces[q.j].shared[*k] = true;
            }
            let (ri, rj) = (root(&parent, q.i), root(&parent, q.j));
            parent[rj] = ri;
        }
    }
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut at = vec![usize::MAX; n];
    for i in 0..n {
        let r = root(&parent, i);
        if at[r] == usize::MAX {
            at[r] = out.len();
            out.push(Vec::new());
        }
        out[at[r]].push(i);
    }
    out
}

struct Rel {
    win: Win,
    cov: Vec<f64>,
    sx: Vec<f64>,
    sy: Vec<f64>,
    tone: Vec<f64>,
    second: Option<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    thick: f64,
    top: usize,
}

fn slopes(gr: &Grid, win: Win, cov: Vec<f64>, tone: Vec<f64>, h: &[f64], thick: f64) -> Rel {
    let (w, ht) = (gr.w as usize, gr.h as usize);
    let mut hs = vec![0.0; gr.n];
    for row in 0..ht {
        for col in 0..w {
            let k = row * w + col;
            if cov[k] > 0.0 {
                let (mut sum, mut wt) = (0.0, 0.0);
                for r in row.saturating_sub(1)..=(row + 1).min(ht - 1) {
                    for c in col.saturating_sub(1)..=(col + 1).min(w - 1) {
                        let q = r * w + c;
                        let cw = cov[q];
                        sum += h[q] * cw;
                        wt += cw;
                    }
                }
                hs[k] = sum / wt;
            }
        }
    }
    let (mut sx, mut sy) = (vec![0.0; gr.n], vec![0.0; gr.n]);
    for row in 0..ht {
        let base = row * w;
        let up = if row > 0 { base - w } else { base };
        let dn = if row + 1 < ht { base + w } else { base };
        for col in 0..w {
            let k = base + col;
            sx[k] = ((if col + 1 < w { hs[k + 1] } else { hs[k] }) - (if col > 0 { hs[k - 1] } else { hs[k] })) * 0.5;
            sy[k] = (hs[dn + col] - hs[up + col]) * 0.5;
        }
    }
    Rel { win, cov, sx, sy, tone, second: None, thick, top: 0 }
}

fn roof(g: &Grid, grp: &[&Piece], gr: &Grid, win: Win, cov: Vec<f64>, l: &Look) -> Rel {
    let (w, n) = (gr.w, gr.n);
    let mut far: f64 = 0.0;
    let mut faces = Vec::new();
    for p in grp {
        let (mut best, mut bl) = (None, 0.0);
        for (i, s) in p.segs.iter().enumerate() {
            let ln = (s[2] - s[0]) * (s[2] - s[0]) + (s[3] - s[1]) * (s[3] - s[1]);
            if !p.shared[i] && ln > bl {
                best = Some(*s);
                bl = ln;
            }
        }
        let best = best.unwrap_or(p.segs[0]);
        let ln = bl.sqrt();
        let (mut nx, mut ny) = (-(best[3] - best[1]) / ln, (best[2] - best[0]) / ln);
        let mut top: f64 = 0.0;
        for s in &p.segs {
            let e = (s[0] - best[0]) * nx + (s[1] - best[1]) * ny;
            if e.abs() > top.abs() {
                top = e;
            }
        }
        if top < 0.0 {
            nx = -nx;
            ny = -ny;
        }
        far = far.max(top.abs());
        let mut fc = vec![0.0; n];
        let pw = p.win;
        for row in 1..=pw.h {
            let src = (row - 1) * pw.w;
            let dst = (row + pw.r0 - win.r0 - 1) * w + pw.c0 - win.c0;
            for col in 1..=pw.w {
                fc[(dst + col - 1) as usize] = p.cov[(src + col - 1) as usize];
            }
        }
        faces.push((fc, l.facet * nx, l.facet * ny, p.tone));
    }
    let (a, b) = (&faces[0], &faces[1]);
    let sx = vec![a.1; n];
    let sy = vec![a.2; n];
    let tone = vec![a.3; n];
    let bx = vec![b.1; n];
    let by = vec![b.2; n];
    let bt = vec![b.3; n];
    let wa: Vec<f64> = (0..n)
        .map(|i| {
            let (ca, cb) = (a.0[i], b.0[i]);
            if ca + cb > 0.0 {
                ca / (ca + cb)
            } else {
                1.0
            }
        })
        .collect();
    Rel { win, cov, sx, sy, tone, second: Some((bx, by, bt, wa)), thick: far * g.s, top: 0 }
}

fn group_relief(g: &Grid, grp: &[&Piece], l: &Look) -> Rel {
    let (mut x0, mut y0, mut x1, mut y1) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for p in grp {
        let w = p.win;
        x0 = x0.min(w.c0);
        y0 = y0.min(w.r0);
        x1 = x1.max(w.c0 + w.w);
        y1 = y1.max(w.r0 + w.h);
    }
    let win = Win { c0: x0, r0: y0, w: x1 - x0, h: y1 - y0 };
    let gr = grid_of(g, &win);
    let n = gr.n;
    let (mut cov, mut tone) = (vec![0.0; n], vec![0.0; n]);
    for p in grp {
        let pw = p.win;
        for row in 1..=pw.h {
            let src = (row - 1) * pw.w;
            let dst = (row + pw.r0 - y0 - 1) * gr.w + pw.c0 - x0;
            for col in 1..=pw.w {
                let c = p.cov[(src + col - 1) as usize];
                let k = (dst + col - 1) as usize;
                cov[k] += c;
                tone[k] += c * p.tone;
            }
        }
    }
    for i in 0..n {
        let c = cov[i];
        if c > 0.0 {
            tone[i] /= c;
        }
        cov[i] = if c > 1.0 { 1.0 } else { c };
    }
    if grp.len() > 1 {
        return roof(g, grp, &gr, win, cov, l);
    }
    let single = grp[0];
    let bev;
    let mut owned = Vec::new();
    let d: &[f64] = if !single.fill {
        bev = single.half / g.s;
        &single.din
    } else {
        bev = single.bevel.map_or(0.9, |b| b * 2.2) / g.s;
        let band = bev + 2.0;
        let segs: Vec<[f64; 4]> = single.segs.iter().map(|s| [s[0] - x0 as f64, s[1] - y0 as f64, s[2] - x0 as f64, s[3] - y0 as f64]).collect();
        let mut d2 = vec![f64::INFINITY; n];
        raster::segment_distance(&gr, &segs, band, &mut d2);
        owned = vec![0.0; n];
        let mut e: Option<Vec<f64>> = None;
        for i in 0..n {
            let c = cov[i];
            if c < 0.5 {
                owned[i] = c - 0.5;
            } else {
                let exact = d2[i].sqrt();
                if exact <= band {
                    owned[i] = exact;
                } else {
                    let ev = e.get_or_insert_with(|| raster::inside_distance(&gr, &cov));
                    owned[i] = ev[i].max(band);
                }
            }
        }
        &owned
    };
    let mut dmax: f64 = 0.0;
    for v in d {
        if *v > dmax {
            dmax = *v;
        }
    }
    let rr = bev.min(dmax).max(0.5);
    let r2 = dmax.min(rr * 5.0).max(rr);
    let thick = dmax * g.s;
    let tilt = l.relief * if thick >= 1.0 { 1.0 } else { thick.max(0.55) };
    let h: Vec<f64> = d.iter().map(|v| profile(*v, rr, r2, tilt, l.cushion)).collect();
    drop(owned);
    slopes(&gr, win, cov, tone, &h, thick)
}

pub struct Shape {
    pub stroke: bool,
    pub paths: Vec<Vec<Cmd>>,
    pub evenodd: bool,
    pub half: f64,
    pub call: i64,
    pub bevel: Option<f64>,
    pub tone: f64,
}

pub struct Layers {
    pub c: Vec<f64>,
    pub sx: Vec<f64>,
    pub sy: Vec<f64>,
    pub t: Vec<f64>,
    pub z: Vec<f64>,
    pub w: Vec<f64>,
    pub bx: Vec<f64>,
    pub by: Vec<f64>,
    pub bt: Vec<f64>,
}

pub fn layers(g: &Grid, shapes: &[Shape], l: &Look) -> Layers {
    let mut pieces: Vec<Piece> = Vec::new();
    for (i, sh) in shapes.iter().enumerate() {
        let mut polys: Vec<Vec<f64>> = Vec::new();
        for cmds in &sh.paths {
            polys.extend(raster::flatten(g, cmds));
        }
        let hp = sh.half / g.s;
        let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for p in &polys {
            let mut j = 0;
            while j + 1 < p.len() {
                x0 = x0.min(p[j]);
                x1 = x1.max(p[j]);
                y0 = y0.min(p[j + 1]);
                y1 = y1.max(p[j + 1]);
                j += 2;
            }
        }
        if x0 <= x1 {
            let win = window(g, x0 - hp, y0 - hp, x1 + hp, y1 + hp);
            let sg = grid_of(g, &win);
            let local = shifted(&polys, &win);
            let mut pc = Piece { fill: !sh.stroke, tone: sh.tone, call: sh.call, bevel: sh.bevel, half: sh.half, win, order: i + 1, cov: Vec::new(), din: Vec::new(), segs: Vec::new(), shared: Vec::new() };
            if sh.stroke {
                let (c, d) = raster::stroke(&sg, &local, hp);
                pc.cov = c;
                pc.din = d;
            } else {
                pc.cov = raster::fill(&sg, &local, sh.evenodd);
                pc.segs = segments(&polys);
                pc.shared = vec![false; pc.segs.len()];
            }
            pieces.push(pc);
        }
    }
    let grp_list = groups(&mut pieces);
    let mut rels: Vec<Rel> = Vec::with_capacity(grp_list.len());
    for grp in &grp_list {
        let refs: Vec<&Piece> = grp.iter().map(|i| &pieces[*i]).collect();
        let mut rel = group_relief(g, &refs, l);
        rel.top = refs.iter().map(|p| p.order).max().unwrap_or(0);
        rels.push(rel);
    }
    sort(&mut rels, |a, b| a.top < b.top);
    let n = g.n;
    let mut o = Layers { c: vec![0.0; n], sx: vec![0.0; n], sy: vec![0.0; n], t: vec![0.0; n], z: vec![0.0; n], w: vec![0.0; n], bx: vec![0.0; n], by: vec![0.0; n], bt: vec![0.0; n] };
    let gw = g.w;
    for rel in &rels {
        let rw = rel.win;
        for row in 1..=rw.h {
            let src = (row - 1) * rw.w;
            let dst = (row + rw.r0 - 1) * gw + rw.c0;
            for col in 1..=rw.w {
                let q = (src + col - 1) as usize;
                let c = rel.cov[q];
                if c > 0.0 {
                    let k = (dst + col - 1) as usize;
                    let co = o.c[k];
                    let cn = c + (1.0 - c) * co;
                    let keep = (1.0 - c) * co / cn;
                    let (o1, o2) = (keep * o.w[k], keep * (1.0 - o.w[k]));
                    let fa = rel.second.as_ref().map_or(1.0, |s| s.3[q]);
                    let (mut n1, n2) = (c / cn * fa, c / cn * (1.0 - fa));
                    let (mut ax, mut ay, mut at) = (rel.sx[q], rel.sy[q], rel.tone[q]);
                    let (mut w2, mut x2, mut y2, mut t2) = (n2, 0.0, 0.0, 0.0);
                    if let Some(s) = &rel.second {
                        x2 = s.0[q];
                        y2 = s.1[q];
                        t2 = s.2[q];
                    }
                    if o1 > w2 {
                        w2 = o1;
                        x2 = o.sx[k];
                        y2 = o.sy[k];
                        t2 = o.t[k];
                    }
                    if o2 > w2 {
                        w2 = o2;
                        x2 = o.bx[k];
                        y2 = o.by[k];
                        t2 = o.bt[k];
                    }
                    if w2 > n1 {
                        std::mem::swap(&mut ax, &mut x2);
                        std::mem::swap(&mut ay, &mut y2);
                        std::mem::swap(&mut at, &mut t2);
                        std::mem::swap(&mut n1, &mut w2);
                    }
                    o.sx[k] = ax;
                    o.sy[k] = ay;
                    o.t[k] = at;
                    o.bx[k] = x2;
                    o.by[k] = y2;
                    o.bt[k] = t2;
                    o.w[k] = n1 / (n1 + w2);
                    o.z[k] = o.z[k] * (1.0 - c) + rel.thick * c;
                    o.c[k] = cn;
                }
            }
        }
    }
    o
}
