//! Tunable parameters, as the Lua generator's tune.lua defaults.

use crate::dist::Curve;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

pub type R2 = [f64; 2];

pub struct Gesture {
    pub edge: f64,
    pub bows: f64,
    pub bow_amp: R2,
    pub bow_width: R2,
    pub density_floor: f64,
    pub bow_density: f64,
    pub end_reach: f64,
}

pub struct PlantCfg {
    pub root: f64,
    pub lookahead: f64,
    pub bend_radius: f64,
    pub bend_ease: f64,
    pub long_shoots: f64,
    pub shoot_start: R2,
    pub shoot_end: R2,
    pub shoot_reach: R2,
    pub shoot_wave: R2,
    pub foot_shoots: f64,
    pub foot_reach: R2,
    pub shoot_bend: f64,
    pub root_width: f64,
    pub tip_width: f64,
    pub fork_ratio: f64,
    pub stem_l: f64,
    pub stem_brush: f64,
}

pub struct Hero {
    pub style: &'static str,
    pub petals: i64,
    pub petal_width: f64,
    pub rotation: f64,
    pub lost_edges: f64,
    pub tilt: f64,
}

pub struct BloomStyle {
    pub weight: f64,
    pub petals: R2,
    pub width: R2,
    pub ragged: f64,
    pub spacing: f64,
    pub rim: f64,
    pub rays: R2,
    pub ray_reach: R2,
    pub ray_contrast: R2,
    pub veins: R2,
    pub gap: Option<f64>,
    pub eye_fade: Option<f64>,
    pub backruns: Option<f64>,
}

#[derive(Clone)]
pub struct Compose {
    pub hero: f64,
    pub features: R2,
    pub leaves: f64,
    pub leaf_size: f64,
    pub leaf_width: R2,
    pub leaf_overlap: f64,
    pub leaf_spacing: R2,
    pub hole_min: f64,
    pub sprays: f64,
    pub small_blooms: f64,
    pub curls: f64,
    pub leaflet_spacing: R2,
    pub leaflet_size: f64,
    pub no_leaflets: bool,
}

pub struct Look {
    pub base_l: R2,
    pub tip_l: R2,
    pub chroma: R2,
    pub curve: R2,
}

pub struct Color {
    pub hue: f64,
    pub hue_jitter: f64,
    pub hue_drift: f64,
    pub grey_chance: f64,
    pub grey_share: f64,
    pub vivid: Look,
    pub hero: Look,
    pub fan: Look,
    pub soft: Look,
    pub ghost: Look,
    pub leaf: Look,
}

pub struct Wash {
    pub blotches: R2,
    pub blotch_opacity: R2,
    pub lifts: R2,
    pub lift_opacity: R2,
    pub blotch_hue: f64,
    pub tideline_chance: f64,
    pub streak_scale: f64,
    pub brush_bands: R2,
    pub brush_pale: R2,
    pub brush_width: f64,
    pub brush_gap: f64,
    pub brush_dry: f64,
    pub band_opacity: R2,
    pub pale_contrast: R2,
    pub vein_contrast: R2,
    pub lost_edges: f64,
    pub lost_fade: f64,
    pub grain_clump: f64,
    pub grain_mottle: f64,
    pub vein_opacity: f64,
}

pub struct Sizes {
    pub gyps: Curve,
    pub leaf: Curve,
    pub leaflet: Curve,
}

pub struct Tune {
    pub gesture: Gesture,
    pub plant: PlantCfg,
    pub hero: Hero,
    pub fan: BloomStyle,
    pub round: BloomStyle,
    pub soft: BloomStyle,
    pub obovate: BloomStyle,
    pub wedge: BloomStyle,
    pub compose: Compose,
    pub gilt_dpi: f64,
    pub sizes: Sizes,
    pub color: Color,
    pub wash: Wash,
}

impl Tune {
    pub fn bloom(&self, name: &str) -> &BloomStyle {
        match name {
            "fan" => &self.fan,
            "round" => &self.round,
            "soft" => &self.soft,
            "obovate" => &self.obovate,
            _ => &self.wedge,
        }
    }

    pub fn look(&self, name: &str) -> &Look {
        match name {
            "hero" => &self.color.hero,
            "fan" => &self.color.fan,
            "soft" => &self.color.soft,
            "ghost" => &self.color.ghost,
            "leaf" => &self.color.leaf,
            _ => &self.color.vivid,
        }
    }
}

