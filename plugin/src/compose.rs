//! Composes one page of flora, in the order the plant grows, painting back to front.

use crate::brush;
use crate::fill;
use crate::flower::{self, BloomOpts};
use crate::frames::{self, Frame};
use crate::geom as g;
use crate::grow::{self, Ctx, Painter};
use crate::layout;
use crate::leaf;
use crate::mix::{self, Dens, Palette, Weights};
use crate::ornament as o;
use crate::ornaments::{self, DrawOpts, Slot};
use crate::orn_title;
use crate::paint::{self, Paint, NONE};
use crate::plant::Stem;
use crate::rng::Rng;
use crate::sprig::Batch;
use crate::tune::tune;
use crate::visitor;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

const HERO_TOP: f64 = 8.0;

#[derive(Clone, Copy, Debug)]
pub struct Bx {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Clone)]
pub struct Spec {
    pub width: f64,
    pub height: f64,
    pub text: Bx,
    pub lines: Option<Vec<Bx>>,
    pub header_y: Option<f64>,
    pub seed: f64,
    pub page: i64,
    pub edge: f64,
    pub layout: Option<String>,
    pub title: Option<Bx>,
    pub frame_only: bool,
    pub closed: Vec<[f64; 4]>,
    pub mix: Option<Weights>,
    pub ornaments: Option<Weights>,
    pub palette: Option<Palette>,
    pub spin: f64,
    pub gilt_vector: bool,
    pub gilt_dpi: Option<f64>,
    pub gilt_env: Option<String>,
    pub gilt_look: Vec<(String, f64)>,
    /// Shorter output for the package; the oracle keeps the reference encoding.
    pub compact: bool,
}

