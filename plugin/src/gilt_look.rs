//! The looks of raster gilt: what its gold reflects, how it is cut, its finish.

use crate::color::Rgb;

#[derive(Clone)]
pub struct Look {
    pub studio: bool,
    pub light: f64,
    pub eye: f64,
    pub relief: f64,
    pub cushion: f64,
    pub facet: f64,
    pub highlight: f64,
    pub highlight_size: f64,
    pub grain: f64,
    pub grain_scale: f64,
    pub sparkle: f64,
    pub sparkle_glint: f64,
    pub shadow: f64,
    pub shadow_offset: f64,
    pub horizon: f64,
    pub horizon_soft: f64,
    pub sky: f64,
    pub sky_glow: f64,
    pub ground: f64,
    pub horizon_dip: f64,
    pub sky_lit: f64,
    pub sky_far: f64,
    pub window: f64,
    pub window_top: f64,
    pub window_foot: f64,
    pub ground_fall: f64,
    pub ground_lit: f64,
    pub band: f64,
    pub band_width: f64,
    pub ground_far: f64,
    pub front: f64,
    pub gold: &'static [(f64, Rgb)],
    pub copper: &'static [(f64, Rgb)],
    pub umber: Rgb,
}

const STUDIO_GOLD: [(f64, Rgb); 6] = [
    (0.00, [0.170, 0.090, 0.030]), (0.22, [0.380, 0.230, 0.090]), (0.45, [0.660, 0.480, 0.220]),
    (0.65, [0.820, 0.650, 0.340]), (0.82, [0.950, 0.840, 0.560]), (1.00, [1.000, 0.975, 0.860]),
];
const STUDIO_COPPER: [(f64, Rgb); 5] = [
    (0.00, [0.150, 0.060, 0.025]), (0.25, [0.360, 0.170, 0.080]), (0.50, [0.620, 0.360, 0.190]),
    (0.75, [0.840, 0.600, 0.400]), (1.00, [0.990, 0.900, 0.780]),
];
const MIRROR_GOLD: [(f64, Rgb); 6] = [
    (0.00, [0.240, 0.130, 0.050]), (0.20, [0.420, 0.270, 0.100]), (0.45, [0.680, 0.500, 0.220]),
    (0.62, [0.840, 0.670, 0.330]), (0.80, [0.950, 0.840, 0.520]), (1.00, [1.000, 0.970, 0.840]),
];
const MIRROR_COPPER: [(f64, Rgb); 6] = [
    (0.00, [0.216, 0.088, 0.030]), (0.20, [0.378, 0.184, 0.060]), (0.45, [0.612, 0.340, 0.132]),
    (0.62, [0.756, 0.456, 0.198]), (0.80, [0.855, 0.571, 0.312]), (1.00, [0.900, 0.660, 0.504]),
];

fn base(studio: bool) -> Look {
    let common = Look {
        studio, light: 135.0, eye: 0.7, relief: 0.45, cushion: 0.15, facet: 0.4, highlight: 0.45, highlight_size: 0.1,
        grain: 0.035, grain_scale: 1.0, sparkle: 4.0, sparkle_glint: 0.35, shadow: 0.42, shadow_offset: 1.0,
        horizon: 0.0, horizon_soft: 0.0, sky: 0.0, sky_glow: 0.0, ground: 0.0, horizon_dip: 0.0, sky_lit: 0.0, sky_far: 0.0,
        window: 0.0, window_top: 0.0, window_foot: 0.0, ground_fall: 0.0, ground_lit: 0.0, band: 0.0, band_width: 0.0,
        ground_far: 0.0, front: 0.0, gold: &STUDIO_GOLD, copper: &STUDIO_COPPER, umber: [0.330, 0.200, 0.090],
    };
    if studio {
        Look {
            horizon: 0.04, horizon_dip: 0.22, horizon_soft: 0.035, sky: 0.5, sky_glow: 0.3, sky_lit: 0.25, sky_far: 0.15,
            window: 0.2, window_top: 0.75, window_foot: 0.1, ground: 0.26, ground_fall: 0.12, ground_lit: 0.05, ..common
        }
    } else {
        Look {
            horizon: 0.02, horizon_soft: 0.04, sky: 0.8, sky_glow: 0.2, band: 0.22, band_width: 0.05, ground: 0.14, ground_far: 0.22,
            front: 0.52, gold: &MIRROR_GOLD, copper: &MIRROR_COPPER, ..common
        }
    }
}

