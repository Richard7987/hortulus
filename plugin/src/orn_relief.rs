//! Gold in relief, shaded by which way its surface faces, as plain fills.

use crate::color::Rgb;
use crate::geom::{self as g, Cmd, Pt};
use crate::paint::{Paint, NONE};
use std::f64::consts::PI;

const LEVELS: usize = 28;
const TILT: f64 = 1.32;
const LX: f64 = -0.6;
const LY: f64 = 0.8;

const RAMP: [(f64, Rgb); 6] = [
    (0.00, [0.240, 0.130, 0.050]),
    (0.20, [0.420, 0.270, 0.100]),
    (0.45, [0.680, 0.500, 0.220]),
    (0.62, [0.840, 0.670, 0.330]),
    (0.80, [0.950, 0.840, 0.520]),
    (1.00, [1.000, 0.970, 0.840]),
];

pub fn color(v: f64, shade: bool) -> Rgb {
    let v = g::clamp(v, 0.0, 1.0);
    let mut c = RAMP[5].1;
    for i in 1..RAMP.len() {
        if v <= RAMP[i].0 {
            let (a, b) = (RAMP[i - 1], RAMP[i]);
            let t = (v - a.0) / (b.0 - a.0);
            c = [a.1[0] + (b.1[0] - a.1[0]) * t, a.1[1] + (b.1[1] - a.1[1]) * t, a.1[2] + (b.1[2] - a.1[2]) * t];
            break;
        }
    }
    if shade {
        c = [c[0] * 0.9, c[1] * 0.68, c[2] * 0.6];
    }
    c
}

fn mirror(nx: f64, ny: f64, th: f64) -> f64 {
    let (st, ct) = (th.sin(), th.cos());
    let (rx, ry) = (2.0 * ct * st * nx, 2.0 * ct * st * ny);
    let u = rx * LX + ry * LY;
    let mut v = 0.14 + 0.22 * g::smoothstep(-0.3, -1.0, u);
    v += (0.8 + 0.2 * g::smoothstep(0.25, 0.95, u) - v) * g::smoothstep(-0.02, 0.06, u);
    v += 0.22 * (-((u - 0.08) / 0.05).powf(2.0)).exp();
    let front = 0.52;
    front + (v - front) * g::smoothstep(0.05, 0.6, (rx * rx + ry * ry).sqrt())
}

fn flatten(cmds: &[Cmd], pt: Option<&Paint>, least: usize) -> Vec<Vec<Pt>> {
    let mut polys: Vec<Vec<Pt>> = Vec::new();
    let mut cur: Option<Vec<Pt>> = None;
    let put = |cur: &mut Option<Vec<Pt>>, x: f64, y: f64| {
        let (x, y) = match pt {
            Some(p) => p.pj(x, y),
            None => (x, y),
        };
        let c = cur.as_mut().unwrap();
        let keep = match c.last() {
            None => true,
            Some(l) => (l[0] - x).abs() + (l[1] - y).abs() > 0.45,
        };
        if keep {
            c.push([x, y]);
        }
    };
    let finish = |cur: &mut Option<Vec<Pt>>, polys: &mut Vec<Vec<Pt>>| {
        if let Some(c) = cur.take() {
            if c.len() >= least {
                polys.push(c);
            }
        }
    };
    let (mut px, mut py) = (0.0, 0.0);
    for c in cmds {
        match *c {
            Cmd::M(x, y) => {
                finish(&mut cur, &mut polys);
                cur = Some(Vec::new());
                put(&mut cur, x, y);
                px = x;
                py = y;
            }
            Cmd::L(x, y) => {
                put(&mut cur, x, y);
                px = x;
                py = y;
            }
            Cmd::C(a, b, c2, d, e, f) => {
                let len = g::dist([px, py], [a, b]) + g::dist([a, b], [c2, d]) + g::dist([c2, d], [e, f]);
                let n = g::clamp((len / 0.9).ceil(), 1.0, 24.0) as usize;
                let pts = g::bezier([px, py], [a, b], [c2, d], [e, f], n);
                for q in &pts[1..] {
                    put(&mut cur, q[0], q[1]);
                }
                px = e;
                py = f;
            }
            Cmd::O(x, y, r) => {
                finish(&mut cur, &mut polys);
                cur = Some(Vec::new());
                let n = g::clamp((2.0 * PI * r / 0.9).ceil(), 8.0, 48.0) as usize;
                for k in 0..n {
                    let a = 2.0 * PI * k as f64 / n as f64;
                    put(&mut cur, x + r * a.cos(), y + r * a.sin());
                }
                finish(&mut cur, &mut polys);
            }
            Cmd::Z => finish(&mut cur, &mut polys),
        }
    }
    finish(&mut cur, &mut polys);
    for p in polys.iter_mut() {
        let n = p.len();
        if n > 3 && (p[0][0] - p[n - 1][0]).abs() + (p[0][1] - p[n - 1][1]).abs() < 0.45 {
            p.pop();
        }
    }
    polys
}

