//! Paints flora as PDF content operators, with the resources they use.

use crate::color::Rgb;
use crate::geom::Cmd;
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;

pub const BP: f64 = 72.0 / 72.27;

pub const PAPER: Rgb = [0.976, 0.972, 0.955];
pub const WHITE: Rgb = [0.995, 0.993, 0.985];
pub const INK: Rgb = [0.050, 0.105, 0.260];
pub const DEEP: Rgb = [0.045, 0.125, 0.470];
pub const BLUE: Rgb = [0.080, 0.280, 0.780];
pub const SKY: Rgb = [0.300, 0.470, 0.830];
pub const MIST: Rgb = [0.780, 0.850, 0.950];
pub const GOLD: Rgb = [0.700, 0.560, 0.340];
pub const COPPER: Rgb = [0.620, 0.330, 0.180];
pub const CREAM: Rgb = [0.960, 0.900, 0.760];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Blend {
    Multiply,
    Normal,
    Screen,
}

impl Blend {
    fn name(self) -> &'static str {
        match self {
            Blend::Multiply => "Multiply",
            Blend::Normal => "Normal",
            Blend::Screen => "Screen",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Opts {
    pub blend: Option<Blend>,
    pub evenodd: bool,
}

pub const NONE: Opts = Opts { blend: None, evenodd: false };
pub const MULT: Opts = Opts { blend: Some(Blend::Multiply), evenodd: false };
pub const EVENODD: Opts = Opts { blend: None, evenodd: true };

pub type Map = Rc<dyn Fn(f64, f64) -> (f64, f64)>;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<u8>,
    pub alpha: Vec<u8>,
}

#[derive(Default)]
pub struct Paint {
    pub out: String,
    pub count: usize,
    pub gilt_vector: bool,
    pub gilt_dpi: f64,
    pub gilt_env: Option<String>,
    pub gilt_look: Vec<(String, f64)>,
    /// Coordinates to a tenth of a point, and small dots as polygons.
    pub compact: bool,
    pub gilding: bool,
    pub flat_gilt: bool,
    pub gilt_pending: Option<crate::orn_raster::Pending>,
    pub ramps: usize,
    pub coverage: f64,
    pub projection: Option<Map>,
    pub frame_fn: Option<Map>,
    gs_names: HashMap<String, String>,
    pub gs_list: Vec<(String, String)>,
    pub patterns: Vec<(String, String)>,
    pub xobjects: Vec<(String, Image)>,
    buf: String,
}

/// `{:.2}`, far cheaper than the generic formatter under an interpreter. It takes
/// the generic path when the value lies too near a rounding tie to be sure.
#[inline]
fn f2(s: &mut String, v: f64) {
    let a = v.abs();
    if a < 4e7 {
        let scaled = a * 100.0;
        let floor = scaled.floor();
        let frac = scaled - floor;
        if (frac - 0.5).abs() > 1e-6 {
            let n = floor as u32 + (frac > 0.5) as u32;
            let (mut whole, cents) = (n / 100, n % 100);
            let mut digits = [0u8; 13];
            let mut i = 10;
            digits[10] = b'.';
            digits[11] = b'0' + (cents / 10) as u8;
            digits[12] = b'0' + (cents % 10) as u8;
            loop {
                i -= 1;
                digits[i] = b'0' + (whole % 10) as u8;
                whole /= 10;
                if whole == 0 {
                    break;
                }
            }
            if v.is_sign_negative() {
                i -= 1;
                digits[i] = b'-';
            }
            // ASCII digits only.
            unsafe { s.as_mut_vec() }.extend_from_slice(&digits[i..]);
            return;
        }
    }
    write!(s, "{:.2}", v).unwrap();
}

/// A tenth of a point, without a trailing `.0`.
#[inline]
fn f1(s: &mut String, v: f64) {
    let a = v.abs();
    if !(a < 9999.95) {
        write!(s, "{:.1}", v).unwrap();
        return;
    }
    let n = (a * 10.0 + 0.5) as u32;
    let (whole, tenth) = (n / 10, n % 10);
    let mut d = [0u8; 8];
    let mut i = 0;
    if v < 0.0 && n != 0 {
        d[0] = b'-';
        i = 1;
    }
    for div in [1000, 100, 10] {
        if whole >= div {
            d[i] = b'0' + (whole / div % 10) as u8;
            i += 1;
        }
    }
    d[i] = b'0' + (whole % 10) as u8;
    i += 1;
    if tenth != 0 {
        d[i] = b'.';
        d[i + 1] = b'0' + tenth as u8;
        i += 2;
    }
    // ASCII digits only.
    unsafe { s.as_mut_vec() }.extend_from_slice(&d[..i]);
}

#[inline]
fn n2(s: &mut String, v: f64) {
    let start = s.len();
    f2(s, v);
    if &s[start..] == "-0.00" {
        s.truncate(start);
        s.push_str("0.00");
    }
}

fn n2s(v: f64) -> String {
    let mut s = String::new();
    n2(&mut s, v);
    s
}

#[inline]
fn b2(s: &mut String, v: f64) {
    n2(s, v * BP);
}

fn rgb3(s: &mut String, c: Rgb) {
    write!(s, "{:.3} {:.3} {:.3}", c[0], c[1], c[2]).unwrap();
}

pub fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[inline(always)]
fn num<const C: bool>(s: &mut String, v: f64) {
    if C { f1(s, v) } else { f2(s, v) }
}

#[inline(always)]
fn space(s: &mut String) {
    // ASCII only.
    unsafe { s.as_mut_vec() }.push(b' ');
}

#[inline]
fn op_m<const C: bool>(buf: &mut String, x: f64, y: f64) {
    num::<C>(buf, x * BP);
    space(buf);
    num::<C>(buf, y * BP);
    buf.push_str(" m");
}

#[inline]
fn op_l<const C: bool>(buf: &mut String, x: f64, y: f64) {
    num::<C>(buf, x * BP);
    space(buf);
    num::<C>(buf, y * BP);
    buf.push_str(" l");
}

#[inline]
fn op_c<const C: bool>(buf: &mut String, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
    for v in [a, b, c, d, e, f] {
        num::<C>(buf, v * BP);
        space(buf);
    }
    buf.push('c');
}

const K: f64 = 0.5523;

/// Small dots, in big points of radius, print as polygons of the same area: a square
/// below `SPECK`, an octagon below `DOT`. Neither strays from the circle by more than
/// 0.03 mm, under a printer pixel, and both cost far fewer bytes than four curves.
const SPECK: f64 = 0.35;
const DOT: f64 = 1.0;

fn dot(buf: &mut String, x: f64, y: f64, r: f64) {
    let (x, y, r) = (x * BP, y * BP, r * BP);
    space(buf);
    if r < SPECK {
        let s = r * 1.7725;
        f1(buf, x - s / 2.0);
        space(buf);
        f1(buf, y - s / 2.0);
        space(buf);
        f2(buf, s);
        space(buf);
        f2(buf, s);
        buf.push_str(" re");
        return;
    }
    let r = r * 1.0539;
    for (k, (c, s)) in OCTAGON.iter().enumerate() {
        if k > 0 {
            space(buf);
        }
        f2(buf, x + r * c);
        space(buf);
        f2(buf, y + r * s);
        buf.push_str(if k == 0 { " m" } else { " l" });
    }
    buf.push_str(" h");
}

const OCTAGON: [(f64, f64); 8] = [
    (0.9239, 0.3827),
    (0.3827, 0.9239),
    (-0.3827, 0.9239),
    (-0.9239, 0.3827),
    (-0.9239, -0.3827),
    (-0.3827, -0.9239),
    (0.3827, -0.9239),
    (0.9239, -0.3827),
];

impl Paint {
    pub fn new(gilt_vector: bool) -> Paint {
        Paint { gilt_vector, gilt_dpi: crate::tune::tune().gilt_dpi, ..Default::default() }
    }

