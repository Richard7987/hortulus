//! Document settings: a key=value list, parsed and checked so mistakes name the key.

use crate::color::{group_of, ramp_of};
use crate::gilt_look;
use crate::inks::COLOURWAYS;
use crate::mix::{Palette, Weights, VISITORS};
use crate::ornaments;
use crate::tune::{MIX_DEFAULTS, PALETTE_DEFAULTS};

#[derive(Default, Clone)]
pub struct Settings {
    pub layout: Option<String>,
    pub seed: Option<f64>,
    pub mix: Option<Weights>,
    pub ornaments: Option<Weights>,
    pub palette: Option<Palette>,
    pub spin: Option<f64>,
    pub gilt: Option<String>,
    pub giltdpi: Option<f64>,
    pub giltenv: Option<String>,
    pub giltlook: Vec<(String, f64)>,
    pub growth: Option<String>,
}

fn weight(v: &str) -> Option<f64> {
    v.parse::<f64>().ok().filter(|n| *n >= 0.0 && *n <= 10.0)
}

fn known(keys: impl Iterator<Item = String>) -> String {
    let mut k: Vec<String> = keys.collect();
    k.sort();
    k.join(", ")
}

fn parse_list(s: &str) -> Result<Vec<(String, Option<String>)>, String> {
    let mut out = Vec::new();
    let take = |piece: &str, out: &mut Vec<(String, Option<String>)>| {
        let piece = piece.trim();
        if piece.is_empty() {
            return;
        }
        match piece.find('=') {
            None => out.push((piece.to_string(), None)),
            Some(i) => {
                let (k, v) = (piece[..i].trim(), piece[i + 1..].trim());
                let v = if v.starts_with('{') && v.ends_with('}') && v.len() >= 2 { v[1..v.len() - 1].trim() } else { v };
                out.push((k.to_string(), Some(v.to_string())));
            }
        }
    };
    let (mut depth, mut start) = (0i64, 0usize);
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    return Err(format!("unbalanced braces in \"{}\"", s));
                }
            }
            ',' if depth == 0 => {
                take(&s[start..i], &mut out);
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(format!("unbalanced braces in \"{}\"", s));
    }
    take(&s[start..], &mut out);
    Ok(out)
}

fn weights(v: &str, is_kind: impl Fn(&str) -> bool, what: &str, kinds: impl Fn() -> String) -> Result<Weights, String> {
    let mut t = Weights::new();
    for (k, val) in parse_list(v)? {
        if !is_kind(&k) {
            return Err(format!("no {} \"{}\" ({}: {})", what, k, if what == "kind" { "kinds" } else { "kinds" }, kinds()));
        }
        let w = val.as_deref().and_then(weight).ok_or_else(|| format!("{} needs a weight from 0 to 10", k))?;
        t.insert(k, w);
    }
    Ok(t)
}

const LAYOUTS: [&str; 6] = ["left", "right", "centered", "split", "title", "surround"];
const COLOURWAY_ORDER: [&str; 8] = ["original", "spring", "summer", "autumn", "winter", "ember", "plum", "rose"];

