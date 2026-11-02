//! Placement on the clearance field, in the order a plant grows.

use crate::dist;
use crate::fill;
use crate::flower::{self, BloomOpts, TiltedOpts};
use crate::geom::{self as g, Pt};
use crate::leaf::{self, Attach, LeafOpts};
use crate::mix::{self, Dens, Weights};
use crate::paint::Paint;
use crate::plant::{self, StemRef};
use crate::rng::Rng;
use crate::space::Space;
use crate::sprig::{self, Batch};
use crate::tune::{tune, Compose};
use std::collections::HashMap;
use std::f64::consts::PI;

const REACH: f64 = 70.0;
const DISC_K: f64 = 0.85;

pub type Painter = Box<dyn FnMut(&mut Paint)>;

fn jitter(k: i64, iter: i64) -> f64 {
    let v = (k as f64 * 12.9898 + iter as f64 * 78.233).sin() * 43758.5453;
    0.85 + 0.3 * (v - v.floor())
}

pub fn open_path(s: &Space, pts: &[Pt]) -> bool {
    pts.iter().step_by(2).all(|p| s.open_at(p[0], p[1]))
}

fn leaf_discs(o: &LeafOpts, p: Pt, ang: f64) -> Vec<[f64; 3]> {
    let spine = g::curve(p, ang, o.len, o.bend, o.hook, 20);
    [0.2, 0.36, 0.52, 0.68, 0.84]
        .iter()
        .map(|t: &f64| {
            let q = spine[(t * (spine.len() as f64 - 1.0) + 0.5).floor() as usize];
            [q[0], q[1], 1.1 * o.width * leaf::profile(o, *t)]
        })
        .collect()
}

fn discs_fit(s: &Space, discs: &[[f64; 3]], k: f64) -> bool {
    discs.iter().all(|d| s.room(d[0], d[1]) >= k * d[2])
}

fn joint(r: &Rng, p: Pt, ang: f64, len: f64, st: &StemRef, t: f64) -> (Attach, Pt) {
    let f = r.fork("attach").range(-1.0, 1.0);
    let stem = st.borrow();
    let w = stem.width(t);
    let (_, ta) = g::at(&stem.pts, t);
    let side = if (ang - ta).sin() >= 0.0 { 1.0 } else { -1.0 };
    let seg: Vec<Pt> = stem.pts.iter().copied().filter(|q| g::dist(*q, p) < len * 1.1).collect();
    let base = if f.abs() < leaf::FRONT { g::polar(p, ta + side * PI / 2.0, w / 2.0 + 3.0) } else { p };
    (Attach { f, p, seg, w }, base)
}

fn outline_clear(s: &Space, r: &Rng, o: &LeafOpts, p: Pt, ang: f64) -> bool {
    let local = leaf::outline(&mut r.clone(), o).0;
    open_path(s, &g::transform(&local, p[0], p[1], ang - PI / 2.0, 1.0, 1.0))
}

pub fn grow_leaf(s: &mut Space, r: &mut Rng, p: Pt, ang: f64, len: f64, overlap: f64, st: &StemRef, t: f64) -> Option<Painter> {
    let k = 1.0 - overlap;
    let mut o = leaf::options(r, len, ang);
    let (at, base) = joint(r, p, ang, len, st, t);
    for _ in 0..3 {
        let discs = leaf_discs(&o, p, ang);
        if discs_fit(s, &discs, k) && outline_clear(s, r, &o, p, ang) {
            for d in &discs {
                s.disc(d[0], d[1], d[2], k);
            }
            let mut mate = None;
            let mut mr = r.fork("mate");
            if o.margin == "entire" && o.len < 69.0 && mr.chance(0.35) {
                let ma = ang + (if mr.chance(0.5) { 1.0 } else { -1.0 }) * mr.range(0.45, 0.7);
                let mo = leaf::options(&mut mr, o.len * 0.65, ma);
                let md = leaf_discs(&mo, p, ma);
                if discs_fit(s, &md, k) && outline_clear(s, &mr, &mo, p, ma) {
                    for d in &md {
                        s.disc(d[0], d[1], d[2], k);
                    }
                    mate = Some((ma, mo));
                }
            }
            let mut o = o;
            o.attach = Some(at);
            let mut r = r.clone();
            return Some(Box::new(move |pt: &mut Paint| {
                if let Some((ma, mo)) = &mut mate {
                    mo.attach = o.attach.clone();
                    leaf::leaf(pt, &mut mr, base[0], base[1], *ma, mo);
                }
                leaf::leaf(pt, &mut r, base[0], base[1], ang, &o);
            }));
        }
        o.len *= 0.78;
        o.width *= 0.78;
        if o.len < 16.0 {
            return None;
        }
    }
    None
}

