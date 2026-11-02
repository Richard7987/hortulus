//! Page layouts as frames: each plant grows in its own local page, mapped to the paper.

use crate::compose::{Bx, Spec};
use crate::geom as g;
use crate::lua::sort;
use crate::orn_title::{self, Item};
use crate::tune::tune;

pub type Matrix = [f64; 6];

pub fn apply(m: &Matrix, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn invert(m: &Matrix) -> Matrix {
    let det = m[0] * m[3] - m[1] * m[2];
    let (a, b, c, d) = (m[3] / det, -m[1] / det, -m[2] / det, m[0] / det);
    [a, b, c, d, -(a * m[4] + c * m[5]), -(b * m[4] + d * m[5])]
}

fn map_box(m: &Matrix, b: Bx) -> Bx {
    let (ax, ay) = apply(m, b.x0, b.y0);
    let (bx, by) = apply(m, b.x1, b.y1);
    Bx { x0: ax.min(bx), y0: ay.min(by), x1: ax.max(bx), y1: ay.max(by) }
}

fn clip(b: Bx, w: f64, h: f64) -> Option<Bx> {
    let c = Bx { x0: b.x0.max(0.0), y0: b.y0.max(0.0), x1: b.x1.min(w), y1: b.y1.min(h) };
    if c.x1 <= c.x0 || c.y1 <= c.y0 {
        None
    } else {
        Some(c)
    }
}

pub struct Frame {
    pub spec: Spec,
    pub map: Option<Matrix>,
}

#[derive(Default)]
struct FrameOpts {
    block: Option<Bx>,
    header: bool,
    frame_only: bool,
    seed: Option<f64>,
}

fn frame(spec: &Spec, m: Matrix, w: f64, h: f64, o: FrameOpts) -> Result<Frame, String> {
    let inv = invert(&m);
    let text = clip(map_box(&inv, o.block.unwrap_or(spec.text)), w, h).ok_or("layout: the text block lies outside its frame")?;
    let lines = spec.lines.as_ref().map(|ls| ls.iter().filter_map(|l| clip(map_box(&inv, *l), w, h)).collect());
    let mut s = spec.clone();
    s.width = w;
    s.height = h;
    s.text = text;
    s.lines = lines;
    s.header_y = if o.header { spec.header_y } else { None };
    s.closed = Vec::new();
    s.frame_only = o.frame_only;
    s.layout = None;
    if let Some(seed) = o.seed {
        s.seed = seed;
    }
    Ok(Frame { spec: s, map: Some(m) })
}

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn seed_of(spec: &Spec, k: f64) -> f64 {
    spec.seed * 31.0 + k * 7907.0
}

fn hero_radius(spec: &Spec, x0: f64) -> f64 {
    let mw = x0 - 9.0 - spec.edge;
    g::clamp(mw * tune().compose.hero, 28.0, 80.0)
}

pub fn of(spec: &Spec) -> Result<(Vec<Frame>, Option<Vec<Item>>), String> {
    let mut layout = spec.layout.clone().unwrap_or_else(|| "left".to_string());
    let (w, h) = (spec.width, spec.height);
    if layout == "split" {
        layout = if spec.page.rem_euclid(2) == 1 { "right".to_string() } else { "left".to_string() };
    }
    match layout.as_str() {
        "left" => Ok((vec![Frame { spec: spec.clone(), map: None }], None)),
        "right" => Ok((vec![frame(spec, [-1.0, 0.0, 0.0, 1.0, w, 0.0], w, h, FrameOpts { header: true, ..Default::default() })?], None)),
        "centered" => {
            let half = w / 2.0;
            Ok((
                vec![
                    frame(spec, IDENTITY, half, h, FrameOpts::default())?,
                    frame(spec, [-1.0, 0.0, 0.0, 1.0, w, 0.0], half, h, FrameOpts { seed: Some(seed_of(spec, 2.0)), ..Default::default() })?,
                ],
                None,
            ))
        }
        "surround" => {
            let mut a = frame(spec, IDENTITY, w, h, FrameOpts { frame_only: true, ..Default::default() })?;
            let mut b = frame(spec, [-1.0, 0.0, 0.0, -1.0, w, h], w, h, FrameOpts { seed: Some(seed_of(spec, 2.0)), frame_only: true, ..Default::default() })?;
            for f in [&mut a, &mut b] {
                let ft = f.spec.text;
                f.spec.closed = vec![[ft.x1, ft.y0 - 6.0, w, h], [0.0, ft.y1 + 4.0, w, h]];
            }
            Ok((vec![a, b], None))
        }
        "title" => {
            let bk = spec.title.unwrap_or_else(|| top_block(spec));
            let depth = h.min(h - bk.y0 + 60.0);
            let r = hero_radius(spec, h - bk.y1);
            let crest = orn_title::crest_size(spec, r);
            let half = w / 2.0 - r * 0.47 - crest * 0.45;
            let mut fs = vec![
                frame(spec, [0.0, -1.0, 1.0, 0.0, 0.0, h], depth, half, FrameOpts { block: Some(bk), frame_only: true, ..Default::default() })?,
                frame(spec, [0.0, -1.0, -1.0, 0.0, w, h], depth, half, FrameOpts { block: Some(bk), seed: Some(seed_of(spec, 2.0)), frame_only: true, ..Default::default() })?,
            ];
            let (plan, keep) = orn_title::plan(spec, bk, crest);
            for f in fs.iter_mut() {
                let inv = invert(&f.map.unwrap());
                let mut closed = Vec::new();
                for k in &keep {
                    if let Some(c) = clip(map_box(&inv, Bx { x0: k[0] - 4.0, y0: k[1] - 4.0, x1: k[2] + 4.0, y1: k[3] + 4.0 }), depth, half) {
                        closed.push([c.x0, c.y0, c.x1, c.y1]);
                    }
                }
                f.spec.closed = closed;
            }
            Ok((fs, Some(plan)))
        }
        other => Err(format!("unknown layout {}", other)),
    }
}

fn top_block(spec: &Spec) -> Bx {
    let t = spec.text;
    let mut ls: Vec<Bx> = spec.lines.as_deref().unwrap_or(&[]).iter().copied().filter(|l| l.y1 > t.y0 && l.y0 < t.y1 + 2.0).collect();
    sort(&mut ls, |p, q| p.y1 > q.y1);
    if ls.is_empty() {
        return Bx { x0: t.x0, y0: t.y1 - 80.0, x1: t.x1, y1: t.y1 };
    }
    let mut b = ls[0];
    for i in 1..ls.len() {
        let l = ls[i];
        let hh = ls[i - 1].y1 - ls[i - 1].y0;
        if b.y0 - l.y1 > 2.0 * hh {
            break;
        }
        b.x0 = b.x0.min(l.x0);
        b.x1 = b.x1.max(l.x1);
        b.y0 = b.y0.min(l.y0);
    }
    b
}