fn bevel(p: &[Pt], buckets: &mut [Vec<Vec<Pt>>], depth_cap: f64, tilt: f64) {
    let n = p.len();
    let a = g::area(p);
    let mut per = 0.0;
    for i in 0..n {
        per += g::dist(p[i], p[(i + 1) % n]);
    }
    if a.abs() < 0.01 || per < 0.1 {
        return;
    }
    let s = if a > 0.0 { 1.0 } else { -1.0 };
    let t = 2.0 * a.abs() / per;
    let depth = (0.45 * t).min(0.5 + 0.16 * t).min(depth_cap);
    let rings = g::clamp((depth / 0.35).ceil(), 2.0, 5.0) as usize;
    let mut en = Vec::with_capacity(n);
    for i in 0..n {
        let (a, b) = (p[i], p[(i + 1) % n]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = (dx * dx + dy * dy).sqrt();
        en.push(if l > 0.0 { [s * dy / l, -s * dx / l] } else { [0.0, 0.0] });
    }
    let mut off = Vec::with_capacity(n);
    for i in 0..n {
        let (n1, n2) = (en[(i + n - 1) % n], en[i]);
        let (mut mx, mut my) = (n1[0] + n2[0], n1[1] + n2[1]);
        let mut l = (mx * mx + my * my).sqrt();
        if l < 1e-6 {
            mx = n2[0];
            my = n2[1];
            l = 1.0;
        }
        mx /= l;
        my /= l;
        let k = 1.0 / (mx * n2[0] + my * n2[1]).max(0.4);
        off.push([mx * k, my * k]);
    }
    let mut outer: Vec<Pt> = p.to_vec();
    for r in 1..=rings {
        let d = depth * r as f64 / rings as f64;
        let inner: Vec<Pt> = (0..n).map(|i| [p[i][0] - off[i][0] * d, p[i][1] - off[i][1] * d]).collect();
        let th = tilt * (1.0 - (r as f64 - 0.5) / rings as f64).powf(1.4);
        let level: Vec<usize> = (0..n)
            .map(|i| (g::clamp(mirror(en[i][0], en[i][1], th), 0.0, 1.0) * (LEVELS - 1) as f64 + 0.5).floor() as usize)
            .collect();
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j + 1 < n && level[j + 1] == level[i] {
                j += 1;
            }
            let mut strip = Vec::with_capacity(2 * (j - i + 2));
            for k in i..=j + 1 {
                strip.push(outer[k % n]);
            }
            for k in (i..=j + 1).rev() {
                strip.push(inner[k % n]);
            }
            if g::area(&strip) < 0.0 {
                strip.reverse();
            }
            buckets[level[i]].push(strip);
            i = j + 1;
        }
        outer = inner;
    }
}

