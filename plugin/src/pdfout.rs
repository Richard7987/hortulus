//! Writes a painted page as a standalone one-page PDF.

use crate::paint::{Paint, BP};
use std::fmt::Write as _;

fn stream(attr: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< {} /Length {} >>\nstream\n", attr, data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

pub fn encode(pt: &Paint, width: f64, height: f64) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = vec![Vec::new(); 4];
    let gs = pt.gs_list.iter().map(|g| format!("/{} {}", g.0, g.1)).collect::<Vec<_>>().join(" ");
    let mut pats = String::new();
    for (i, p) in pt.patterns.iter().enumerate() {
        objects.push(p.1.clone().into_bytes());
        let _ = write!(pats, "/{} {} 0 R ", p.0, 5 + i);
    }
    let mut xobj = String::new();
    for (name, im) in &pt.xobjects {
        let n = objects.len() + 1;
        let head = format!("/Type /XObject /Subtype /Image /Width {} /Height {} /BitsPerComponent 8", im.w, im.h);
        objects.push(stream(&format!("{} /ColorSpace /DeviceGray", head), &im.alpha));
        objects.push(stream(&format!("{} /ColorSpace /DeviceRGB /SMask {} 0 R", head, n), &im.rgb));
        let _ = write!(xobj, "/{} {} 0 R ", name, n + 1);
    }
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    objects[1] = b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec();
    objects[2] = format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /ExtGState << {} >> /Pattern << {} >> /XObject << {} >> >> /Contents 4 0 R >>",
        width * BP, height * BP, gs, pats, xobj
    )
    .into_bytes();
    objects[3] = stream("", pt.out.as_bytes());
    let total: usize = objects.iter().map(|o| o.len() + 32).sum();
    let mut out = Vec::with_capacity(total + objects.len() * 20 + 128);
    out.extend_from_slice(b"%PDF-1.5\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for o in &offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", o).as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objects.len() + 1, xref).as_bytes());
    out
}
