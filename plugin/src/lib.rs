//! Procedural watercolour margins for Typst: a port of tikz-flora's generator.

mod branch;
mod brush;
mod color;
pub mod compose;
mod dist;
mod fill;
mod flower;
mod frames;
mod geom;
mod gilt_look;
mod grow;
mod inks;
mod layout;
mod leaf;
mod lua;
mod mix;
mod orn_gild;
mod orn_kinds;
mod orn_kit;
mod orn_pen;
mod orn_raster;
mod orn_relief;
mod orn_title;
mod ornament;
mod ornaments;
pub mod paint;
pub mod pdfout;
mod plant;
mod raster;
mod raster_relief;
mod rng;
pub mod settings;
mod space;
mod sprig;
mod tune;
mod visitor;
mod wash;

use compose::{Bx, Spec};
use paint::Paint;

/// The page as text: width height x0 y0 x1 y1 header page (title x0 y0 x1 y1),
/// then one x0 y0 x1 y1 per text line, in TeX points; `nan` for "none".
pub fn page(spec_text: &str, settings_text: &str, compact: bool) -> Result<(Paint, f64, f64), String> {
    let n: Vec<f64> = spec_text.split_ascii_whitespace().map(|s| s.parse::<f64>().map_err(|_| format!("bad number \"{}\"", s))).collect::<Result<_, _>>()?;
    if n.len() < 12 {
        return Err("page spec is too short".into());
    }
    let set = settings::parse(settings_text)?;
    let mut lines = Vec::new();
    for q in n[12..].chunks_exact(4) {
        lines.push(Bx { x0: q[0], y0: q[1], x1: q[2], y1: q[3] });
    }
    let page = n[7] as i64;
    let base = set.seed.unwrap_or(1.0);
    let spec = Spec {
        width: n[0],
        height: n[1],
        text: Bx { x0: n[2], y0: n[3], x1: n[4], y1: n[5] },
        lines: Some(lines),
        header_y: if n[6].is_nan() { None } else { Some(n[6]) },
        seed: base * 7919.0 + page as f64 * 104729.0,
        page,
        edge: 16.0,
        layout: set.layout,
        title: if n[8].is_nan() { None } else { Some(Bx { x0: n[8], y0: n[9], x1: n[10], y1: n[11] }) },
        frame_only: false,
        closed: Vec::new(),
        mix: set.mix,
        ornaments: set.ornaments,
        palette: set.palette,
        spin: set.spin.unwrap_or(0.0),
        gilt_vector: set.gilt.as_deref() == Some("vector"),
        gilt_dpi: set.giltdpi,
        gilt_env: set.giltenv,
        gilt_look: set.giltlook,
        compact,
        natural: set.growth.as_deref() == Some("natural"),
    };
    let pt = compose::page(&spec)?;
    Ok((pt, spec.width, spec.height))
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[link(wasm_import_module = "typst_env")]
    extern "C" {
        fn wasm_minimal_protocol_send_result_to_host(ptr: *const u8, len: usize);
        fn wasm_minimal_protocol_write_args_to_buffer(ptr: *mut u8);
    }

    fn reply(r: Result<Vec<u8>, String>) -> i32 {
        match r {
            Ok(b) => {
                unsafe { wasm_minimal_protocol_send_result_to_host(b.as_ptr(), b.len()) };
                0
            }
            Err(e) => {
                unsafe { wasm_minimal_protocol_send_result_to_host(e.as_ptr(), e.len()) };
                1
            }
        }
    }

    /// Paint one page: returns its flora as the bytes of a PDF.
    #[no_mangle]
    pub extern "C" fn page(spec_len: usize, set_len: usize) -> i32 {
        let mut buf = vec![0u8; spec_len + set_len];
        unsafe { wasm_minimal_protocol_write_args_to_buffer(buf.as_mut_ptr()) };
        let run = || -> Result<Vec<u8>, String> {
            let spec = std::str::from_utf8(&buf[..spec_len]).map_err(|e| e.to_string())?;
            let set = std::str::from_utf8(&buf[spec_len..]).map_err(|e| e.to_string())?;
            let (pt, w, h) = super::page(spec, set, true)?;
            Ok(super::pdfout::encode(&pt, w, h))
        };
        reply(run())
    }

}