pub struct PaintOpts {
    pub shade: bool,
    pub shadow: f64,
    pub key: f64,
    pub depth: f64,
    pub tilt: f64,
}

impl Default for PaintOpts {
    fn default() -> Self {
        PaintOpts { shade: false, shadow: 0.0, key: 0.14, depth: 4.0, tilt: TILT }
    }
}

pub fn paint_polys(pt: &mut Paint, polys: &[Vec<Pt>], o: &PaintOpts) {
    if polys.is_empty() {
        return;
    }
    let (proj, frame) = (pt.projection.take(), pt.frame_fn.take());
    let all: Vec<Vec<Cmd>> = polys.iter().map(|p| g::polyline(p, true)).collect();
    if o.shadow > 0.0 {
        let sh: Vec<Vec<Cmd>> = polys
            .iter()
            .map(|p| {
                let q: Vec<Pt> = p.iter().map(|v| [v[0] - LX * o.shadow, v[1] - LY * o.shadow]).collect();
                g::polyline(&q, true)
            })
            .collect();
        pt.fill_many(&sh, [0.45, 0.28, 0.14], 0.35, NONE);
    }
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for p in polys {
        for v in p {
            b = [b[0].min(v[0]), b[1].min(v[1]), b[2].max(v[0]), b[3].max(v[1])];
        }
    }
    let stops = [
        (0.0, color(0.62, o.shade)),
        (0.3, color(0.7, o.shade)),
        (0.62, color(0.42, o.shade)),
        (0.85, color(0.5, o.shade)),
        (1.0, color(0.46, o.shade)),
    ];
    let (cx, cy) = ((b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0);
    let half = (LX.abs() * (b[2] - b[0]) + LY.abs() * (b[3] - b[1])) / 2.0 + 0.5;
    let clip: Vec<Cmd> = all.iter().flatten().copied().collect();
    pt.clip_begin(&clip);
    pt.shade(&stops, [cx + LX * half, cy + LY * half], [cx - LX * half, cy - LY * half], (b[2] - b[0]) + (b[3] - b[1]) + 2.0);
    pt.clip_end();
    let mut buckets: Vec<Vec<Vec<Pt>>> = vec![Vec::new(); LEVELS];
    for p in polys {
        bevel(p, &mut buckets, o.depth, o.tilt);
    }
    for (q, quads) in buckets.iter().enumerate() {
        if !quads.is_empty() {
            let cmds: Vec<Vec<Cmd>> = quads.iter().map(|s| g::polyline(s, true)).collect();
            pt.fill_many(&cmds, color(q as f64 / (LEVELS - 1) as f64, o.shade), 1.0, NONE);
        }
    }
    if o.key > 0.0 {
        pt.stroke_many(&all, [0.30, 0.18, 0.07], o.key, 0.7, NONE);
    }
    pt.projection = proj;
    pt.frame_fn = frame;
}

pub fn paint(pt: &mut Paint, paths: &[Vec<Cmd>], o: &PaintOpts) {
    let mut polys = Vec::new();
    {
        let p: &Paint = pt;
        let view = if p.has_pj() { Some(p) } else { None };
        for cmds in paths {
            polys.extend(flatten(cmds, view, 3));
        }
    }
    paint_polys(pt, &polys, o);
}

pub fn ribbons(pt: &Paint, paths: &[Vec<Cmd>], w: f64) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    for cmds in paths {
        let closed = cmds.iter().any(|c| matches!(c, Cmd::Z | Cmd::O(..)));
        for mut poly in flatten(cmds, None, 2) {
            if closed {
                poly.push(poly[0]);
            }
            if poly.len() >= 2 {
                let mut rib = g::ribbon(&poly, |_| w / 2.0);
                if pt.has_pj() {
                    for v in rib.iter_mut() {
                        let (x, y) = pt.pj(v[0], v[1]);
                        *v = [x, y];
                    }
                }
                out.push(rib);
            }
        }
    }
    out
}