pub fn parse(s: &str) -> Result<Settings, String> {
    let mut t = Settings::default();
    for (k, v) in parse_list(s)? {
        let known_keys = "colourway, density, gilt, giltdpi, giltenv, giltlook, growth, layout, mix, ornaments, palette, seed, spin, visitors";
        if !known_keys.split(", ").any(|x| x == k) {
            return Err(format!("unknown key \"{}\" (known keys: {})", k, known_keys));
        }
        let v = v.ok_or_else(|| format!("key \"{}\" needs a value", k))?;
        let wrap = |msg: String| format!("{}: {} (got \"{}\")", k, msg, v);
        match k.as_str() {
            "layout" => {
                if !LAYOUTS.contains(&v.as_str()) {
                    return Err(wrap("layout must be one of left, right, centered, split, title, surround".into()));
                }
                t.layout = Some(v.clone());
            }
            "growth" => {
                if v != "classic" && v != "natural" {
                    return Err(wrap("growth must be classic or natural".into()));
                }
                t.growth = Some(v.clone());
            }
            "seed" => {
                t.seed = Some(v.parse::<f64>().map_err(|_| wrap("seed must be a number".into()))?);
            }
            "density" => {
                let n = weight(&v).ok_or_else(|| wrap("density must be a number from 0 to 10".into()))?;
                t.mix.get_or_insert_with(Weights::new).insert("density".into(), n);
            }
            "visitors" => {
                let n = weight(&v).ok_or_else(|| wrap("visitors must be a number from 0 to 10".into()))?;
                let m = t.mix.get_or_insert_with(Weights::new);
                for name in VISITORS {
                    m.insert(name.into(), n);
                }
            }
            "mix" => {
                let w = weights(&v, |x| MIX_DEFAULTS.iter().any(|e| e.0 == x), "kind", || known(MIX_DEFAULTS.iter().map(|e| e.0.to_string()))).map_err(wrap)?;
                t.mix.get_or_insert_with(Weights::new).extend(w);
            }
            "ornaments" => {
                let w = weights(&v, ornaments::is_kind, "ornament", || known(ornaments::all().map(|e| e.name.to_string()))).map_err(wrap)?;
                t.ornaments.get_or_insert_with(Weights::new).extend(w);
            }
            "gilt" => {
                if v != "vector" && v != "raster" {
                    return Err(wrap("gilt must be vector or raster".into()));
                }
                t.gilt = Some(v.clone());
            }
            "giltdpi" => {
                let n = v.parse::<f64>().ok().filter(|n| *n >= 50.0 && *n <= 1200.0).ok_or_else(|| wrap("giltdpi must be a number from 50 to 1200".into()))?;
                t.giltdpi = Some(n);
            }
            "giltenv" => {
                if v != "studio" && v != "mirror" {
                    return Err(wrap("giltenv must be studio or mirror".into()));
                }
                t.giltenv = Some(v.clone());
            }
            "giltlook" => {
                for (lk, lv) in parse_list(&v).map_err(&wrap)? {
                    let r = gilt_look::KEYS.iter().find(|e| e.0 == lk).ok_or_else(|| wrap(format!("no gilt look key \"{}\" (keys: {})", lk, known(gilt_look::KEYS.iter().map(|e| e.0.to_string())))))?;
                    let n = lv.as_deref().and_then(|x| x.parse::<f64>().ok()).filter(|n| *n >= r.1 && *n <= r.2).ok_or_else(|| wrap(format!("{} must be a number from {} to {}", lk, r.1, r.2)))?;
                    t.giltlook.retain(|e| e.0 != lk);
                    t.giltlook.push((lk, n));
                }
            }
            "palette" => {
                for (part, ink) in parse_list(&v).map_err(&wrap)? {
                    if !PALETTE_DEFAULTS.iter().any(|e| e.0 == part) {
                        return Err(wrap(format!("no part \"{}\" (parts: {})", part, known(PALETTE_DEFAULTS.iter().map(|e| e.0.to_string())))));
                    }
                    let ink = ink.unwrap_or_default();
                    if !(ink == "look" || ink == "random" || group_of(&ink).is_some() || ramp_of(&ink).is_some()) {
                        return Err(wrap(format!("\"{}\" is not look, random, an ink group or an ink name", ink)));
                    }
                    t.palette.get_or_insert_with(Palette::new).insert(part, ink);
                }
            }
            "colourway" => {
                let name = if v == "fall" { "autumn" } else { v.as_str() };
                let src: Vec<(&str, &str)> = if name == "original" {
                    PALETTE_DEFAULTS.to_vec()
                } else {
                    COLOURWAYS.iter().find(|e| e.0 == name).map(|e| e.1.to_vec()).ok_or_else(|| wrap(format!("colourway must be one of {}", COLOURWAY_ORDER.join(", "))))?
                };
                let m = t.palette.get_or_insert_with(Palette::new);
                for (part, def) in PALETTE_DEFAULTS {
                    let val = src.iter().find(|e| e.0 == *part).map_or(*def, |e| e.1);
                    m.insert(part.to_string(), val.to_string());
                }
            }
            "spin" => {
                let n = v.parse::<f64>().ok().filter(|n| *n == n.floor() && *n >= 0.0).ok_or_else(|| wrap("spin must be a whole number".into()))?;
                t.spin = Some(n);
            }
            _ => {}
        }
    }
    gilt_look::resolve(t.giltenv.as_deref().unwrap_or("studio"), &t.giltlook)?;
    Ok(t)
}