    pub fn gstate(&mut self, ca: f64, ca2: f64, bm: Blend) -> String {
        let (a, b) = (n2s(ca), n2s(ca2));
        let key = format!("{}/{}/{}", a, b, bm.name());
        if let Some(n) = self.gs_names.get(&key) {
            return n.clone();
        }
        let name = format!("flgs{}", self.gs_names.len() + 1);
        self.gs_names.insert(key, name.clone());
        self.gs_list.push((name.clone(), format!("<< /ca {} /CA {} /BM /{} >>", a, b, bm.name())));
        name
    }

    /// Any paint flushes raster gilt collected before it, so the gold keeps its
    /// place in the painting order.
    pub fn emit(&mut self, s: &str) {
        if self.gilt_pending.is_some() {
            crate::orn_raster::flush(self);
        }
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        self.out.push_str(s);
    }

    pub fn project(&mut self, f: Option<Map>) {
        self.projection = f;
    }

    pub fn frame(&mut self, f: Option<Map>) {
        self.frame_fn = f;
    }

    pub fn projection(&self) -> Option<Map> {
        self.projection.clone()
    }

    #[inline]
    pub fn pj(&self, x: f64, y: f64) -> (f64, f64) {
        let (mut x, mut y) = (x, y);
        if let Some(p) = &self.projection {
            let r = p(x, y);
            x = r.0;
            y = r.1;
        }
        if let Some(f) = &self.frame_fn {
            let r = f(x, y);
            x = r.0;
            y = r.1;
        }
        (x, y)
    }

