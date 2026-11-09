//! Oracle driver: paints a page natively for comparison with the Lua generator.
//! usage: native <spec file> <settings> <out prefix> [compact]

use std::fmt::Write as _;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let spec = std::fs::read_to_string(&a[1]).unwrap();
    let t = std::time::Instant::now();
    let (pt, w, h) = hortulus::page(&spec, &a[2], a.get(4).is_some_and(|s| s == "compact")).unwrap();
    let dt = t.elapsed().as_secs_f64();
    std::fs::write(format!("{}.body", a[3]), &pt.out).unwrap();
    let mut r = String::new();
    for g in &pt.gs_list {
        writeln!(r, "gs {} {}", g.0, g.1).unwrap();
    }
    for p in &pt.patterns {
        writeln!(r, "pat {} {}", p.0, p.1).unwrap();
    }
    for (n, im) in &pt.xobjects {
        let sum = |b: &[u8]| b.iter().fold(0u64, |s, x| (s * 31 + *x as u64) % 1000000007);
        writeln!(r, "img {} {} {} {} {}", n, im.w, im.h, sum(&im.rgb), sum(&im.alpha)).unwrap();
    }
    std::fs::write(format!("{}.res", a[3]), r).unwrap();
    std::fs::write(format!("{}.pdf", a[3]), hortulus::pdfout::encode(&pt, w, h)).unwrap();
    println!("native {:.3}s, {} objetos, {} bytes", dt, pt.count, pt.out.len());
}