pub struct Ctx<'a> {
    pub cfg: Compose,
    pub mix: Option<&'a Weights>,
    pub mw: f64,
    pub dens: Dens,
    pub ylo: f64,
    pub span: f64,
    pub strands: Vec<StemRef>,
    pub stems: Vec<StemRef>,
    pub stem_batch: Batch,
    pub sprays: Batch,
    pub blooms: Vec<Painter>,
    pub back: Vec<Painter>,
    pub leaves: Vec<Painter>,
    pub leaflets: Vec<[f64; 4]>,
    pub visitors: Vec<Painter>,
    pub counts: HashMap<&'static str, i64>,
}

fn stemlet(s: &mut Space, rng: &mut Rng, ctx: &mut Ctx, p: Pt, ang: f64, len: f64, side: f64) -> bool {
    let stalk = len * rng.range(0.35, 0.5);
    let bend = side * rng.range(0.2, 0.5);
    let curve = g::curve(p, ang, stalk, bend, 0.0, 6);
    if !open_path(s, &curve) {
        return false;
    }
    let tip = curve[curve.len() - 1];
    let ta = g::angle(curve[curve.len() - 2], tip);
    let size = ((len - stalk) * 0.85).min(11.0);
    ctx.sprays.line(&curve, 1);
    let spread = rng.range(0.45, 0.7);
    let fan: &[f64] = if rng.chance(0.5) { &[-spread, spread] } else { &[-spread, 0.0, spread] };
    for da in fan {
        ctx.leaflets.push([tip[0], tip[1], ta + da, size * if *da == 0.0 { 1.0 } else { 0.8 }]);
    }
    s.disc(tip[0], tip[1], size * 0.6, DISC_K);
    true
}

pub fn node_leaves(s: &mut Space, rng: &mut Rng, ctx: &mut Ctx) {
    let strands = ctx.strands.clone();
    let sp = ctx.cfg.leaf_spacing;
    let sizes = &tune().sizes;
    for (ki, st) in strands.iter().enumerate() {
        let k = ki + 1;
        let pts = st.borrow().pts.clone();
        let total = g::length(&pts);
        let mut side = if rng.chance(0.5) { 1.0 } else { -1.0 };
        let mut d = rng.rangev(sp) * 0.5;
        let mut n = 0;
        while d < total - 8.0 && (ctx.leaves.len() as f64) < ctx.cfg.leaves {
            n += 1;
            let (p, a) = g::at(&pts, d / total);
            let dn = (ctx.dens)(p[0], p[1]);
            side = -side;
            let ang = a + side * rng.range(0.6, 1.05);
            let len = (ctx.mw * 0.85).min(130.0) * ctx.cfg.leaf_size * (0.45 + 0.55 * dn) * dist::sample(rng, &sizes.leaf);
            let painted = if len >= 24.0 {
                grow_leaf(s, &mut rng.fork(&format!("leaf{}.{}", k, n)), p, ang, len, ctx.cfg.leaf_overlap, st, d / total)
            } else {
                None
            };
            if let Some(f) = painted {
                ctx.leaves.push(f);
            } else {
                let q = g::polar(p, ang, 7.0);
                let room = s.room(q[0], q[1]);
                if room >= 7.0 && rng.chance(0.45) {
                    stemlet(s, rng, ctx, p, ang, (room * 2.2).min(26.0), side);
                } else if room >= 4.0 {
                    let size = (room * 1.6).min(ctx.cfg.leaflet_size * dist::sample(rng, &sizes.leaflet));
                    ctx.leaflets.push([p[0], p[1], ang, size]);
                    let c = g::polar(p, ang, size * 0.5);
                    s.disc(c[0], c[1], size * 0.3, DISC_K);
                }
            }
            d += rng.rangev(sp) * (1.7 - 0.7 * dn);
        }
    }
}