    pub fn has_pj(&self) -> bool {
        self.projection.is_some() || self.frame_fn.is_some()
    }

    pub fn projected(&self, cmds: &[Cmd]) -> Vec<Cmd> {
        let mut out = Vec::with_capacity(cmds.len());
        for c in cmds {
            match *c {
                Cmd::M(x, y) => {
                    let (a, b) = self.pj(x, y);
                    out.push(Cmd::M(a, b));
                }
                Cmd::L(x, y) => {
                    let (a, b) = self.pj(x, y);
                    out.push(Cmd::L(a, b));
                }
                Cmd::C(a, b, c2, d, e, f) => {
                    let (a1, a2) = self.pj(a, b);
                    let (a3, a4) = self.pj(c2, d);
                    let (a5, a6) = self.pj(e, f);
                    out.push(Cmd::C(a1, a2, a3, a4, a5, a6));
                }
                Cmd::O(x, y, r) => {
                    let k = r * K;
                    let (mx, my) = self.pj(x + r, y);
                    out.push(Cmd::M(mx, my));
                    for arc in [
                        [x + r, y + k, x + k, y + r, x, y + r],
                        [x - k, y + r, x - r, y + k, x - r, y],
                        [x - r, y - k, x - k, y - r, x, y - r],
                        [x + k, y - r, x + r, y - k, x + r, y],
                    ] {
                        let (a1, a2) = self.pj(arc[0], arc[1]);
                        let (a3, a4) = self.pj(arc[2], arc[3]);
                        let (a5, a6) = self.pj(arc[4], arc[5]);
                        out.push(Cmd::C(a1, a2, a3, a4, a5, a6));
                    }
                    out.push(Cmd::Z);
                }
                Cmd::Z => out.push(Cmd::Z),
            }
        }
        out
    }

    fn path(&self, cmds: &[Cmd], buf: &mut String) {
        if self.compact && cmds.iter().any(|c| matches!(c, Cmd::O(..))) {
            for c in cmds {
                match *c {
                    Cmd::O(x, y, r) if r * BP < DOT => {
                        let (cx, cy) = self.pj(x, y);
                        let (ex, ey) = self.pj(x + r, y);
                        dot(buf, cx, cy, ((ex - cx).powi(2) + (ey - cy).powi(2)).sqrt());
                    }
                    _ if self.has_pj() => write_path::<true>(&self.projected(&[*c]), buf),
                    _ => write_path::<true>(&[*c], buf),
                }
            }
            return;
        }
        if self.has_pj() {
            let p = self.projected(cmds);
            if self.compact { write_path::<true>(&p, buf) } else { write_path::<false>(&p, buf) }
        } else {
            if self.compact { write_path::<true>(cmds, buf) } else { write_path::<false>(cmds, buf) }
        }
    }

    fn is_gilt(&self, rgb: Rgb, opacity: f64, o: Opts) -> bool {
        self.gilding && !self.flat_gilt && (rgb == GOLD || rgb == COPPER) && opacity >= 0.5 && o.blend.is_none()
    }

