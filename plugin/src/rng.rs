//! Seeded pseudo-random streams (mulberry32), bit-compatible with the Lua generator.

use crate::lua::sort;

const MASK: u64 = 0xffff_ffff;

pub fn hash(s: &str) -> u64 {
    let mut h: u64 = 2166136261;
    for b in s.bytes() {
        h = ((h ^ b as u64).wrapping_mul(16777619)) & MASK;
    }
    h
}

#[derive(Clone, Debug)]
pub struct Rng {
    pub seed: u64,
    pub s: u64,
}

impl Rng {
    pub fn new(seed: f64) -> Rng {
        let seed = (seed.floor() as i64 as u64) & MASK;
        Rng { seed, s: seed }
    }

    fn from_int(seed: u64) -> Rng {
        let seed = seed & MASK;
        Rng { seed, s: seed }
    }

    pub fn next(&mut self) -> f64 {
        self.s = (self.s.wrapping_add(0x6D2B79F5)) & MASK;
        let mut t = self.s;
        t = ((t ^ (t >> 15)).wrapping_mul(t | 1)) & MASK;
        t = (t ^ (t.wrapping_add(((t ^ (t >> 7)).wrapping_mul(t | 61)) & MASK))) & MASK;
        ((t ^ (t >> 14)) & MASK) as f64 / 4294967296.0
    }

    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.next()
    }

    pub fn rangev(&mut self, r: [f64; 2]) -> f64 {
        self.range(r[0], r[1])
    }

    pub fn int(&mut self, a: i64, b: i64) -> i64 {
        a + (self.next() * (b - a + 1) as f64).floor() as i64
    }

    pub fn jit(&mut self, a: f64) -> f64 {
        (self.next() * 2.0 - 1.0) * a
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.next() < p
    }

    pub fn normal(&mut self, mean: f64, sd: f64) -> f64 {
        let u = self.next().max(1e-12);
        let v = self.next();
        mean + sd * (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }

    pub fn pick<'a, T>(&mut self, t: &'a [T]) -> &'a T {
        &t[(self.int(1, t.len() as i64) - 1) as usize]
    }

    /// Pick a key from (key, weight) pairs, visiting keys in byte-sorted order.
    pub fn weighted<K: AsRef<str> + Clone>(&mut self, weights: &[(K, f64)]) -> Option<K> {
        let mut keys: Vec<&(K, f64)> = weights.iter().filter(|e| e.1 > 0.0).collect();
        let total: f64 = {
            // Lua sums in table order; the sum is taken here in the same sorted
            // order used for the walk. Weights are small decimals, so any
            // difference would sit far below a draw's resolution.
            keys.iter().map(|e| e.1).sum()
        };
        if total <= 0.0 {
            return None;
        }
        sort(&mut keys, |a, b| a.0.as_ref().as_bytes() < b.0.as_ref().as_bytes());
        let mut x = self.next() * total;
        for e in &keys {
            x -= e.1;
            if x < 0.0 {
                return Some(e.0.clone());
            }
        }
        keys.last().map(|e| e.0.clone())
    }

    pub fn clone_at(&self) -> Rng {
        self.clone()
    }

    pub fn fork(&self, label: &str) -> Rng {
        Rng::from_int(hash(&format!("{}:{}", label, self.seed)))
    }
}