pub const MIX_DEFAULTS: &[(&str, f64)] = &[
    ("bloom", 1.0), ("soft", 1.0), ("lily", 1.0), ("ghost", 1.0), ("umbel", 1.0), ("cyme", 1.0), ("gyps", 1.0),
    ("leaves", 1.0), ("leaflets", 1.0), ("sprays", 1.0), ("curls", 1.0), ("density", 1.0),
    ("bee", 0.0), ("butterfly", 0.0), ("bird", 0.0), ("snail", 0.0),
];

pub const ORNAMENT_DEFAULTS: &[(&str, f64)] = &[
    ("night", 3.0), ("sun", 1.0), ("moon", 1.0), ("constellation", 1.0), ("planet", 1.0), ("armillary", 1.0), ("scrolled", 1.0), ("strapwork", 1.0),
    ("compass", 3.0), ("rosette", 1.0), ("sunburst", 1.0), ("quatrefoil", 1.0),
    ("knot", 3.0), ("medallion", 1.0), ("zodiac_pendant", 1.0),
    ("sun_crest", 2.0), ("compass_rose", 2.0), ("armillary_crest", 1.0), ("zodiac", 1.0), ("wreath", 1.0),
    ("swash", 1.0), ("fleuron_rule", 1.0), ("headpiece", 1.0), ("copperplate", 2.0), ("swelled_rule", 2.0), ("fleuronnee", 1.0), ("rinceau", 2.0),
    ("cul_de_lampe", 1.0), ("hedera", 1.0), ("garland", 1.0),
    ("bracket", 1.0), ("fan", 1.0), ("corner_leaf", 1.0), ("acanthus_corner", 2.0), ("cornerpiece", 2.0), ("pen_corner", 1.0),
    ("gilt_cross", 4.0), ("gilt_rosette", 3.0), ("gilt_dotted", 3.0), ("gilt_lozenge", 1.0),
    ("gilt_star", 1.5), ("gilt_crescent", 1.0), ("gilt_trefoil", 1.0),
];

pub const PALETTE_DEFAULTS: &[(&str, &str)] = &[
    ("hero", "look"), ("blooms", "look"), ("leaves", "blue"), ("leaflets", "blue"), ("stems", "chiku-rin"),
    ("sprays", "look"), ("ghost", "look"), ("cartouche", "look"),
];

fn curve(range: R2, points: &[[f64; 2]]) -> Curve {
    Curve { range, points: points.to_vec() }
}

