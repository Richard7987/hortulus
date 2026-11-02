//! Gilt for the ornaments: computed raster gold, or gold in relief.

use crate::geom::{self as g, Cmd, Pt};
use crate::orn_raster;
use crate::orn_relief::{self, PaintOpts};
use crate::paint::{self, Paint, NONE};

pub fn bbox(paths: &[Vec<Cmd>]) -> [f64; 4] {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut add = |x: f64, y: f64, r: f64| {
        x0 = x0.min(x - r);
        x1 = x1.max(x + r);
        y0 = y0.min(y - r);
        y1 = y1.max(y + r);
    };
    for cmds in paths {
        for c in cmds {
            match *c {
                Cmd::M(x, y) | Cmd::L(x, y) => add(x, y, 0.0),
                Cmd::C(a, b, c2, d, e, f) => {
                    add(a, b, 0.0);
                    add(c2, d, 0.0);
                    add(e, f, 0.0);
                }
                Cmd::O(x, y, r) => add(x, y, r),
                Cmd::Z => {}
            }
        }
    }
    [x0, y0, x1, y1]
}

fn raster(pt: &Paint) -> bool {
    !pt.gilt_vector && !pt.flat_gilt
}

#[derive(Clone, Copy, Default)]
pub struct FillOpts {
    pub bevel: Option<f64>,
    pub key: Option<f64>,
    pub shadow: Option<f64>,
    pub shade: bool,
    pub evenodd: bool,
}

pub fn fill(pt: &mut Paint, paths: &[Vec<Cmd>], shade: bool) {
    fill_o(pt, paths, FillOpts { shade, ..Default::default() });
}

pub fn fill_o(pt: &mut Paint, paths: &[Vec<Cmd>], o: FillOpts) {
    if paths.is_empty() {
        return;
    }
    if pt.flat_gilt {
        pt.fill_many(paths, if o.shade { paint::COPPER } else { paint::GOLD }, 0.95, NONE);
        return;
    }
    let b = bbox(paths);
    let size = (b[2] - b[0]).min(b[3] - b[1]);
    let shadow = o.shadow.unwrap_or(if o.shade { 0.0 } else { g::clamp(size * 0.02, 0.15, 0.6) });
    if raster(pt) {
        let e = o.bevel.unwrap_or(g::clamp(size * 0.03, 0.18, 0.7).min(size * 0.1));
        orn_raster::add(pt, false, paths, orn_raster::AddOpts { bevel: Some(e), shadow, shade: o.shade, width: 0.0, whole: false, evenodd: o.evenodd });
        return;
    }
    orn_relief::paint(pt, paths, &PaintOpts { shade: o.shade, shadow, key: o.key.unwrap_or(0.16f64.min(size * 0.04)), ..Default::default() });
}

pub fn line(pt: &mut Paint, paths: &[Vec<Cmd>], w: f64) {
    if pt.flat_gilt {
        pt.stroke_many(paths, paint::GOLD, w, 0.95, NONE);
        return;
    }
    if raster(pt) {
        orn_raster::add(pt, true, paths, orn_raster::AddOpts { bevel: None, shadow: w * 0.3, shade: false, width: w, whole: false, evenodd: false });
        return;
    }
    let fine = w < 1.0;
    let polys = orn_relief::ribbons(pt, paths, w.max(0.35));
    let mut o = PaintOpts { key: if fine { 0.0 } else { 0.1 }, ..Default::default() };
    if fine {
        o.tilt = 0.75;
    }
    orn_relief::paint_polys(pt, &polys, &o);
}

pub fn dots(pt: &mut Paint, pts: &[Pt], r: f64) {
    if pts.is_empty() {
        return;
    }
    let paths: Vec<Vec<Cmd>> = pts.iter().map(|p| vec![Cmd::O(p[0], p[1], r)]).collect();
    if pt.flat_gilt {
        pt.fill_many(&paths, paint::GOLD, 0.95, NONE);
        return;
    }
    if raster(pt) {
        orn_raster::add(pt, false, &paths, orn_raster::AddOpts { bevel: Some(r), shadow: r * 0.3, shade: false, width: 0.0, whole: true, evenodd: false });
        return;
    }
    orn_relief::paint(pt, &paths, &PaintOpts { key: 0.1f64.min(r * 0.15), ..Default::default() });
}
