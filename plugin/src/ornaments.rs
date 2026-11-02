//! The ornament registry: each kind fills one slot, picked by weight.

use crate::mix::Weights;
use crate::orn_kinds;
use crate::orn_raster;
use crate::paint::Paint;
use crate::rng::Rng;
use crate::tune::ORNAMENT_DEFAULTS;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Slot {
    Cartouche,
    Header,
    Pendant,
    Crest,
    Flourish,
    Tailpiece,
    Corner,
    Gilt,
}

impl Slot {
    pub fn name(self) -> &'static str {
        match self {
            Slot::Cartouche => "cartouche",
            Slot::Header => "header",
            Slot::Pendant => "pendant",
            Slot::Crest => "crest",
            Slot::Flourish => "flourish",
            Slot::Tailpiece => "tailpiece",
            Slot::Corner => "corner",
            Slot::Gilt => "gilt",
        }
    }
}

pub struct Kind {
    pub name: &'static str,
    pub slot: Slot,
    pub sparkle: &'static str,
    pub classic: bool,
}

const fn k(name: &'static str, slot: Slot) -> Kind {
    Kind { name, slot, sparkle: "", classic: false }
}

const fn g(name: &'static str, sparkle: &'static str, classic: bool) -> Kind {
    Kind { name, slot: Slot::Gilt, sparkle, classic }
}

pub static KINDS: [Kind; 36] = [
    k("night", Slot::Cartouche), k("compass", Slot::Header), k("knot", Slot::Pendant),
    k("sun", Slot::Cartouche), k("moon", Slot::Cartouche), k("constellation", Slot::Cartouche), k("planet", Slot::Cartouche), k("armillary", Slot::Cartouche),
    k("scrolled", Slot::Cartouche), k("strapwork", Slot::Cartouche),
    k("rosette", Slot::Header), k("sunburst", Slot::Header), k("quatrefoil", Slot::Header),
    k("medallion", Slot::Pendant), k("zodiac_pendant", Slot::Pendant),
    k("sun_crest", Slot::Crest), k("compass_rose", Slot::Crest), k("armillary_crest", Slot::Crest), k("zodiac", Slot::Crest), k("wreath", Slot::Crest),
    k("fleuron_rule", Slot::Flourish), k("headpiece", Slot::Flourish),
    k("cul_de_lampe", Slot::Tailpiece), k("hedera", Slot::Tailpiece), k("garland", Slot::Tailpiece),
    k("bracket", Slot::Corner), k("corner_leaf", Slot::Corner),
    k("copperplate", Slot::Flourish), k("swash", Slot::Flourish), k("swelled_rule", Slot::Flourish), k("fleuronnee", Slot::Flourish),
    k("rinceau", Slot::Flourish), k("acanthus_corner", Slot::Corner),
    k("cornerpiece", Slot::Corner), k("pen_corner", Slot::Corner), k("fan", Slot::Corner),
];

pub static GILT: [Kind; 7] = [
    g("gilt_cross", "cross", true), g("gilt_rosette", "rosette", true), g("gilt_dotted", "dotted", true), g("gilt_lozenge", "lozenge", true),
    g("gilt_star", "star5", false), g("gilt_crescent", "crescent", false), g("gilt_trefoil", "trefoil", false),
];

pub fn all() -> impl Iterator<Item = &'static Kind> {
    KINDS.iter().chain(GILT.iter())
}

pub fn of(slot: Slot) -> Vec<&'static Kind> {
    all().filter(|e| e.slot == slot).collect()
}

pub fn is_kind(name: &str) -> bool {
    all().any(|e| e.name == name)
}

pub fn weight(weights: Option<&Weights>, name: &str) -> f64 {
    if let Some(w) = weights.and_then(|t| t.get(name)) {
        return *w;
    }
    ORNAMENT_DEFAULTS.iter().find(|e| e.0 == name).map_or(1.0, |e| e.1)
}

pub fn pick(rng: &mut Rng, slot: Slot, weights: Option<&Weights>) -> Option<&'static Kind> {
    let t: Vec<(&'static str, f64)> = of(slot).iter().map(|e| (e.name, weight(weights, e.name))).collect();
    let name = rng.weighted(&t)?;
    all().find(|e| e.name == name)
}

#[derive(Clone, Copy, Default)]
pub struct DrawOpts {
    pub cord: f64,
    pub flip: Option<[f64; 2]>,
}

pub fn draw(kind: &Kind, pt: &mut Paint, rng: &mut Rng, b: [f64; 4], o: &DrawOpts) {
    let was = pt.gilding;
    pt.gilding = true;
    orn_kinds::draw(kind, pt, rng, b, o);
    if pt.gilt_pending.is_some() {
        orn_raster::flush(pt);
    }
    pt.gilding = was;
}