/// Each numeric key: lowest, highest, and the one look it belongs to (`None`: every look).
pub const KEYS: &[(&str, f64, f64, Option<&str>)] = &[
    ("light", 0.0, 360.0, None), ("eye", 0.3, 3.0, None), ("relief", 0.0, 1.0, None), ("cushion", 0.0, 0.5, None),
    ("facet", 0.0, 1.0, None), ("highlight", 0.0, 1.0, None), ("highlight_size", 0.01, 0.4, None), ("grain", 0.0, 0.2, None),
    ("grain_scale", 0.25, 4.0, None), ("sparkle", 0.0, 40.0, None), ("sparkle_glint", 0.0, 1.0, None), ("shadow", 0.0, 1.0, None),
    ("shadow_offset", 0.0, 3.0, None), ("horizon", -0.5, 0.5, None), ("horizon_soft", 0.005, 0.3, None), ("sky", 0.0, 1.0, None),
    ("sky_glow", 0.0, 0.6, None), ("ground", 0.0, 0.8, None), ("horizon_dip", 0.0, 0.6, Some("studio")), ("sky_lit", 0.0, 0.5, Some("studio")),
    ("sky_far", 0.0, 0.5, Some("studio")), ("window", 0.0, 0.6, Some("studio")), ("window_top", 0.2, 1.0, Some("studio")),
    ("window_foot", 0.0, 0.5, Some("studio")), ("ground_fall", 0.0, 0.5, Some("studio")), ("ground_lit", 0.0, 0.3, Some("studio")),
    ("band", 0.0, 0.6, Some("mirror")), ("band_width", 0.01, 0.3, Some("mirror")), ("ground_far", 0.0, 0.6, Some("mirror")), ("front", 0.0, 1.0, Some("mirror")),
];

impl Look {
    fn set(&mut self, k: &str, v: f64) {
        let f = match k {
            "light" => &mut self.light, "eye" => &mut self.eye, "relief" => &mut self.relief, "cushion" => &mut self.cushion,
            "facet" => &mut self.facet, "highlight" => &mut self.highlight, "highlight_size" => &mut self.highlight_size,
            "grain" => &mut self.grain, "grain_scale" => &mut self.grain_scale, "sparkle" => &mut self.sparkle,
            "sparkle_glint" => &mut self.sparkle_glint, "shadow" => &mut self.shadow, "shadow_offset" => &mut self.shadow_offset,
            "horizon" => &mut self.horizon, "horizon_soft" => &mut self.horizon_soft, "sky" => &mut self.sky, "sky_glow" => &mut self.sky_glow,
            "ground" => &mut self.ground, "horizon_dip" => &mut self.horizon_dip, "sky_lit" => &mut self.sky_lit, "sky_far" => &mut self.sky_far,
            "window" => &mut self.window, "window_top" => &mut self.window_top, "window_foot" => &mut self.window_foot,
            "ground_fall" => &mut self.ground_fall, "ground_lit" => &mut self.ground_lit, "band" => &mut self.band,
            "band_width" => &mut self.band_width, "ground_far" => &mut self.ground_far, "front" => &mut self.front,
            _ => return,
        };
        *f = v;
    }
}

pub fn resolve(name: &str, over: &[(String, f64)]) -> Result<Look, String> {
    let mut l = base(name != "mirror");
    for (k, v) in over {
        let key = KEYS.iter().find(|e| e.0 == k).ok_or_else(|| format!("giltlook: no key \"{}\"", k))?;
        if let Some(only) = key.3 {
            if only != name {
                return Err(format!("giltlook: {} belongs to the {} look, not {}", k, only, name));
            }
        }
        l.set(k, *v);
    }
    if l.studio && l.window_foot >= l.window_top {
        return Err(format!("giltlook: window_foot ({}) must be below window_top ({})", l.window_foot, l.window_top));
    }
    Ok(l)
}
