//! Title-page furniture: crest, flourish, tailpiece and foot corners.

use crate::compose::{Bx, Spec};
use crate::geom as g;
use crate::ornaments::{self, DrawOpts, Kind, Slot};
use crate::paint::Paint;
use crate::rng::Rng;

const PAD: f64 = 10.0;

fn choose(spec: &Spec, slot: Slot) -> Option<&'static Kind> {
    let mut r = Rng::new(spec.seed).fork(&format!("choose title {}", slot.name()));
    ornaments::pick(&mut r, slot, spec.ornaments.as_ref())
}

pub fn crest_size(spec: &Spec, r: f64) -> f64 {
    if choose(spec, Slot::Crest).is_none() {
        return 0.0;
    }
    g::clamp(r * 1.2, 44.0, 84.0)
}

pub struct Item {
    pub slot: Slot,
    pub kind: &'static Kind,
    pub b: [f64; 4],
    pub o: DrawOpts,
}

pub fn plan(spec: &Spec, b: Bx, crest: f64) -> (Vec<Item>, Vec<[f64; 4]>) {
    let (w, h, t) = (spec.width, spec.height, spec.text);
    let cx = (b.x0 + b.x1) / 2.0;
    let (mut out, mut keep) = (Vec::new(), Vec::new());
    let lines: &[Bx] = spec.lines.as_deref().unwrap_or(&[]);
    let mut add = |slot: Slot, bx: [f64; 4], reserve: bool| {
        if let Some(kind) = choose(spec, slot) {
            out.push(Item { slot, kind, b: bx, o: DrawOpts::default() });
            if reserve {
                keep.push(bx);
            }
        }
    };
    if crest > 0.0 {
        add(Slot::Crest, [w / 2.0 - crest / 2.0, h - PAD - crest, w / 2.0 + crest / 2.0, h - PAD], false);
    }
    let mut below: Option<Bx> = None;
    for l in lines {
        if l.y1 < b.y0 && below.map_or(true, |x| l.y1 > x.y1) {
            below = Some(*l);
        }
    }
    let fw = 280f64.min((b.x1 - b.x0) * 1.15).min(t.x1 - t.x0);
    let fh = fw * 46.0 / 280.0;
    let floor = below.map_or(t.y0, |x| x.y1);
    if b.y0 - floor >= fh + 2.0 * PAD {
        let y1 = b.y0 - PAD * 0.6;
        add(Slot::Flourish, [cx - fw / 2.0, y1 - fh, cx + fw / 2.0, y1], true);
    }
    let mut last: Option<Bx> = None;
    for l in lines {
        if last.map_or(true, |x| l.y0 < x.y0) {
            last = Some(*l);
        }
    }
    if let Some(l) = last {
        if l.y1 < b.y0 {
            let tw = 140f64.min((t.x1 - t.x0) * 0.4);
            let th = tw * 88.0 / 140.0;
            let y1 = l.y0 - PAD * 1.6;
            if y1 - th >= t.y0 {
                let tx = (t.x0 + t.x1) / 2.0;
                add(Slot::Tailpiece, [tx - tw / 2.0, y1 - th, tx + tw / 2.0, y1], true);
            }
        }
    }
    let s = 72f64.min(t.x0 - 24.0).min(w - t.x1 - 24.0).min(t.y0 - 24.0);
    if s >= 28.0 {
        if let Some(kind) = choose(spec, Slot::Corner) {
            out.push(Item { slot: Slot::Corner, kind, b: [16.0, 16.0, 16.0 + s, 16.0 + s], o: DrawOpts { flip: Some([1.0, 1.0]), cord: 0.0 } });
            out.push(Item { slot: Slot::Corner, kind, b: [w - 16.0 - s, 16.0, w - 16.0, 16.0 + s], o: DrawOpts { flip: Some([-1.0, 1.0]), cord: 0.0 } });
        }
    }
    (out, keep)
}

pub fn paint(pt: &mut Paint, spec: &Spec, plan: &[Item]) {
    let rng = Rng::new(spec.seed);
    for (i, p) in plan.iter().enumerate() {
        let label = if p.slot == Slot::Corner { format!("title {}", p.slot.name()) } else { format!("title {}{}", p.slot.name(), i + 1) };
        ornaments::draw(p.kind, pt, &mut rng.fork(&label), p.b, &p.o);
    }
}
