//! Plane geometry: points are [x, y] in TeX points.

use crate::lua::{fmod, sort};
use crate::rng::Rng;
use std::f64::consts::PI;

pub type Pt = [f64; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    M(f64, f64),
    L(f64, f64),
    C(f64, f64, f64, f64, f64, f64),
    O(f64, f64, f64),
    Z,
}

#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[inline]
pub fn clamp(x: f64, a: f64, b: f64) -> f64 {
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}

pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp((x - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn dist(p: Pt, q: Pt) -> f64 {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    (dx * dx + dy * dy).sqrt()
}

#[inline]
pub fn angle(p: Pt, q: Pt) -> f64 {
    (q[1] - p[1]).atan2(q[0] - p[0])
}

#[inline]
pub fn polar(p: Pt, ang: f64, r: f64) -> Pt {
    [p[0] + r * ang.cos(), p[1] + r * ang.sin()]
}

#[inline]
pub fn mix(p: Pt, q: Pt, t: f64) -> Pt {
    [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]
}

pub fn transform(pts: &[Pt], ox: f64, oy: f64, ang: f64, sx: f64, sy: f64) -> Vec<Pt> {
    let (c, s) = (ang.cos(), ang.sin());
    pts.iter()
        .map(|p| {
            let (x, y) = (p[0] * sx, p[1] * sy);
            [ox + x * c - y * s, oy + x * s + y * c]
        })
        .collect()
}

pub fn toward(pts: &[Pt], cx: f64, cy: f64, s: f64) -> Vec<Pt> {
    pts.iter().map(|p| [cx + (p[0] - cx) * s, cy + (p[1] - cy) * s]).collect()
}

pub fn bbox(pts: &[Pt]) -> (f64, f64, f64, f64) {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in pts {
        if p[0] < x0 {
            x0 = p[0];
        }
        if p[0] > x1 {
            x1 = p[0];
        }
        if p[1] < y0 {
            y0 = p[1];
        }
        if p[1] > y1 {
            y1 = p[1];
        }
    }
    (x0, y0, x1, y1)
}

pub fn area(pts: &[Pt]) -> f64 {
    let n = pts.len();
    let mut a = 0.0;
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        a = a + p[0] * q[1] - q[0] * p[1];
    }
    a / 2.0
}

pub fn centroid(pts: &[Pt]) -> Pt {
    let (mut x, mut y) = (0.0, 0.0);
    for p in pts {
        x += p[0];
        y += p[1];
    }
    [x / pts.len() as f64, y / pts.len() as f64]
}

/// Samples points inside a polygon, close to uniform by area.
pub struct SpanSampler {
    rows: Vec<(f64, Vec<f64>)>,
    cum: Vec<f64>,
    total: f64,
    h: f64,
}

pub fn span_sampler(pts: &[Pt], h: f64) -> Option<SpanSampler> {
    let (_, y0, _, y1) = bbox(pts);
    let n = pts.len();
    let (mut rows, mut cum, mut total) = (Vec::new(), Vec::new(), 0.0);
    let mut y = y0 + h / 2.0;
    while y <= y1 {
        let mut xs: Vec<f64> = Vec::new();
        let mut j = n - 1;
        for i in 0..n {
            let (a, b) = (pts[i], pts[j]);
            if (a[1] > y) != (b[1] > y) {
                xs.push((b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]);
            }
            j = i;
        }
        sort(&mut xs, |a, b| a < b);
        let mut len = 0.0;
        let mut k = 0;
        while k + 1 < xs.len() {
            len = len + xs[k + 1] - xs[k];
            k += 2;
        }
        if len > 0.0 {
            total += len;
            rows.push((y, xs));
            cum.push(total);
        }
        y += h;
    }
    if total <= 0.0 {
        return None;
    }
    Some(SpanSampler { rows, cum, total, h })
}

impl SpanSampler {
    pub fn sample(&self, rng: &mut Rng) -> Option<(f64, f64)> {
        let u = rng.next() * self.total;
        let (mut lo, mut hi) = (1usize, self.cum.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.cum[mid - 1] < u {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let (y, xs) = &self.rows[lo - 1];
        let mut into = u - if lo >= 2 { self.cum[lo - 2] } else { 0.0 };
        let nx = xs.len() as i64;
        let mut k: i64 = 1;
        while k <= nx - 1 {
            let (a, b) = (xs[(k - 1) as usize], xs[k as usize]);
            let w = b - a;
            if into <= w || k + 2 > nx - 1 {
                return Some((a + into.min(w), y + (rng.next() - 0.5) * self.h));
            }
            into -= w;
            k += 2;
        }
        None
    }
}

pub fn inside(pts: &[Pt], x: f64, y: f64) -> bool {
    let mut c = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (pts[i], pts[j]);
        if (a[1] > y) != (b[1] > y) && x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0] {
            c = !c;
        }
        j = i;
    }
    c
}

/// Smooth periodic value noise in [-1, 1].
#[derive(Clone)]
pub struct Noise {
    v: Vec<f64>,
}

impl Noise {
    pub fn new(rng: &mut Rng, period: usize) -> Noise {
        Noise { v: (0..period).map(|_| rng.next() * 2.0 - 1.0).collect() }
    }

    #[inline]
    pub fn at(&self, x: f64) -> f64 {
        let i = x.floor();
        let f = x - i;
        let p = self.v.len() as i64;
        let i = i as i64;
        let (a, b) = (self.v[i.rem_euclid(p) as usize], self.v[(i + 1).rem_euclid(p) as usize]);
        let f = f * f * (3.0 - 2.0 * f);
        a + (b - a) * f
    }
}

#[derive(Clone)]
pub struct Fbm {
    n1: Noise,
    n2: Noise,
}

impl Fbm {
    pub fn new(rng: &mut Rng, period: usize) -> Fbm {
        let n1 = Noise::new(rng, period);
        let n2 = Noise::new(rng, period * 2);
        Fbm { n1, n2 }
    }

    #[inline]
    pub fn at(&self, x: f64) -> f64 {
        0.7 * self.n1.at(x) + 0.3 * self.n2.at(x * 2.0)
    }
}

/// Catmull-Rom spline through points, as Bezier commands.
pub fn smooth(pts: &[Pt], closed: bool) -> Vec<Cmd> {
    smooth_t(pts, closed, 1.0)
}

pub fn smooth_t(pts: &[Pt], closed: bool, tension: f64) -> Vec<Cmd> {
    let k = tension / 6.0;
    let n = pts.len();
    let mut cmds = Vec::with_capacity(n + 2);
    cmds.push(Cmd::M(pts[0][0], pts[0][1]));
    if n < 3 {
        for p in &pts[1..] {
            cmds.push(Cmd::L(p[0], p[1]));
        }
        if closed {
            cmds.push(Cmd::Z);
        }
        return cmds;
    }
    let ni = n as i64;
    let at = |i: i64| -> Pt {
        if closed {
            pts[(i - 1).rem_euclid(ni) as usize]
        } else {
            pts[(i.clamp(1, ni) - 1) as usize]
        }
    };
    let last = if closed { ni } else { ni - 1 };
    for i in 1..=last {
        let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
        cmds.push(Cmd::C(
            p1[0] + (p2[0] - p0[0]) * k,
            p1[1] + (p2[1] - p0[1]) * k,
            p2[0] - (p3[0] - p1[0]) * k,
            p2[1] - (p3[1] - p1[1]) * k,
            p2[0],
            p2[1],
        ));
    }
    if closed {
        cmds.push(Cmd::Z);
    }
    cmds
}

pub fn polyline(pts: &[Pt], closed: bool) -> Vec<Cmd> {
    let mut cmds = Vec::with_capacity(pts.len() + 1);
    cmds.push(Cmd::M(pts[0][0], pts[0][1]));
    for p in &pts[1..] {
        cmds.push(Cmd::L(p[0], p[1]));
    }
    if closed {
        cmds.push(Cmd::Z);
    }
    cmds
}

pub fn spline_points(pts: &[Pt], per: usize) -> Vec<Pt> {
    let n = pts.len();
    if n < 2 {
        return vec![pts[0]];
    }
    let ni = n as i64;
    let at = |i: i64| pts[(i.clamp(1, ni) - 1) as usize];
    let mut out = Vec::with_capacity((n - 1) * per + 1);
    for i in 1..ni {
        let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
        for s in 0..per {
            let t = s as f64 / per as f64;
            let (t2, t3) = (t * t, t * t * t);
            let cr = |a: f64, b: f64, c: f64, d: f64| {
                0.5 * ((2.0 * b) + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3)
            };
            out.push([cr(p0[0], p1[0], p2[0], p3[0]), cr(p0[1], p1[1], p2[1], p3[1])]);
        }
    }
    out.push(pts[n - 1]);
    out
}

pub fn length(pts: &[Pt]) -> f64 {
    let mut l = 0.0;
    for i in 1..pts.len() {
        l += dist(pts[i - 1], pts[i]);
    }
    l
}

/// Point and tangent at arclength fraction t.
pub fn at(pts: &[Pt], t: f64) -> (Pt, f64) {
    let total = length(pts);
    let mut target = clamp(t, 0.0, 1.0) * total;
    let n = pts.len();
    for i in 1..n {
        let d = dist(pts[i - 1], pts[i]);
        if target <= d || i == n - 1 {
            let f = if d > 0.0 { clamp(target / d, 0.0, 1.0) } else { 0.0 };
            return (mix(pts[i - 1], pts[i], f), angle(pts[i - 1], pts[i]));
        }
        target -= d;
    }
    (pts[n - 1], 0.0)
}

pub fn resample(pts: &[Pt], n: usize) -> Vec<Pt> {
    let m = pts.len();
    let seg: Vec<f64> = (1..m).map(|i| dist(pts[i - 1], pts[i])).collect();
    let total: f64 = seg.iter().sum();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n as f64 - 1.0);
        let mut target = clamp(t, 0.0, 1.0) * total;
        let mut p = pts[m - 1];
        for j in 1..m {
            let d = seg[j - 1];
            if target <= d || j == m - 1 {
                let f = if d > 0.0 { clamp(target / d, 0.0, 1.0) } else { 0.0 };
                p = mix(pts[j - 1], pts[j], f);
                break;
            }
            target -= d;
        }
        out.push(p);
    }
    out
}

