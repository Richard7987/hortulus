//! The page's mix: how much of each kind of growth a page gets.

use crate::color::group_of;
use crate::inks::ORDER;
use crate::rng::Rng;
use crate::tune::{set_palette, palette_all, Compose, MIX_DEFAULTS};
use std::collections::BTreeMap;
use std::rc::Rc;

pub type Weights = BTreeMap<String, f64>;
pub type Dens = Rc<dyn Fn(f64, f64) -> f64>;

pub const VISITORS: [&str; 4] = ["bee", "butterfly", "bird", "snail"];

pub fn weight(mix: Option<&Weights>, kind: &str) -> f64 {
    if let Some(w) = mix.and_then(|m| m.get(kind)) {
        return *w;
    }
    MIX_DEFAULTS.iter().find(|e| e.0 == kind).map_or(1.0, |e| e.1)
}

fn neutral(mix: Option<&Weights>) -> bool {
    ["leaves", "leaflets", "sprays", "curls", "density"].iter().all(|k| weight(mix, k) == 1.0)
}

pub fn cfg(base: &Compose, mix: Option<&Weights>) -> Compose {
    let mut c = base.clone();
    if neutral(mix) {
        return c;
    }
    let k = weight(mix, "density").max(0.05);
    c.hole_min = base.hole_min / k;
    let w = weight(mix, "leaves");
    if w <= 0.0 {
        c.leaves = 0.0;
    } else {
        let s = 1.0 / (w * k);
        c.leaf_spacing = [base.leaf_spacing[0] * s, base.leaf_spacing[1] * s];
        c.leaves = (base.leaves * w.max(1.0) + 0.5).floor();
    }
    let w = weight(mix, "leaflets");
    c.no_leaflets = w <= 0.0;
    if w > 0.0 {
        let s = 1.0 / (w * k);
        c.leaflet_spacing = [base.leaflet_spacing[0] * s, base.leaflet_spacing[1] * s];
    }
    c.sprays = (base.sprays * weight(mix, "sprays") * k.min(1.0) + 0.5).floor();
    c.curls = base.curls * weight(mix, "curls");
    c
}

pub fn density(dens: Dens, mix: Option<&Weights>) -> Dens {
    let k = weight(mix, "density");
    if k == 1.0 {
        return dens;
    }
    Rc::new(move |x, y| (dens(x, y) * k).min(1.0))
}

pub type Palette = BTreeMap<String, String>;

pub fn resolve(palette: Option<&Palette>, seed: f64, spin: f64) -> Option<BTreeMap<String, &'static str>> {
    let p = palette?;
    let mut out = BTreeMap::new();
    for (part, v) in p {
        let ink: &'static str = if v == "random" {
            let mut r = Rng::new(seed + spin * 104723.0).fork(&format!("ink {}", part));
            ORDER[(r.int(1, ORDER.len() as i64) - 1) as usize]
        } else {
            intern(v)
        };
        out.insert(part.clone(), ink);
    }
    Some(out)
}

fn intern(v: &str) -> &'static str {
    if v == "look" {
        return "look";
    }
    if let Some(n) = ORDER.iter().find(|n| **n == v) {
        return n;
    }
    if group_of(v).is_some() {
        return crate::inks::GROUPS.iter().find(|e| e.0 == v).unwrap().0;
    }
    "look"
}

/// Run `f` with the palette overridden, restoring it afterwards.
pub fn with_palette<T>(palette: Option<BTreeMap<String, &'static str>>, f: impl FnOnce() -> T) -> T {
    let p = match palette {
        Some(p) if !p.is_empty() => p,
        _ => return f(),
    };
    let cur = palette_all();
    let mut tmp = cur.clone();
    for (k, v) in &p {
        if let Some(e) = crate::tune::PALETTE_DEFAULTS.iter().find(|e| e.0 == k) {
            tmp.insert(e.0, *v);
        }
    }
    set_palette(tmp);
    let r = f();
    set_palette(cur);
    r
}
