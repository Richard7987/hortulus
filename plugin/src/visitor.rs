//! Visitors to the border: a bee, a butterfly, a songbird and a snail.

use crate::color::{self};
use crate::geom::{self as g, Cmd, Pt};
use crate::grow::Ctx;
use crate::mix;
use crate::paint::{self, Paint, MULT, NONE};
use crate::rng::Rng;
use crate::space::Space;
use crate::wash::{self, WashOpts};
use std::f64::consts::PI;

fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, n: usize) -> Vec<Pt> {
    (0..n)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / n as f64;
            [cx + rx * a.cos(), cy + ry * a.sin()]
        })
        .collect()
}

fn hair(pt: &mut Paint, pts: &[Pt], w: f64, op: f64) {
    pt.stroke(&g::polyline(pts, false), paint::INK, w, op, MULT);
}

fn bee(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, ang: f64, l: f64) {
    let s = l / 13.0;
    let at = |pts: &[Pt]| g::transform(pts, x, y, ang, s, s);
    for w in [[1.5, 4.2, 0.55], [-0.5, 3.6, 0.95]] {
        let wing = at(&g::transform(&ellipse(0.0, 0.0, 4.4, 2.0, 18), w[0], w[1], w[2], 1.0, 1.0));
        let path = g::smooth(&wing, true);
        pt.fill(&path, paint::WHITE, 0.75, NONE);
        pt.fill(&path, paint::SKY, 0.10, MULT);
        pt.stroke(&path, paint::INK, 0.25 * s, 0.55, MULT);
    }
    for k in [-1.0, 0.0, 1.0] {
        hair(pt, &at(&[[1.8 + k * 1.2, -1.6], [1.2 + k * 1.6, -3.4], [0.6 + k * 1.8, -4.2]]), 0.3 * s, 0.85);
    }
    let abd = at(&ellipse(-2.6, 0.0, 4.8, 3.2, 18));
    wash::wash(pt, &abd, rng, WashOpts { size: 6.0 * s, pigment: Some(paint::GOLD), pool: Some(paint::COPPER), simple: true, rings: Some(5), lift: Some(0.12), grain: 0.4, outline: 0.6, outline_w: Some(0.3 * s), ..Default::default() });
    pt.clip_begin(&g::smooth(&abd, true));
    for bx in [-4.6, -2.4, -0.2] {
        pt.stroke(&g::polyline(&at(&[[bx + 0.4, -4.0], [bx - 0.4, 4.0]]), false), paint::INK, 1.1 * s, 0.8, MULT);
    }
    pt.clip_end();
    hair(pt, &at(&[[-7.4, 0.0], [-8.6, -0.3]]), 0.4 * s, 0.85);
    pt.fill(&g::smooth(&at(&ellipse(2.3, 0.0, 2.6, 2.4, 18)), true), paint::INK, 0.85, MULT);
    pt.fill(&g::smooth(&at(&ellipse(5.2, 0.2, 1.7, 1.6, 18)), true), paint::INK, 0.9, MULT);
    hair(pt, &at(&[[6.2, 1.2], [7.6, 2.8], [8.8, 3.1]]), 0.25 * s, 0.85);
    hair(pt, &at(&[[6.3, 0.8], [8.0, 1.8], [9.1, 1.6]]), 0.25 * s, 0.85);
}

fn trail(rng: &mut Rng, x: f64, y: f64, ang: f64, l: f64, s: &Space) -> Vec<Pt> {
    let mut pts = Vec::new();
    let turn = if rng.chance(0.5) { 1.0 } else { -1.0 };
    let loop_at = rng.range(0.35, 0.6);
    let n = 70;
    let back = ang + PI;
    let (mut px, mut py) = (x + back.cos() * l * 0.6, y + back.sin() * l * 0.6);
    let mut h = back + turn * rng.range(-0.3, 0.3);
    for i in 1..=n {
        let u = i as f64 / n as f64;
        let k = 0.04 * (u * 7.0).sin() + if (u - loop_at).abs() < 0.12 { turn * 0.55 } else { 0.0 };
        h += k;
        px += h.cos() * 2.2;
        py += h.sin() * 2.2;
        if !s.free(px, py) {
            break;
        }
        pts.push([px, py]);
    }
    pts
}

const FORE: [Pt; 7] = [[0.0, 1.0], [4.0, 7.0], [10.0, 12.0], [14.0, 12.5], [14.5, 8.5], [10.0, 3.5], [3.0, 0.5]];
const HIND: [Pt; 6] = [[0.0, -0.5], [6.0, -1.5], [10.5, -5.0], [9.0, -10.5], [4.0, -11.0], [1.0, -6.0]];