pub fn tips(s: &mut Space, rng: &mut Rng, ctx: &mut Ctx) {
    let strands = ctx.strands.clone();
    for (ki, st) in strands.iter().enumerate() {
        let tip = st.borrow().tip;
        if let Some((p, a)) = tip {
            let mut r = rng.fork(&format!("tip{}", ki + 1));
            let len = (ctx.mw * 0.7).min(110.0) * ctx.cfg.leaf_size * dist::sample(&mut r, &tune().sizes.leaf);
            let painted = if len >= 20.0 && ctx.cfg.leaves > 0.0 {
                grow_leaf(s, &mut r, p, a, len, ctx.cfg.leaf_overlap, st, 1.0)
            } else {
                None
            };
            if let Some(f) = painted {
                ctx.leaves.push(f);
            } else {
                let mut tmp = Batch::default();
                let l = r.range(14.0, 22.0);
                let dir = if r.chance(0.5) { 1.0 } else { -1.0 };
                sprig::cyme(&mut tmp, &mut r, p, a, l, dir);
                if !fill::fits(s, &tmp) {
                    tmp = Batch::default();
                    tmp.bud(p[0], p[1], a, 4.0);
                }
                if fill::fits(s, &tmp) {
                    ctx.sprays.merge(tmp);
                    s.disc(p[0], p[1], 8.0, DISC_K);
                }
            }
        }
    }
}

#[derive(Clone)]
struct ReachPt {
    x: f64,
    y: f64,
    tangent: f64,
    stem: Option<StemRef>,
    t: f64,
}

struct ReachMap {
    pts: Vec<ReachPt>,
    sd: Vec<f64>,
    keys: Vec<i64>,
}

impl ReachMap {
    fn new(s: &Space) -> ReachMap {
        ReachMap { pts: Vec::new(), sd: vec![f64::NAN; (s.nx * s.ny) as usize], keys: Vec::new() }
    }

    fn add(&mut self, s: &Space, pts: &[Pt], stem: Option<&StemRef>, step: usize) {
        let (c, nx, ny) = (s.c, s.nx, s.ny);
        let n = pts.len();
        let mut i = 2;
        while i + 1 <= n {
            let p = pts[i - 1];
            self.pts.push(ReachPt { x: p[0], y: p[1], tangent: g::angle(pts[i - 2], pts[i]), stem: stem.cloned(), t: (i - 1) as f64 / (n - 1) as f64 });
            let (i0, i1) = (((p[0] - REACH) / c).floor().max(0.0) as i64, ((p[0] + REACH) / c).floor().min((nx - 1) as f64) as i64);
            let (j0, j1) = (((p[1] - REACH) / c).floor().max(0.0) as i64, ((p[1] + REACH) / c).floor().min((ny - 1) as f64) as i64);
            for j in j0..=j1 {
                let dy = (j as f64 + 0.5) * c - p[1];
                for ii in i0..=i1 {
                    let dx = (ii as f64 + 0.5) * c - p[0];
                    let k = j * nx + ii;
                    let d = (dx * dx + dy * dy).sqrt();
                    let old = self.sd[k as usize];
                    if old.is_nan() {
                        self.keys.push(k);
                        self.sd[k as usize] = d;
                    } else if d < old {
                        self.sd[k as usize] = d;
                    }
                }
            }
            i += step;
        }
    }