    pub fn fill(&mut self, cmds: &[Cmd], rgb: Rgb, opacity: f64, o: Opts) {
        if self.is_gilt(rgb, opacity, o) {
            self.gilding = false;
            crate::orn_gild::fill(self, &[cmds.to_vec()], rgb == COPPER);
            self.gilding = true;
            return;
        }
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        self.fill_head(&mut buf, rgb, opacity, o);
        self.path(cmds, &mut buf);
        self.fill_tail(buf, o);
    }

    pub fn fill_many(&mut self, paths: &[Vec<Cmd>], rgb: Rgb, opacity: f64, o: Opts) {
        if self.is_gilt(rgb, opacity, o) {
            self.gilding = false;
            crate::orn_gild::fill(self, paths, rgb == COPPER);
            self.gilding = true;
            return;
        }
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        self.fill_head(&mut buf, rgb, opacity, o);
        for c in paths {
            self.path(c, &mut buf);
        }
        self.fill_tail(buf, o);
    }

    fn fill_head(&mut self, buf: &mut String, rgb: Rgb, opacity: f64, o: Opts) {
        buf.push('q');
        let op = if opacity < 1.0 { opacity } else { 1.0 };
        if op < 1.0 || o.blend.is_some() {
            let g = self.gstate(op, 1.0, o.blend.unwrap_or(Blend::Normal));
            buf.push_str(" /");
            buf.push_str(&g);
            buf.push_str(" gs");
        }
        buf.push(' ');
        rgb3(buf, rgb);
        buf.push_str(" rg");
    }

    fn fill_tail(&mut self, mut buf: String, o: Opts) {
        buf.push_str(if o.evenodd { " f* Q" } else { " f Q" });
        self.emit(&buf);
        self.buf = buf;
        self.count += 1;
    }

    pub fn stroke(&mut self, cmds: &[Cmd], rgb: Rgb, width: f64, opacity: f64, o: Opts) {
        if rgb == GOLD && self.is_gilt(rgb, opacity, o) {
            self.gilding = false;
            crate::orn_gild::line(self, &[cmds.to_vec()], width);
            self.gilding = true;
            return;
        }
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        self.stroke_head(&mut buf, rgb, width, opacity, o);
        self.path(cmds, &mut buf);
        self.stroke_tail(buf);
    }

    pub fn stroke_many(&mut self, paths: &[Vec<Cmd>], rgb: Rgb, width: f64, opacity: f64, o: Opts) {
        if rgb == GOLD && self.is_gilt(rgb, opacity, o) {
            self.gilding = false;
            crate::orn_gild::line(self, paths, width);
            self.gilding = true;
            return;
        }
        let mut buf = std::mem::take(&mut self.buf);
        buf.clear();
        self.stroke_head(&mut buf, rgb, width, opacity, o);
        for c in paths {
            self.path(c, &mut buf);
        }
        self.stroke_tail(buf);
    }

    fn stroke_head(&mut self, buf: &mut String, rgb: Rgb, width: f64, opacity: f64, o: Opts) {
        buf.push('q');
        let op = if opacity < 1.0 { opacity } else { 1.0 };
        if op < 1.0 || o.blend.is_some() {
            let g = self.gstate(1.0, op, o.blend.unwrap_or(Blend::Normal));
            buf.push_str(" /");
            buf.push_str(&g);
            buf.push_str(" gs");
        }
        buf.push(' ');
        rgb3(buf, rgb);
        buf.push_str(" RG ");
        b2(buf, width);
        buf.push_str(" w 1 J 1 j");
    }

    fn stroke_tail(&mut self, mut buf: String) {
        buf.push_str(" S Q");
        self.emit(&buf);
        self.buf = buf;
        self.count += 1;
    }