pub fn ribbon(pts: &[Pt], mut w: impl FnMut(f64) -> f64) -> Vec<Pt> {
    let n = pts.len();
    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);
    for i in 0..n {
        let a = pts[i.saturating_sub(1)];
        let b = pts[(i + 1).min(n - 1)];
        let ang = angle(a, b) + PI / 2.0;
        let hw = w(i as f64 / (n as f64 - 1.0));
        left.push(polar(pts[i], ang, hw));
        right.push(polar(pts[i], ang, -hw));
    }
    right.reverse();
    left.extend(right);
    left
}

pub fn deform(pts: &[Pt], rng: &mut Rng, amp: f64) -> Vec<Pt> {
    let nz = Noise::new(rng, pts.len());
    let nz2 = Noise::new(rng, pts.len());
    pts.iter()
        .enumerate()
        .map(|(i, p)| {
            let x = i as f64 * 0.35;
            [p[0] + amp * nz.at(x), p[1] + amp * nz2.at(x)]
        })
        .collect()
}

pub fn curve(p: Pt, ang: f64, len: f64, bend: f64, curl: f64, n: usize) -> Vec<Pt> {
    let mut pts = Vec::with_capacity(n + 1);
    pts.push(p);
    let (mut x, mut y) = (p[0], p[1]);
    let step = len / n as f64;
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let a = ang + bend * t + curl * t.powf(4.0);
        x += step * a.cos();
        y += step * a.sin();
        pts.push([x, y]);
    }
    pts
}