    fn nearest(&self, x: f64, y: f64) -> (ReachPt, f64) {
        let (mut best, mut bd) = (0, f64::INFINITY);
        for (i, q) in self.pts.iter().enumerate() {
            let d = (q.x - x) * (q.x - x) + (q.y - y) * (q.y - y);
            if d < bd {
                best = i;
                bd = d;
            }
        }
        (self.pts[best].clone(), bd.sqrt())
    }
}

#[derive(Clone)]
struct Motif {
    kind: &'static str,
    w: f64,
    cap: Option<f64>,
    rmin: f64,
    rmax: f64,
    below: Option<f64>,
    spray: bool,
}

fn motifs(mw: f64, cfg: &Compose, rng: &mut Rng, mx: Option<&Weights>) -> Vec<Motif> {
    let kr = rng.rangev(cfg.features);
    let m = |kind, w, cap, rmin, rmax, below, spray| Motif { kind, w, cap, rmin, rmax, below, spray };
    let all = vec![
        m("bloom", 3.0, Some(cfg.small_blooms), 9.0, mw * 0.13 * kr, None, false),
        m("soft", 1.5, Some(2.0), 10.0, mw * 0.12 * kr, None, false),
        m("lily", 1.2, Some(1.0), 16.0, mw * 0.21 * kr, None, false),
        m("ghost", 1.0, Some(1.0), 16.0, mw * 0.22 * kr, Some(0.4), false),
        m("umbel", 1.6, Some(3.0), 12.0, mw * 0.19 * kr, None, false),
        m("cyme", 3.0, None, 7.0, 22.0, None, true),
        m("gyps", 4.0, None, 5.0, 30.0, None, true),
    ];
    let mut out = Vec::new();
    for mut x in all {
        let w = mix::weight(mx, x.kind);
        if w == 1.0 {
            out.push(x);
        } else if w > 0.0 {
            x.w *= w;
            x.cap = x.cap.map(|c| ((c * w + 0.5).floor()).max(1.0));
            out.push(x);
        }
    }
    out
}

fn bloom_stalk(s: &Space, rp: &ReachPt, sd: f64, c: Pt, r: f64) -> Option<Vec<Pt>> {
    let sp = [rp.x, rp.y];
    let chord = g::angle(sp, c);
    let dir0 = g::depart(sp, rp.tangent, c, 0.35);
    let base = g::polar(sp, chord, (sd - r * 0.45).max(3.0));
    let stalk = g::link(sp, dir0, base, chord + g::turn(chord, dir0) * 0.6, 0.5, 0);
    if open_path(s, &stalk) {
        Some(stalk)
    } else {
        None
    }
}