fn plant(spec: &Spec, pt: &mut Paint) {
    let mut rng = Rng::new(spec.seed);
    let w = spec.width;
    let edge = spec.edge;
    let (mut s, m) = layout::region(spec);
    let mw = m.x1 - m.x0;
    let top = spec.header_y.unwrap_or(spec.text.y1);
    let t = tune();

    let cfg = mix::cfg(&t.compose, spec.mix.as_ref());
    let r0 = g::clamp(mw * cfg.hero, 28.0, 80.0);
    let (_, rh) = flower::tilt(0.0, 0.0, r0, t.hero.tilt);
    let hy = (top - r0 * 0.32).min(spec.height - HERO_TOP - rh);
    let hero = [g::clamp(m.x0 + mw * 0.5, m.x0 + r0 * 0.75, m.x1 - r0 * 0.95), hy, r0];
    s.field();
    {
        let (_, rh) = flower::tilt(hero[0], hero[1], r0, t.hero.tilt);
        s.disc(hero[0], hero[1], rh, 0.85);
        for sx in [-1.0, 1.0] {
            s.disc(hero[0] + sx * r0 * 0.45, hero[1], r0 * 0.55, 0.85);
        }
    }

    let gest = Rc::new(layout::gesture(&mut rng.fork("gesture"), &m, hero));
    let gd = gest.clone();
    let dens: Dens = mix::density(Rc::new(move |x, y| gd.density(x, y)), spec.mix.as_ref());
    let (ylo, span) = (m.y0, hero[1] - m.y0);
    let py = gest.quiet.first().copied().unwrap_or(ylo + span * 0.5);
    let (pw, ph) = layout::cartouche_size(mw, 1.0);
    let (strands_raw, root) = layout::strands(&rng.fork("strands"), &m, hero, w, &gest, Some([m.x0, py - ph / 2.0 - 3.0, m.x0 + pw + 5.0, py + ph / 2.0 + 3.0]));
    let mut strands_raw: Vec<Stem> = strands_raw;
    if !spec.frame_only {
        if let Some(e) = layout::excursion(&mut rng.fork("excursion"), spec, root) {
            strands_raw.push(e);
        }
    }
    let first = strands_raw[0].pts[0];
    let runners_raw = layout::runners(&m, first, w, edge);
    crate::plant::fork(&mut strands_raw[0], 0.005, 0.8, 0.82);
    let strands: Vec<Rc<RefCell<Stem>>> = strands_raw.into_iter().map(|x| Rc::new(RefCell::new(x))).collect();
    let runners: Vec<Rc<RefCell<Stem>>> = runners_raw.into_iter().map(|x| Rc::new(RefCell::new(x))).collect();
    let mut stems = Vec::new();
    for list in [&strands, &runners] {
        for st in list {
            s.reserve_corridor(&st.borrow().pts, 2.0);
            stems.push(st.clone());
        }
    }

    let mut orn: Vec<Painter> = Vec::new();
    let mut panel: Option<[f64; 4]> = None;
    let cartouche = ornaments::pick(&mut rng.fork("choose cartouche"), Slot::Cartouche, spec.ornaments.as_ref());
    if let Some(kind) = cartouche {
        for k in [1.0, 0.82, 0.66] {
            let (cw, ch) = layout::cartouche_size(mw, k);
            let cx = m.x0 + cw / 2.0 + 2.0;
            let mut fits = true;
            let mut x = cx - cw / 2.0 - 1.0;
            while x <= cx + cw / 2.0 + 3.0 {
                let mut y = py - ch / 2.0 - 3.0;
                while y <= py + ch / 2.0 + 3.0 {
                    if !s.free(x, y) {
                        fits = false;
                    }
                    y += 2.0;
                }
                x += 2.0;
            }
            if fits {
                s.boxed(cx - cw / 2.0, py - ch / 2.0, cx + cw / 2.0, py + ch / 2.0);
                let mut r = rng.fork("panel");
                orn.push(Box::new(move |p: &mut Paint| ornaments::draw(kind, p, &mut r, [cx - cw / 2.0, py - ch / 2.0, cx + cw / 2.0, py + ch / 2.0], &DrawOpts::default())));
                panel = Some([cx, py, cw, ch]);
                break;
            }
        }
    }
    let mut ctx = Ctx {
        cfg: cfg.clone(),
        mix: spec.mix.as_ref(),
        mw,
        dens: dens.clone(),
        ylo,
        span,
        strands: strands.clone(),
        stems,
        stem_batch: Batch::default(),
        sprays: Batch::default(),
        blooms: Vec::new(),
        back: Vec::new(),
        leaves: Vec::new(),
        leaflets: Vec::new(),
        visitors: Vec::new(),
        counts: HashMap::new(),
    };
    {
        let mut gr = rng.fork("ghost");
        let gw = mix::weight(spec.mix.as_ref(), "ghost");
        if gw > 0.0 && gr.chance(0.85 * gw.min(1.0)) {
            for _ in 0..8 {
                let r = mw * gr.range(0.23, 0.28);
                let x = m.x0 + mw * gr.range(0.35, 0.5);
                let y = m.foot + gr.range(35.0, 75.0);
                let mut ok = true;
                for k in 0..12 {
                    let q = g::polar([x, y], k as f64 * PI / 6.0, r * 0.9);
                    let in_panel = panel.map_or(false, |p| (q[0] - p[0]).abs() < p[2] / 2.0 + 2.0 && (q[1] - p[1]).abs() < p[3] / 2.0 + 2.0);
                    if !s.open_at(q[0], q[1]) || in_panel {
                        ok = false;
                    }
                }
                if ok {
                    s.disc(x, y, r, 0.8);
                    let mut fr = gr.fork("bloom");
                    ctx.back.push(Box::new(move |p: &mut Paint| {
                        flower::bloom(p, &mut fr, x, y, r, &BloomOpts { kind: Some("ghost"), look: Some("ghost"), width: Some(0.55), ..Default::default() });
                    }));
                    ctx.counts.insert("ghost", 1);
                    break;
                }
            }
        }
    }
    let grow_rng = rng.fork("grow");
    grow::node_leaves(&mut s, &mut grow_rng.fork("nodes"), &mut ctx);
    grow::tips(&mut s, &mut grow_rng.fork("tips"), &mut ctx);
    visitor::place(&mut s, &rng.fork("visitors"), &mut ctx);

    if let Some(hdr) = spec.header_y {
        let (hx, hy) = (spec.text.x0 - 6.0, hdr - 3.0);
        let mark = ornaments::pick(&mut rng.fork("choose header"), Slot::Header, spec.ornaments.as_ref());
        let tx1 = spec.text.x1;
        let mut r = rng.fork("header");
        orn.push(Box::new(move |p: &mut Paint| {
            p.stroke(&g::polyline(&[[hx + 4.0, hy], [tx1, hy]], false), paint::INK, 0.4, 0.9, NONE);
            if let Some(kind) = mark {
                ornaments::draw(kind, p, &mut r, [hx - 13.0, hy - 13.0, hx + 13.0, hy + 13.0], &DrawOpts::default());
            }
        }));
    }
    let pendant = ornaments::pick(&mut rng.fork("choose pendant"), Slot::Pendant, spec.ornaments.as_ref());
    if let Some(kind) = pendant {
        let mut r = rng.fork("pendant");
        let pw = g::clamp(mw * r.range(0.15, 0.19), 14.0, 30.0);
        let ph = pw * r.range(1.3, 1.5);
        let x = m.x0 + pw / 2.0 + r.range(3.0, 10.0);
        let mut y = m.y0 + ph / 2.0 + 1.0;
        while y <= m.y0 + 90.0 {
            let mut clear = s.room(x, y) >= pw * 0.3;
            for q in [[-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0], [1.0, 1.0]] {
                if !s.open_at(x + q[0] * pw / 2.0, y + q[1] * ph / 2.0) {
                    clear = false;
                }
            }
            if clear {
                s.boxed(x - pw / 2.0, y - ph / 2.0, x + pw / 2.0, y + ph / 2.0);
                let mut cord = 0.0;
                while cord < 40.0 && s.free(x, y + ph / 2.0 + cord + 2.0) {
                    cord += 2.0;
                }
                if cord >= 6.0 {
                    s.boxed(x - 1.0, y + ph / 2.0, x + 1.0, y + ph / 2.0 + cord);
                } else {
                    cord = 0.0;
                }
                let py2 = y;
                orn.push(Box::new(move |p: &mut Paint| {
                    ornaments::draw(kind, p, &mut r, [x - pw / 2.0, py2 - ph / 2.0, x + pw / 2.0, py2 + ph / 2.0], &DrawOpts { cord, flip: None });
                }));
                break;
            }
            y += 3.0;
        }
    }

    let px = m.x0 + 2.5;
    let (mut runs, mut run): (Vec<[f64; 2]>, Option<[f64; 2]>) = (Vec::new(), None);
    let (_, hero_rh) = flower::tilt(0.0, 0.0, r0, t.hero.tilt);
    let mut y = hero[1] - hero_rh - 6.0;
    while y >= m.y0 + 6.0 {
        if s.free(px, y) && s.free(px - 1.5, y) && s.free(px + 1.5, y) {
            match &mut run {
                Some(r) => r[1] = y,
                None => run = Some([y, y]),
            }
        } else {
            if let Some(r) = run {
                if r[0] - r[1] >= 28.0 {
                    runs.push(r);
                }
            }
            run = None;
        }
        y -= 2.0;
    }
    if let Some(r) = run {
        if r[0] - r[1] >= 28.0 {
            runs.push(r);
        }
    }
    for (i, rn) in runs.iter().enumerate() {
        let rn = *rn;
        let mut y = rn[0];
        while y >= rn[1] {
            s.reserve(px, y, 5.0);
            y -= 5.0;
        }
        let mut r = rng.fork(&format!("chain{}", i + 1));
        let foot = panel.map_or(false, |p| (px - p[0]).abs() < p[2] / 2.0 && rn[1] - (p[1] + p[3] / 2.0) < 8.0 && rn[1] > p[1]);
        orn.push(Box::new(move |p: &mut Paint| o::chain(p, &mut r, px, rn[0], rn[1], foot)));
    }

    grow::holes(&mut s, &mut grow_rng.fork("holes"), &mut ctx);
    grow::details(&mut s, &mut grow_rng.fork("details"), &mut ctx);
    if !cfg.no_leaflets {
        for st in &strands {
            let pts = st.borrow().pts.clone();
            for it in crate::sprig::leaflet_items(&mut rng, &pts, cfg.leaflet_spacing, cfg.leaflet_size) {
                if s.room(it[0], it[1]) >= it[3] * 0.3 && rng.next() < dens(it[0], it[1]) {
                    ctx.leaflets.push(it);
                }
            }
        }
    }
    let sparkles = fill::sparkles(&mut rng.fork("sparkles"), &mut s, 40, spec.ornaments.as_ref());
    let midfoot = [w * 0.66, m.y0 + 10.0];

    let mut painted = Vec::new();
    for st in &strands {
        let b = st.borrow();
        if b.cum.is_some() {
            painted.push(st.clone());
        } else {
            ctx.stem_batch.stem_by(&b.pts, &|t| b.width(t));
        }
    }
    for st in &runners {
        let b = st.borrow();
        ctx.stem_batch.stem_by(&b.pts, &|t| b.width(t));
    }

    for f in ctx.back.iter_mut() {
        f(pt);
    }
    let mut sr = rng.fork("stem paint");
    for st in &painted {
        let b = st.borrow();
        brush::stem(pt, &mut sr, &b.pts, &|t| b.width(t), &t.plant);
    }
    ctx.stem_batch.paint(pt, None);
    ctx.sprays.paint(pt, Some(paint::mix(paint::INK, paint::SKY, 0.4)));
    leaf::leaflets(pt, &mut rng.fork("leaflets"), &ctx.leaflets, None, 0.88);
    for f in ctx.leaves.iter_mut() {
        f(pt);
    }
    for f in ctx.blooms.iter_mut() {
        f(pt);
    }
    flower::hero(pt, &mut rng.fork("hero"), hero);
    for f in ctx.visitors.iter_mut() {
        f(pt);
    }
    for f in orn.iter_mut() {
        f(pt);
    }
    o::sparkles(pt, &sparkles);
    o::compass(pt, midfoot[0], midfoot[1], 7.0, 0.9, 0);
    pt.coverage = s.coverage();
}

fn new_painter(spec: &Spec) -> Paint {
    let mut pt = Paint::new(spec.gilt_vector);
    if let Some(d) = spec.gilt_dpi {
        pt.gilt_dpi = d;
    }
    pt.gilt_env = spec.gilt_env.clone();
    pt.gilt_look = spec.gilt_look.clone();
    pt.compact = spec.compact;
    pt
}

pub fn page(spec: &Spec) -> Result<Paint, String> {
    let palette = mix::resolve(spec.palette.as_ref(), spec.seed, spec.spin);
    mix::with_palette(palette, || page_frames(spec))
}

fn page_frames(spec: &Spec) -> Result<Paint, String> {
    let (fs, title): (Vec<Frame>, _) = frames::of(spec)?;
    let mut pt = new_painter(spec);
    if fs.len() == 1 && fs[0].map.is_none() {
        plant(spec, &mut pt);
        return Ok(pt);
    }
    for f in &fs {
        let m = f.map.unwrap();
        pt.frame(Some(Rc::new(move |x, y| frames::apply(&m, x, y))));
        plant(&f.spec, &mut pt);
    }
    pt.frame(None);
    if let Some(plan) = &title {
        orn_title::paint(&mut pt, spec, plan);
    }
    Ok(pt)
}