/// A curve from p whose curvature at fraction t is kfun(t). `n` of 0 picks steps.
pub fn flow(p: Pt, heading: f64, len: f64, kfun: impl Fn(f64) -> f64, n: usize) -> (Vec<Pt>, f64) {
    let n = if n > 0 {
        n as f64
    } else {
        let mut kmax: f64 = 0.0;
        for i in 0..64 {
            kmax = kmax.max(kfun((i as f64 + 0.5) / 64.0).abs());
        }
        8f64.max((len / 1.5).ceil()).max((len * kmax / 0.12).ceil())
    };
    let step = len / n;
    let (mut x, mut y, mut a) = (p[0], p[1], heading);
    let mut pts = vec![[x, y]];
    let mut i = 1.0;
    while i <= n {
        a += kfun((i - 0.5) / n) * step;
        x += step * a.cos();
        y += step * a.sin();
        pts.push([x, y]);
        i += 1.0;
    }
    (pts, a)
}

pub fn curl_coefficient(len: f64, turns: f64, pw: f64) -> f64 {
    2.0 * PI * turns * (pw + 1.0) / len
}

pub fn bezier(p0: Pt, p1: Pt, p2: Pt, p3: Pt, n: usize) -> Vec<Pt> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            [a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0], a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1]]
        })
        .collect()
}