fn butterfly(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, ang: f64, ss: f64, open: f64) {
    let s = ss / 29.0;
    let rot = ang - PI / 2.0;
    let place = |pts: &[Pt]| g::transform(pts, x, y, rot, s, s);
    let choice = *rng.pick(&["blue", "violet", "teal"]);
    let ink = color::ink_for(rng, color::group_of(choice).map(|_| choice_static(choice)));
    for side in [1.0, -1.0] {
        for (wi, shape) in [(&HIND[..], 1), (&FORE[..], 2)].iter().map(|(sh, i)| (*i, *sh)) {
            let w: Vec<Pt> = shape.iter().map(|p| [side * p[0] * open, p[1]]).collect();
            let poly = place(&w);
            let q = shape[3];
            let tip = place(&[[side * q[0] * open, q[1]]])[0];
            let mut fr = rng.fork(&format!("wing{}{}", side as i64, wi));
            wash::wash(pt, &poly, &mut fr, WashOpts {
                size: 12.0 * s,
                ramp: Some("vivid"),
                part: Some("blooms"),
                axis: Some(([x, y], tip)),
                pigment: Some(ink.map_or(paint::BLUE, |i| color::ink_rgb(i, 0.5, 1.0))),
                pool: Some(paint::DEEP),
                simple: true,
                outline: 0.55,
                outline_w: Some(0.3 * s),
                grain: 0.15,
                ..Default::default()
            });
            let c = place(&[[side * (q[0] - 3.0) * open, q[1] - 1.5]])[0];
            pt.fill(&[Cmd::O(c[0], c[1], 1.1 * s)], paint::WHITE, 0.8, NONE);
        }
    }
    let body = place(&ellipse(0.0, -1.0, 1.1, 7.0, 18));
    pt.fill(&g::smooth(&body, true), paint::INK, 0.85, MULT);
    for side in [1.0, -1.0] {
        let a = place(&[[0.0, 5.5], [side * 2.5, 10.0], [side * 4.0, 12.5]]);
        hair(pt, &a, 0.3 * s, 0.85);
        pt.fill(&[Cmd::O(a[2][0], a[2][1], 0.6 * s)], paint::INK, 0.85, MULT);
    }
}

fn choice_static(name: &str) -> &'static str {
    crate::inks::GROUPS.iter().find(|e| e.0 == name).map_or("look", |e| e.0)
}

const BIRD: [Pt; 13] = [[-17.0, -1.0], [-13.0, 1.5], [-6.0, 6.5], [2.0, 8.5], [8.0, 12.0], [12.0, 12.5], [15.0, 10.0], [14.5, 6.5], [11.0, 3.0], [7.0, -2.5], [0.0, -6.0], [-6.0, -5.5], [-12.0, -2.5]];
const WING: [Pt; 6] = [[-11.0, 1.0], [-4.0, 6.0], [4.0, 6.5], [7.0, 2.5], [0.0, -1.5], [-7.0, -1.0]];

fn bird(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, dir: f64, l: f64) {
    let s = l / 32.0;
    let at = |pts: &[Pt]| -> Vec<Pt> { pts.iter().map(|p| [x + dir * p[0] * s, y + (p[1] + 9.0) * s]).collect() };
    for lx in [-1.5, 1.5] {
        hair(pt, &at(&[[lx, -5.5], [lx + 0.5, -8.0], [lx, -9.0]]), 0.45 * s, 0.85);
        hair(pt, &at(&[[lx - 1.6, -9.0], [lx + 1.8, -9.0]]), 0.4 * s, 0.85);
    }
    let choice = *rng.pick(&["blue", "blue-grey", "teal"]);
    let ink = color::ink_for(rng, Some(choice_static(choice)));
    let body = at(&BIRD);
    wash::wash(pt, &body, &mut rng.fork("body"), WashOpts {
        size: 18.0 * s,
        ramp: Some("soft"),
        part: Some("blooms"),
        axis: Some((at(&[[6.0, -4.0]])[0], at(&[[-4.0, 8.0]])[0])),
        pigment: Some(ink.map_or(paint::BLUE, |i| color::ink_rgb(i, 0.55, 1.0))),
        pool: Some(paint::DEEP),
        simple: true,
        outline: 0.6,
        outline_w: Some(0.35 * s),
        grain: 0.12,
        ..Default::default()
    });
    let wing = at(&WING);
    wash::wash(pt, &wing, &mut rng.fork("wing"), WashOpts {
        size: 12.0 * s,
        ramp: Some("vivid"),
        part: Some("blooms"),
        axis: Some((at(&[[5.0, 3.0]])[0], at(&[[-11.0, 1.0]])[0])),
        pigment: Some(paint::BLUE),
        pool: Some(paint::DEEP),
        simple: true,
        outline: 0.5,
        outline_w: Some(0.3 * s),
        ..Default::default()
    });
    for k in 0..3 {
        let kf = k as f64;
        hair(pt, &at(&[[3.0 - kf * 2.0, 4.0 - kf * 1.2], [-4.0 - kf * 2.0, 3.0 - kf * 1.5], [-9.0 - kf, 0.5 - kf * 0.6]]), 0.2 * s, 0.5);
    }
    let beak = at(&[[14.6, 10.5], [19.5, 9.2], [14.6, 8.0]]);
    pt.fill(&g::polyline(&beak, true), paint::GOLD, 0.9, MULT);
    pt.stroke(&g::polyline(&beak, true), paint::INK, 0.25 * s, 0.8, MULT);
    let eye = at(&[[11.4, 9.6]])[0];
    pt.fill(&[Cmd::O(eye[0], eye[1], 0.95 * s)], paint::INK, 0.95, MULT);
    pt.fill(&[Cmd::O(eye[0] + dir * 0.3 * s, eye[1] + 0.3 * s, 0.3 * s)], paint::WHITE, 0.9, NONE);
}

