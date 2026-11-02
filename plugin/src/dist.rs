//! Size distributions drawn as curves (monotone cubic splines).

use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Curve {
    pub range: [f64; 2],
    pub points: Vec<[f64; 2]>,
}

fn slopes(xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len();
    let d: Vec<f64> = (0..n - 1).map(|i| (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]).max(1e-9)).collect();
    let mut m = vec![0.0; n];
    m[0] = d[0];
    m[n - 1] = d[n - 2];
    for i in 1..n - 1 {
        m[i] = if d[i - 1] * d[i] <= 0.0 { 0.0 } else { (d[i - 1] + d[i]) / 2.0 };
    }
    for i in 0..n - 1 {
        if d[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let (a, b) = (m[i] / d[i], m[i + 1] / d[i]);
            let s = a * a + b * b;
            if s > 9.0 {
                let t = 3.0 / s.sqrt();
                m[i] = t * a * d[i];
                m[i + 1] = t * b * d[i];
            }
        }
    }
    m
}

pub fn spline(points: &[[f64; 2]]) -> impl Fn(f64) -> f64 {
    let xs: Vec<f64> = points.iter().map(|p| p[0]).collect();
    let ys: Vec<f64> = points.iter().map(|p| p[1].max(0.0)).collect();
    let m = if xs.len() > 1 { slopes(&xs, &ys) } else { vec![] };
    move |x: f64| {
        let n = xs.len();
        if n == 1 || x <= xs[0] {
            return ys[0];
        }
        if x >= xs[n - 1] {
            return ys[n - 1];
        }
        let mut i = 0;
        while xs[i + 1] < x {
            i += 1;
        }
        let h = xs[i + 1] - xs[i];
        let t = (x - xs[i]) / h;
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * ys[i] + (t3 - 2.0 * t2 + t) * h * m[i] + (-2.0 * t3 + 3.0 * t2) * ys[i + 1] + (t3 - t2) * h * m[i + 1]
    }
}

pub fn sample(rng: &mut Rng, c: &Curve) -> f64 {
    let (lo, hi) = (c.range[0], c.range[1]);
    let f = spline(&c.points);
    let w: Vec<f64> = (0..=64).map(|i| f(i as f64 / 64.0).max(0.0)).collect();
    let n = w.len();
    let mut total = 0.0;
    for i in 0..n - 1 {
        total += (w[i].max(0.0) + w[i + 1].max(0.0)) / 2.0;
    }
    if total <= 0.0 {
        return lo + (hi - lo) * rng.next();
    }
    let mut r = rng.next() * total;
    for i in 0..n - 1 {
        let (a, b) = (w[i].max(0.0), w[i + 1].max(0.0));
        let mass = (a + b) / 2.0;
        if r <= mass || i == n - 2 {
            r = r.min(mass);
            let t = if (b - a).abs() < 1e-9 {
                if a > 0.0 {
                    r / a
                } else {
                    0.5
                }
            } else {
                (-a + (a * a + 2.0 * (b - a) * r).max(0.0).sqrt()) / (b - a)
            };
            let t = t.max(0.0).min(1.0);
            return lo + (hi - lo) * (i as f64 + t) / (n as f64 - 1.0);
        }
        r -= mass;
    }
    hi
}