/// A smooth stem between two points with given headings; `n` of 0 picks steps.
pub fn link(p0: Pt, dir0: f64, p1: Pt, dir1: f64, k: f64, n: usize) -> Vec<Pt> {
    let h = dist(p0, p1) * k;
    let n = if n > 0 { n } else { 12f64.max((dist(p0, p1) / 1.2).ceil()) as usize };
    bezier(p0, polar(p0, dir0, h), polar(p1, dir1 + PI, h), p1, n)
}

#[inline]
pub fn turn(a: f64, b: f64) -> f64 {
    let d = fmod(a - b, 2.0 * PI);
    if d > PI {
        d - 2.0 * PI
    } else {
        d
    }
}

pub fn depart(p: Pt, along: f64, target: Pt, bias: f64) -> f64 {
    let want = angle(p, target);
    let fwd = if turn(want, along).abs() <= PI / 2.0 { along } else { along + PI };
    let side = if turn(want, fwd) >= 0.0 { 1.0 } else { -1.0 };
    fwd + side * bias
}

pub struct PursueOpts {
    pub step: f64,
    pub kmax: f64,
    pub dk: f64,
    pub k0: f64,
    pub len: f64,
}

impl Default for PursueOpts {
    fn default() -> Self {
        PursueOpts { step: 2.0, kmax: 1.0 / 30.0, dk: 1.0 / 900.0, k0: 0.0, len: 0.0 }
    }
}

pub fn pursue(
    p: Pt,
    heading: f64,
    mut target_at: impl FnMut(Pt, f64, f64) -> Option<Pt>,
    o: &PursueOpts,
    mut stop: impl FnMut(Pt) -> bool,
) -> (Vec<Pt>, f64) {
    let (step, kmax, dk) = (o.step, o.kmax, o.dk);
    let mut pts = vec![p];
    let (mut a, mut k, mut s) = (heading, o.k0, 0.0);
    let (mut x, mut y) = (p[0], p[1]);
    while s < o.len {
        let t = match target_at([x, y], s, a) {
            Some(t) => t,
            None => break,
        };
        let d = dist([x, y], t).max(1e-6);
        let want = clamp(2.0 * turn(angle([x, y], t), a).sin() / d, -kmax, kmax);
        k += clamp(want - k, -dk * step, dk * step);
        a += k * step;
        x += step * a.cos();
        y += step * a.sin();
        if stop([x, y]) {
            break;
        }
        pts.push([x, y]);
        s += step;
    }
    (pts, a)
}

/// A target for `pursue` leading along a polyline.
pub struct Follow<'a> {
    guide: &'a [Pt],
    cum: Vec<f64>,
    j: usize,
    ahead: f64,
    finish: f64,
}

impl<'a> Follow<'a> {
    pub fn new(guide: &'a [Pt], ahead: f64, finish: f64) -> Follow<'a> {
        let mut cum = vec![0.0; guide.len()];
        for i in 1..guide.len() {
            cum[i] = cum[i - 1] + dist(guide[i - 1], guide[i]);
        }
        Follow { guide, cum, j: 0, ahead, finish }
    }

    pub fn target(&mut self, q: Pt) -> Option<Pt> {
        let n = self.guide.len();
        if dist(q, self.guide[n - 1]) < self.finish {
            return None;
        }
        let (mut best, mut bd) = (self.j, dist(self.guide[self.j], q));
        for i in (self.j + 1)..n.min(self.j + 41) {
            let d = dist(self.guide[i], q);
            if d < bd {
                best = i;
                bd = d;
            }
        }
        self.j = best;
        let mut k = self.j;
        while k < n - 1 && self.cum[k] - self.cum[self.j] < self.ahead {
            k += 1;
        }
        Some(self.guide[k])
    }
}