fn snail(pt: &mut Paint, rng: &mut Rng, x: f64, y: f64, ang: f64, side: f64, l: f64) {
    let s = l / 18.0;
    let at = |pts: &[Pt]| g::transform(pts, x, y, ang, s, s * side);
    let foot = at(&[[-9.0, 0.4], [-4.0, 2.2], [4.0, 2.6], [8.0, 3.4], [10.0, 2.6], [9.5, 0.8], [4.0, 0.2], [-4.0, -0.2]]);
    wash::wash(pt, &foot, &mut rng.fork("foot"), WashOpts { size: 8.0 * s, pigment: Some(paint::MIST), pool: Some(paint::SKY), simple: true, rings: Some(4), lift: Some(0.1), outline: 0.5, outline_w: Some(0.25 * s), ..Default::default() });
    for t in [[9.0, 3.0], [9.8, 2.6]] {
        let tip = [t[0] + 2.4, t[1] + 3.2];
        hair(pt, &at(&[t, [t[0] + 1.2, t[1] + 1.8], tip]), 0.3 * s, 0.85);
        let q = at(&[tip])[0];
        pt.fill(&[Cmd::O(q[0], q[1], 0.45 * s)], paint::INK, 0.85, MULT);
    }
    let choice = *rng.pick(&["earth", "gold", "warm"]);
    let earth = color::ink_for(rng, Some(choice_static(choice)));
    let shell = at(&ellipse(-1.0, 7.0, 6.2, 5.8, 22));
    wash::wash(pt, &shell, &mut rng.fork("shell"), WashOpts {
        size: 9.0 * s,
        ramp: Some("soft"),
        part: Some("leaves"),
        axis: Some((at(&[[1.0, 10.0]])[0], at(&[[-4.0, 3.0]])[0])),
        pigment: Some(earth.map_or(paint::GOLD, |i| color::ink_rgb(i, 0.55, 1.0))),
        pool: Some(paint::COPPER),
        simple: true,
        outline: 0.6,
        outline_w: Some(0.3 * s),
        grain: 0.25,
        ..Default::default()
    });
    let spiral: Vec<Pt> = (0..=40)
        .map(|i| {
            let a = i as f64 / 40.0 * 2.6 * PI;
            let r = 5.4 * (1.0 - i as f64 / 46.0);
            [-1.0 + r * (a + 0.6).cos(), 7.0 + r * (a + 0.6).sin()]
        })
        .collect();
    hair(pt, &at(&spiral), 0.35 * s, 0.7);
}

fn count(rng: &mut Rng, chance: f64, most: i64, w: f64) -> i64 {
    let e = chance * w;
    let mut n = e.floor() as i64;
    if rng.next() < e - n as f64 {
        n += 1;
    }
    n.min(most)
}

fn open_spots(s: &Space, need: f64) -> Vec<[f64; 3]> {
    let (nx, c) = (s.nx, s.c);
    let mut out = Vec::new();
    for k in 0..s.nx * s.ny {
        let f = s.f[k as usize];
        if f >= need {
            out.push([((k % nx) as f64 + 0.5) * c, ((k / nx) as f64 + 0.5) * c, f]);
        }
    }
    out
}