    /// Paint a gradient so its start lies at p0 and its end at p1, covering
    /// `half` pt to either side of that axis. Use inside a clip.
    pub fn shade(&mut self, stops: &[(f64, Rgb)], p0: [f64; 2], p1: [f64; 2], half: f64) {
        let (mut p0, mut p1) = (p0, p1);
        if self.has_pj() {
            let (a, b) = self.pj(p0[0], p0[1]);
            let (c, d) = self.pj(p1[0], p1[1]);
            p0 = [a, b];
            p1 = [c, d];
        }
        let (dx, dy) = (p1[0] - p0[0], p1[1] - p0[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.01 {
            return;
        }
        let pname = format!("flpat{}", self.patterns.len() + 1);
        let mut src = String::from("<< /PatternType 2 /Shading << /ShadingType 2 /ColorSpace /DeviceRGB /Coords [");
        for (i, v) in [p0[0], p0[1], p1[0], p1[1]].iter().enumerate() {
            if i > 0 {
                src.push(' ');
            }
            b2(&mut src, *v);
        }
        src.push_str("] /Function ");
        stitch(&mut src, stops);
        src.push_str(" /Extend [true true] >> >>");
        self.patterns.push((pname.clone(), src));
        let (ux, uy) = (dx / len, dy / len);
        let (nx, ny) = (-uy * half, ux * half);
        let ext = len * 0.5;
        let a = [p0[0] - ux * ext, p0[1] - uy * ext];
        let b = [p1[0] + ux * ext, p1[1] + uy * ext];
        let mut s = String::with_capacity(96);
        write!(s, "q /Pattern cs /{} scn ", pname).unwrap();
        b2(&mut s, a[0] + nx);
        s.push(' ');
        b2(&mut s, a[1] + ny);
        s.push_str(" m ");
        b2(&mut s, b[0] + nx);
        s.push(' ');
        b2(&mut s, b[1] + ny);
        s.push_str(" l ");
        b2(&mut s, b[0] - nx);
        s.push(' ');
        b2(&mut s, b[1] - ny);
        s.push_str(" l ");
        b2(&mut s, a[0] - nx);
        s.push(' ');
        b2(&mut s, a[1] - ny);
        s.push_str(" l h f Q");
        self.emit(&s);
        self.count += 1;
    }

    pub fn clip_begin(&mut self, cmds: &[Cmd]) {
        let mut buf = String::from("q");
        self.path(cmds, &mut buf);
        buf.push_str(" W n");
        self.emit(&buf);
    }

    pub fn clip_end(&mut self) {
        self.emit("Q");
    }

    pub fn group_begin(&mut self, opacity: f64) {
        let g = self.gstate(opacity, opacity, Blend::Normal);
        self.emit(&format!("q /{} gs", g));
    }

    pub fn group_end(&mut self) {
        self.emit("Q");
    }
}

fn write_path<const C: bool>(cmds: &[Cmd], buf: &mut String) {
    for c in cmds {
        space(buf);
        match *c {
            Cmd::M(x, y) => op_m::<C>(buf, x, y),
            Cmd::L(x, y) => op_l::<C>(buf, x, y),
            Cmd::C(a, b, c2, d, e, f) => op_c::<C>(buf, a, b, c2, d, e, f),
            Cmd::Z => buf.push('h'),
            Cmd::O(x, y, r) => {
                let k = r * K;
                op_m::<C>(buf, x + r, y);
                space(buf);
                op_c::<C>(buf, x + r, y + k, x + k, y + r, x, y + r);
                space(buf);
                op_c::<C>(buf, x - k, y + r, x - r, y + k, x - r, y);
                space(buf);
                op_c::<C>(buf, x - r, y - k, x - k, y - r, x, y - r);
                space(buf);
                op_c::<C>(buf, x + k, y - r, x + r, y - k, x + r, y);
                buf.push_str(" h");
            }
        }
    }
}

fn stitch(s: &mut String, stops: &[(f64, Rgb)]) {
    s.push_str("<< /FunctionType 3 /Domain [0 1] /Functions [");
    for i in 0..stops.len() - 1 {
        if i > 0 {
            s.push(' ');
        }
        s.push_str("<< /FunctionType 2 /Domain [0 1] /C0 [");
        rgb3(s, stops[i].1);
        s.push_str("] /C1 [");
        rgb3(s, stops[i + 1].1);
        s.push_str("] /N 1 >>");
    }
    s.push_str("] /Bounds [");
    for i in 1..stops.len() - 1 {
        if i > 1 {
            s.push(' ');
        }
        write!(s, "{:.4}", stops[i].0).unwrap();
    }
    s.push_str("] /Encode [");
    for i in 0..stops.len() - 1 {
        if i > 0 {
            s.push(' ');
        }
        s.push_str("0 1");
    }
    s.push_str("] >>");
}