pub fn holes(s: &mut Space, rng: &mut Rng, ctx: &mut Ctx) {
    let mut reach = ReachMap::new(s);
    for st in ctx.stems.clone() {
        let pts = st.borrow().pts.clone();
        reach.add(s, &pts, Some(&st), 3);
    }
    let cfg = ctx.cfg.clone();
    let kinds = motifs(ctx.mw, &cfg, &mut rng.fork("motifs"), ctx.mix);
    let mut sprays = 0.0;
    let mut rejected = vec![false; (s.nx * s.ny) as usize];
    let (nx, c) = (s.nx, s.c);
    let dens = ctx.dens.clone();
    let floor_at = |x: f64, y: f64| cfg.hole_min * (1.0 + 1.4 * (1.0 - dens(x, y)));
    for iter in 1..=400i64 {
        let (mut best, mut bi) = (-1.0, None);
        for &k in &reach.keys {
            let ku = k as usize;
            let fk = s.f[ku];
            if fk * 1.15 > best && reach.sd[ku] <= REACH && !rejected[ku] {
                let v = fk * jitter(k, iter);
                if v > best {
                    let (x, y) = (((k % nx) as f64 + 0.5) * c, ((k / nx) as f64 + 0.5) * c);
                    if fk >= floor_at(x, y) {
                        best = v;
                        bi = Some(k);
                    }
                }
            }
        }
        let bi = match bi {
            Some(b) => b,
            None => break,
        };
        let (x, y) = (((bi % nx) as f64 + 0.5) * c, ((bi / nx) as f64 + 0.5) * c);
        let room = s.f[bi as usize];
        let fy = (y - ctx.ylo) / ctx.span;
        let mut elig: Vec<(&Motif, f64)> = Vec::new();
        let mut wsum = 0.0;
        for m in &kinds {
            let n = *ctx.counts.get(m.kind).unwrap_or(&0) as f64;
            let ok = m.rmin <= room * 1.05 && m.cap.map_or(true, |cp| n < cp) && m.below.map_or(true, |b| fy < b) && (!m.spray || sprays < cfg.sprays);
            if ok {
                let wt = m.w / (1.0 + n / m.cap.unwrap_or(8.0));
                elig.push((m, wt));
                wsum += wt;
            }
        }
        let mut placed = false;
        if !elig.is_empty() {
            let mut pick = elig[elig.len() - 1].0;
            let mut acc = rng.next() * wsum;
            for e in &elig {
                acc -= e.1;
                if acc <= 0.0 {
                    pick = e.0;
                    break;
                }
            }
            let r = pick.rmax.min(room * 0.95);
            let (sp, sd) = reach.nearest(x, y);
            let mut fr = rng.fork(&format!("hole{}", iter));
            if pick.spray || pick.kind == "umbel" {
                let tries = [(1.0, fr.jit(0.3)), (0.7, 0.0)];
                for t in tries {
                    let mut tmp = Batch::default();
                    let stalk = fill::grow(&mut tmp, &mut fr, pick.kind, [sp.x, sp.y], sp.tangent, sd, x, y, r, t.0, t.1);
                    if fill::fits(s, &tmp) {
                        ctx.sprays.merge(tmp);
                        reach.add(s, &stalk, None, 2);
                        s.disc(x, y, r * t.0, DISC_K);
                        placed = true;
                        break;
                    }
                }
            } else if let Some(stalk) = bloom_stalk(s, &sp, sd, [x, y], r) {
                if let Some(st) = &sp.stem {
                    if st.borrow().cum.is_some() {
                        plant::fork(&mut st.borrow_mut(), sp.t, 0.7, 0.82);
                    }
                }
                ctx.stem_batch.stem(&stalk, 0.7, 0.4);
                reach.add(s, &stalk, None, 2);
                s.disc(x, y, r, DISC_K);
                placed = true;
                if pick.kind == "lily" {
                    let facing = g::angle(stalk[stalk.len() - 2], stalk[stalk.len() - 1]);
                    ctx.blooms.push(Box::new(move |pt: &mut Paint| {
                        flower::tilted(pt, &mut fr, x, y, r, facing, 1.0, &TiltedOpts { kind: Some("lance"), petals: Some(6), width: Some(0.36), ..Default::default() });
                    }));
                } else if pick.kind == "ghost" {
                    ctx.back.push(Box::new(move |pt: &mut Paint| {
                        flower::bloom(pt, &mut fr, x, y, r, &BloomOpts { kind: Some("ghost"), look: Some("ghost"), width: Some(0.55), ..Default::default() });
                    }));
                } else {
                    let style = if pick.kind == "soft" { "soft" } else { flower::pick_style(&mut fr, &["fan", "round"]) };
                    ctx.blooms.push(Box::new(move |pt: &mut Paint| {
                        flower::bloom(pt, &mut fr, x, y, r * 0.8, &BloomOpts { style: Some(style), ..Default::default() });
                    }));
                }
            }
            if placed {
                *ctx.counts.entry(pick.kind).or_insert(0) += 1;
                if pick.spray || pick.kind == "umbel" {
                    sprays += 1.0;
                }
            }
        }
        if !placed {
            rejected[bi as usize] = true;
        }
    }
}