pub fn place(s: &mut Space, rng: &Rng, ctx: &mut Ctx) {
    let spec: [(&str, f64, i64, f64); 4] = [("bee", 0.6, 3, 14.0), ("butterfly", 0.7, 3, 20.0), ("bird", 0.5, 1, 28.0), ("snail", 0.45, 1, 12.0)];
    let mut want = [0i64; 4];
    let mut any = false;
    for (i, name) in mix::VISITORS.iter().enumerate() {
        let w = mix::weight(ctx.mix, name);
        let sp = spec.iter().find(|e| e.0 == *name).unwrap();
        want[i] = if w > 0.0 { count(&mut rng.fork(name), sp.1, sp.2, w) } else { 0 };
        any = any || want[i] > 0;
    }
    if !any {
        return;
    }
    let (want_bee, want_fly, want_bird, want_snail) = (want[0], want[1], want[2], want[3]);
    let main_pts = ctx.strands[0].borrow().pts.clone();

    if want_snail > 0 {
        let mut r = rng.fork("snail");
        for _ in 0..8 {
            let t = r.range(0.15, 0.45);
            let (p, a) = g::at(&main_pts, t);
            let l = r.range(26.0, 31.0);
            let (mut best, mut side) = (-1.0, 1.0);
            for sd in [1.0, -1.0] {
                let q = g::polar(p, a + sd * PI / 2.0, l * 0.45);
                let rm = s.room(q[0], q[1]);
                if rm > best {
                    best = rm;
                    side = sd;
                }
            }
            if best >= 12.0 * 0.6 {
                let c = g::polar(p, a + side * PI / 2.0, l * 0.4);
                s.disc(c[0], c[1], l * 0.55, 1.0);
                let mut fr = r.fork("paint");
                ctx.visitors.push(Box::new(move |pt: &mut Paint| snail(pt, &mut fr, p[0], p[1], a, side, l)));
                break;
            }
        }
    }

    if want_bird > 0 {
        let mut r = rng.fork("bird");
        for tr in 1..=40 {
            let st_pts = if tr <= 15 {
                main_pts.clone()
            } else {
                let i = r.int(1, ctx.strands.len() as i64) as usize - 1;
                ctx.strands[i].borrow().pts.clone()
            };
            let t = r.range(0.25, 0.75);
            let (p, a) = g::at(&st_pts, t);
            let sd = if r.chance(0.5) { 1.0 } else { -1.0 };
            let out = a + sd * r.range(0.7, 1.1);
            let len = r.range(22.0, 34.0);
            let tip = g::polar(p, out, len);
            let l = r.range(40.0, 50.0);
            let body = [tip[0], tip[1] + l * 0.35];
            if s.room(body[0], body[1]) >= l * 0.38 && s.open_at(tip[0], tip[1]) {
                let twig = g::link(p, a, tip, out + sd * 0.4, 0.5, 0);
                let clear = twig[1..].iter().all(|q| s.open_at(q[0], q[1]));
                if clear {
                    ctx.stem_batch.stem(&twig, 0.9, 0.5);
                    s.reserve_corridor(&twig, 2.0);
                    s.disc(body[0], body[1], l * 0.5, 1.0);
                    let dir = if out.cos() >= 0.0 { 1.0 } else { -1.0 };
                    let mut fr = r.fork("paint");
                    ctx.visitors.push(Box::new(move |pt: &mut Paint| bird(pt, &mut fr, tip[0], tip[1], dir, l)));
                    break;
                }
            }
        }
    }

    for (kind, wanted, need) in [("butterfly", want_fly, 20.0), ("bee", want_bee, 14.0)] {
        let mut r = rng.fork(&format!("{} place", kind));
        for n in 1..=wanted {
            let spots = open_spots(s, need);
            if spots.is_empty() {
                break;
            }
            let sp = spots[(r.int(1, spots.len() as i64) - 1) as usize];
            let (x, y) = (sp[0], sp[1]);
            let mut fr = r.fork(&format!("{}{}", kind, n));
            if kind == "butterfly" {
                let ss = (sp[2] * 1.7).min(r.range(32.0, 42.0));
                let ang = PI / 2.0 + r.range(-0.7, 0.7);
                let open = r.range(0.7, 1.0);
                s.disc(x, y, ss * 0.55, 1.0);
                ctx.visitors.push(Box::new(move |pt: &mut Paint| butterfly(pt, &mut fr, x, y, ang, ss, open)));
            } else {
                let l = r.range(16.0, 20.0);
                let ang = r.range(0.0, 2.0 * PI);
                s.disc(x, y, l * 0.7, 1.0);
                let tr = trail(&mut fr.fork("trail"), x, y, ang, l, s);
                if tr.len() > 4 {
                    s.reserve_corridor(&tr, 1.2);
                }
                ctx.visitors.push(Box::new(move |pt: &mut Paint| {
                    for q in tr.iter().step_by(2) {
                        pt.fill(&[Cmd::O(q[0], q[1], 0.6)], paint::INK, 0.6, MULT);
                    }
                    bee(pt, &mut fr, x, y, ang, l);
                }));
            }
        }
    }
}