pub fn tune() -> &'static Tune {
    static T: OnceLock<Tune> = OnceLock::new();
    T.get_or_init(|| Tune {
        gesture: Gesture {
            edge: 0.24, bows: 3.0, bow_amp: [0.20, 0.33], bow_width: [0.34, 0.42],
            density_floor: 0.24, bow_density: 0.6, end_reach: 3.0,
        },
        plant: PlantCfg {
            root: 0.64, lookahead: 82.0, bend_radius: 91.0, bend_ease: 23.0, long_shoots: 2.0,
            shoot_start: [-0.15, 0.3], shoot_end: [0.6, 1.0], shoot_reach: [0.24, 0.36], shoot_wave: [220.0, 330.0],
            foot_shoots: 5.0, foot_reach: [34.0, 70.0], shoot_bend: 29.0, root_width: 5.8, tip_width: 3.0,
            fork_ratio: 0.8, stem_l: 0.25, stem_brush: 1.0,
        },
        hero: Hero { style: "wedge", petals: 5, petal_width: 0.35, rotation: -1.0, lost_edges: 0.56, tilt: 35.0 },
        fan: BloomStyle { weight: 3.0, petals: [5.0, 6.0], width: [0.32, 0.40], ragged: 0.10, spacing: 0.10, rim: 0.16,
            rays: [28.0, 44.0], ray_reach: [0.25, 0.85], ray_contrast: [0.35, 0.70], veins: [10.0, 18.0], gap: None, eye_fade: None, backruns: None },
        round: BloomStyle { weight: 2.0, petals: [5.0, 5.0], width: [0.52, 0.58], ragged: 0.08, spacing: 0.12, rim: 0.07,
            rays: [0.0, 0.0], ray_reach: [0.30, 0.80], ray_contrast: [0.45, 0.80], veins: [22.0, 22.0], gap: None, eye_fade: None, backruns: None },
        soft: BloomStyle { weight: 2.0, petals: [5.0, 5.0], width: [0.62, 0.72], ragged: 0.08, spacing: 0.12, rim: 0.07,
            rays: [0.0, 0.0], ray_reach: [0.30, 0.80], ray_contrast: [0.45, 0.80], veins: [12.0, 12.0], gap: None, eye_fade: None, backruns: None },
        obovate: BloomStyle { weight: 0.0, petals: [5.0, 5.0], width: [0.30, 0.36], ragged: 0.08, spacing: 0.12, rim: 0.11,
            rays: [0.0, 0.0], ray_reach: [0.30, 0.80], ray_contrast: [0.45, 0.80], veins: [22.0, 22.0], gap: None, eye_fade: None, backruns: None },
        wedge: BloomStyle { weight: 3.2, petals: [5.0, 5.0], width: [0.35, 0.44], ragged: 0.15, spacing: 0.09, rim: 0.12,
            rays: [30.0, 44.0], ray_reach: [0.25, 0.6], ray_contrast: [0.4, 0.6], veins: [12.0, 18.0], gap: Some(0.08), eye_fade: Some(0.28), backruns: Some(0.8) },
        compose: Compose {
            hero: 0.5, features: [1.0, 1.3], leaves: 100.0, leaf_size: 1.25, leaf_width: [0.17, 0.23], leaf_overlap: 0.45,
            leaf_spacing: [26.0, 44.0], hole_min: 7.0, sprays: 150.0, small_blooms: 6.0, curls: 0.45,
            leaflet_spacing: [40.0, 72.0], leaflet_size: 17.5, no_leaflets: false,
        },
        gilt_dpi: 600.0,
        sizes: Sizes {
            gyps: curve([0.3, 1.0], &[[0.0, 0.35], [0.25, 0.8], [0.5, 1.0], [0.75, 0.75], [1.0, 0.5]]),
            leaf: curve([0.35, 1.3], &[[0.0, 0.4], [0.25, 0.9], [0.5, 1.0], [0.75, 0.6], [1.0, 0.3]]),
            leaflet: curve([0.5, 2.0], &[[0.0, 0.6], [0.25, 1.0], [0.5, 0.7], [0.75, 0.35], [1.0, 0.15]]),
        },
        color: Color {
            hue: 263.0, hue_jitter: 6.0, hue_drift: -24.0, grey_chance: 0.12, grey_share: 0.25,
            vivid: Look { base_l: [0.80, 0.90], tip_l: [0.2, 0.32], chroma: [0.65, 1.00], curve: [0.4, 1.3] },
            hero: Look { base_l: [0.74, 1.0], tip_l: [0.36, 0.48], chroma: [0.81, 1.0], curve: [0.35, 0.75] },
            fan: Look { base_l: [0.68, 0.80], tip_l: [0.36, 0.50], chroma: [0.80, 1.00], curve: [0.35, 0.80] },
            soft: Look { base_l: [0.95, 0.99], tip_l: [0.48, 0.68], chroma: [0.40, 0.85], curve: [0.8, 1.8] },
            ghost: Look { base_l: [0.97, 0.99], tip_l: [0.83, 0.88], chroma: [0.25, 0.3], curve: [1.0, 1.2] },
            leaf: Look { base_l: [0.66, 0.8], tip_l: [0.32, 0.65], chroma: [0.65, 1.0], curve: [0.4, 2.2] },
        },
        wash: Wash {
            blotches: [2.0, 4.0], blotch_opacity: [0.10, 0.24], lifts: [0.0, 1.0], lift_opacity: [0.18, 0.32],
            blotch_hue: 10.0, tideline_chance: 0.55, streak_scale: 0.05, brush_bands: [5.0, 10.0], brush_pale: [10.0, 20.0],
            brush_width: 1.0, brush_gap: 0.35, brush_dry: 0.3, band_opacity: [0.38, 0.62], pale_contrast: [0.12, 0.65],
            vein_contrast: [0.15, 0.55], lost_edges: 0.42, lost_fade: 1.0, grain_clump: 0.55, grain_mottle: 0.8, vein_opacity: 0.34,
        },
    })
}

thread_local! {
    static PALETTE: RefCell<HashMap<&'static str, &'static str>> = RefCell::new(PALETTE_DEFAULTS.iter().copied().collect());
}

/// What paints a part, as tune.color.palette holds it for the current page.
pub fn palette(part: &str) -> Option<&'static str> {
    PALETTE.with(|p| p.borrow().get(part).copied())
}

pub fn palette_all() -> HashMap<&'static str, &'static str> {
    PALETTE.with(|p| p.borrow().clone())
}

pub fn set_palette(m: HashMap<&'static str, &'static str>) {
    PALETTE.with(|p| *p.borrow_mut() = m);
}