pub fn details(s: &mut Space, rng: &mut Rng, ctx: &mut Ctx) {
    let mut reach = ReachMap::new(s);
    for st in ctx.stems.clone() {
        let pts = st.borrow().pts.clone();
        reach.add(s, &pts, Some(&st), 4);
    }
    let (nx, c) = (s.nx, s.c);
    let mut rejected = vec![false; (s.nx * s.ny) as usize];
    let mut made = 0.0;
    for iter in 1..=300i64 {
        if made >= ctx.cfg.sprays {
            break;
        }
        let (mut best, mut bi) = (-1.0, None);
        for &k in &reach.keys {
            let ku = k as usize;
            if s.f[ku] > best && reach.sd[ku] < 22.0 && !rejected[ku] {
                best = s.f[ku];
                bi = Some(k);
            }
        }
        let bi = match bi {
            Some(b) if best >= 3.5 => b,
            _ => break,
        };
        let (x, y) = (((bi % nx) as f64 + 0.5) * c, ((bi / nx) as f64 + 0.5) * c);
        rejected[bi as usize] = true;
        if rng.next() < (ctx.dens)(x, y) {
            let (sp, sd) = reach.nearest(x, y);
            let mut fr = rng.fork(&format!("detail{}", iter));
            let kind = if best >= 6.0 {
                fr.weighted(&[("gyps", 3.0), ("bud", 2.0), ("cyme", 1.0)]).unwrap()
            } else {
                fr.weighted(&[("gyps", 2.0), ("floret", 2.0)]).unwrap()
            };
            let mut tmp = Batch::default();
            fill::grow(&mut tmp, &mut fr, kind, [sp.x, sp.y], sp.tangent, sd, x, y, best, 1.0, 0.0);
            if fill::fits(s, &tmp) {
                ctx.sprays.merge(tmp);
                s.disc(x, y, best, DISC_K);
                made += 1.0;
            }
        }
    }
    for st in ctx.strands.clone() {
        let pts = st.borrow().pts.clone();
        let mut i = 12;
        while i + 12 <= pts.len() {
            let p = pts[i - 1];
            if rng.next() < ctx.cfg.curls * (0.04 + 0.1 * (ctx.dens)(p[0], p[1])) {
                let side = if rng.chance(0.5) { 1.0 } else { -1.0 };
                let start = g::angle(pts[i - 2], pts[i]) + side * rng.range(0.25, 0.45);
                let curve: Vec<Pt> = if rng.chance(0.45) {
                    let l = rng.range(55.0, 90.0);
                    let v = sprig::flourish(p, start, l, side);
                    let n = (v.len() as f64 * 0.8).floor() as usize;
                    v[..n].to_vec()
                } else {
                    let l = rng.range(30.0, 55.0);
                    let v = sprig::tendril(p, start, l, side);
                    let n = (v.len() as f64 * 0.7).floor() as usize;
                    v[..n].to_vec()
                };
                let n = curve.len();
                if n >= 4 && open_path(s, &curve) {
                    let (mid, tip) = (curve[n / 2 - 1], curve[n - 1]);
                    if s.room(mid[0], mid[1]) >= 3.0 && s.room(tip[0], tip[1]) >= 5.0 {
                        let tip_a = g::angle(curve[n - 2], tip);
                        let size = ctx.cfg.leaflet_size * dist::sample(rng, &tune().sizes.leaflet);
                        ctx.sprays.line(&curve, 2);
                        ctx.leaflets.push([tip[0], tip[1], tip_a, size.min(s.room(tip[0], tip[1]) * 1.8)]);
                        s.disc(mid[0], mid[1], 3.0, 1.0);
                        s.disc(tip[0], tip[1], 4.0, 1.0);
                    }
                }
            }
            i += 14;
        }
    }
}
