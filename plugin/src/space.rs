//! Occupancy grid over the page, with a clearance field.

const BIG: f64 = 1e9;

pub struct Space {
    pub c: f64,
    pub nx: i64,
    pub ny: i64,
    pub v: Vec<u8>,
    pub f: Vec<f64>,
    pub maxf: f64,
}

impl Space {
    pub fn new(width: f64, height: f64, cell: f64) -> Space {
        let (nx, ny) = ((width / cell).ceil() as i64, (height / cell).ceil() as i64);
        Space { c: cell, nx, ny, v: vec![1; (nx * ny) as usize], f: Vec::new(), maxf: 0.0 }
    }

    #[inline]
    fn idx(&self, i: i64, j: i64) -> usize {
        (j * self.nx + i) as usize
    }

    fn set_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, val: u8) {
        let c = self.c;
        let (i0, i1) = (((x0 / c).floor() as i64).max(0), ((x1 / c).ceil() as i64 - 1).min(self.nx - 1));
        let (j0, j1) = (((y0 / c).floor() as i64).max(0), ((y1 / c).ceil() as i64 - 1).min(self.ny - 1));
        for j in j0..=j1 {
            for i in i0..=i1 {
                let k = self.idx(i, j);
                self.v[k] = val;
            }
        }
    }

    pub fn open(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.set_rect(x0, y0, x1, y1, 0);
    }

    pub fn block_text(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.set_rect(x0, y0, x1, y1, 2);
    }

    pub fn close(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.set_rect(x0, y0, x1, y1, 1);
    }

    #[inline]
    pub fn free(&self, x: f64, y: f64) -> bool {
        let (i, j) = ((x / self.c).floor() as i64, (y / self.c).floor() as i64);
        if i < 0 || j < 0 || i >= self.nx || j >= self.ny {
            return false;
        }
        self.v[self.idx(i, j)] == 0
    }

    #[inline]
    pub fn open_at(&self, x: f64, y: f64) -> bool {
        let (i, j) = ((x / self.c).floor() as i64, (y / self.c).floor() as i64);
        if i < 0 || j < 0 || i >= self.nx || j >= self.ny {
            return false;
        }
        let v = self.v[self.idx(i, j)];
        v == 0 || v == 3 || v == 4
    }

    pub fn clearance(&self, x: f64, y: f64, maxr: f64) -> f64 {
        if !self.free(x, y) {
            return 0.0;
        }
        let c = self.c;
        let (ci, cj) = ((x / c).floor() as i64, (y / c).floor() as i64);
        let rings = (maxr / c).ceil() as i64;
        let (nx, ny) = (self.nx, self.ny);
        let v = &self.v;
        let test = |best: &mut f64, di: i64, dj: i64| {
            let d = ((di * di + dj * dj) as f64).sqrt() * c;
            if d < *best {
                let (i, j) = (ci + di, cj + dj);
                if !(i >= 0 && j >= 0 && i < nx && j < ny) || v[(j * nx + i) as usize] != 0 {
                    *best = d;
                }
            }
        };
        let mut best = maxr + c;
        for r in 1..=rings {
            if (r - 1) as f64 * c >= best {
                break;
            }
            for di in -r..=r {
                test(&mut best, di, -r);
                test(&mut best, di, r);
            }
            for dj in (-r + 1)..=(r - 1) {
                test(&mut best, -r, dj);
                test(&mut best, r, dj);
            }
        }
        maxr.min((best - c).max(0.0))
    }

    pub fn field(&mut self) {
        let (nx, ny, c) = (self.nx, self.ny, self.c);
        let mut f: Vec<f64> = self.v.iter().map(|cv| if *cv == 0 || *cv == 4 { BIG } else { 0.0 }).collect();
        let (d1, d2) = (1.0, 2f64.sqrt());
        let at = |f: &[f64], i: i64, j: i64| -> f64 {
            if i < 0 || j < 0 || i >= nx || j >= ny {
                0.0
            } else {
                f[(j * nx + i) as usize]
            }
        };
        for j in 0..ny {
            for i in 0..nx {
                let k = (j * nx + i) as usize;
                if f[k] > 0.0 {
                    f[k] = f[k]
                        .min(at(&f, i - 1, j) + d1)
                        .min(at(&f, i, j - 1) + d1)
                        .min(at(&f, i - 1, j - 1) + d2)
                        .min(at(&f, i + 1, j - 1) + d2);
                }
            }
        }
        for j in (0..ny).rev() {
            for i in (0..nx).rev() {
                let k = (j * nx + i) as usize;
                if f[k] > 0.0 {
                    f[k] = f[k]
                        .min(at(&f, i + 1, j) + d1)
                        .min(at(&f, i, j + 1) + d1)
                        .min(at(&f, i + 1, j + 1) + d2)
                        .min(at(&f, i - 1, j + 1) + d2);
                }
            }
        }
        let mut maxf = 0.0;
        for e in f.iter_mut() {
            *e = ((*e - 0.5) * c).max(0.0);
            if *e > maxf {
                maxf = *e;
            }
        }
        self.f = f;
        self.maxf = maxf;
    }

    #[inline]
    pub fn room(&self, x: f64, y: f64) -> f64 {
        let (i, j) = ((x / self.c).floor() as i64, (y / self.c).floor() as i64);
        if i < 0 || j < 0 || i >= self.nx || j >= self.ny {
            return -1.0;
        }
        self.f[self.idx(i, j)]
    }

    pub fn disc(&mut self, x: f64, y: f64, r: f64, k: f64) {
        let kr = k * r;
        if !self.f.is_empty() {
            let c = self.c;
            let reach = kr + self.maxf;
            let (i0, i1) = (((x - reach) / c).floor().max(0.0) as i64, ((x + reach) / c).floor().min((self.nx - 1) as f64) as i64);
            let (j0, j1) = (((y - reach) / c).floor().max(0.0) as i64, ((y + reach) / c).floor().min((self.ny - 1) as f64) as i64);
            for j in j0..=j1 {
                let cy = (j as f64 + 0.5) * c - y;
                for i in i0..=i1 {
                    let cx = (i as f64 + 0.5) * c - x;
                    let k2 = (j * self.nx + i) as usize;
                    let d = (cx * cx + cy * cy).sqrt() - kr;
                    if d < self.f[k2] {
                        self.f[k2] = d.max(0.0);
                    }
                }
            }
        }
        self.mark(x, y, kr, 3);
    }

    pub fn boxed(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        let c = self.c;
        if !self.f.is_empty() {
            let reach = self.maxf;
            let (i0, i1) = (((x0 - reach) / c).floor().max(0.0) as i64, ((x1 + reach) / c).floor().min((self.nx - 1) as f64) as i64);
            let (j0, j1) = (((y0 - reach) / c).floor().max(0.0) as i64, ((y1 + reach) / c).floor().min((self.ny - 1) as f64) as i64);
            for j in j0..=j1 {
                let cy = (j as f64 + 0.5) * c;
                let dy = (y0 - cy).max(cy - y1).max(0.0);
                for i in i0..=i1 {
                    let cx = (i as f64 + 0.5) * c;
                    let dx = (x0 - cx).max(cx - x1).max(0.0);
                    let k = (j * self.nx + i) as usize;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < self.f[k] {
                        self.f[k] = d;
                    }
                }
            }
        }
        let (i0, i1) = (((x0 / c).floor() as i64).max(0), ((x1 / c).floor() as i64).min(self.nx - 1));
        let (j0, j1) = (((y0 / c).floor() as i64).max(0), ((y1 / c).floor() as i64).min(self.ny - 1));
        for j in j0..=j1 {
            for i in i0..=i1 {
                let k = self.idx(i, j);
                if self.v[k] == 0 || self.v[k] == 4 {
                    self.v[k] = 3;
                }
            }
        }
    }

    pub fn reserve(&mut self, x: f64, y: f64, r: f64) {
        if !self.f.is_empty() {
            self.disc(x, y, r, 1.0);
        } else {
            self.mark(x, y, r, 3);
        }
    }

    pub fn mark(&mut self, x: f64, y: f64, r: f64, val: u8) {
        let c = self.c;
        let (i0, i1) = (((x - r) / c).floor().max(0.0) as i64, ((x + r) / c).floor().min((self.nx - 1) as f64) as i64);
        let (j0, j1) = (((y - r) / c).floor().max(0.0) as i64, ((y + r) / c).floor().min((self.ny - 1) as f64) as i64);
        for j in j0..=j1 {
            for i in i0..=i1 {
                let (cx, cy) = ((i as f64 + 0.5) * c, (j as f64 + 0.5) * c);
                if (cx - x) * (cx - x) + (cy - y) * (cy - y) <= r * r {
                    let k = self.idx(i, j);
                    if self.v[k] == 0 {
                        self.v[k] = val;
                    }
                }
            }
        }
    }

    pub fn reserve_corridor(&mut self, pts: &[[f64; 2]], r: f64) {
        for p in pts.iter().step_by(2) {
            self.mark(p[0], p[1], r, 4);
        }
    }

    pub fn free_cells(&self, step: usize) -> Vec<[f64; 2]> {
        let mut out = Vec::new();
        for j in (0..self.ny).step_by(step) {
            for i in (0..self.nx).step_by(step) {
                if self.v[self.idx(i, j)] == 0 {
                    out.push([(i as f64 + 0.5) * self.c, (j as f64 + 0.5) * self.c]);
                }
            }
        }
        out
    }

    pub fn coverage(&self) -> f64 {
        let (mut free, mut placed) = (0.0, 0.0);
        for v in &self.v {
            if *v == 0 {
                free += 1.0;
            } else if *v == 3 || *v == 4 {
                placed += 1.0;
            }
        }
        if free + placed == 0.0 {
            1.0
        } else {
            placed / (free + placed)
        }
    }
}
