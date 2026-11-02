//! Lua semantics the port must keep to reproduce the generator exactly.

/// Lua's floored float modulo.
#[inline]
pub fn fmod(a: f64, b: f64) -> f64 {
    let m = a % b;
    if m * b < 0.0 {
        m + b
    } else {
        m
    }
}

/// `table.sort`: Lua 5.3's quicksort, so elements that compare equal land where
/// they do in Lua. Above 100 elements Lua randomises its pivot; the midpoint is
/// used here.
pub fn sort<T, F: FnMut(&T, &T) -> bool>(a: &mut [T], mut lt: F) {
    if a.len() > 1 {
        aux(a, 0, a.len() - 1, &mut lt);
    }
}

fn aux<T, F: FnMut(&T, &T) -> bool>(a: &mut [T], mut lo: usize, mut up: usize, lt: &mut F) {
    while lo < up {
        if lt(&a[up], &a[lo]) {
            a.swap(lo, up);
        }
        if up - lo == 1 {
            break;
        }
        let p = (lo + up) / 2;
        if lt(&a[p], &a[lo]) {
            a.swap(p, lo);
        } else if lt(&a[up], &a[p]) {
            a.swap(p, up);
        }
        if up - lo == 2 {
            break;
        }
        a.swap(p, up - 1);
        // Partition around the pivot now at up - 1.
        let pv = up - 1;
        let mut i = lo;
        let mut j = up - 1;
        loop {
            i += 1;
            while lt(&a[i], &a[pv]) {
                i += 1;
            }
            j -= 1;
            while lt(&a[pv], &a[j]) {
                j -= 1;
            }
            if j < i {
                break;
            }
            a.swap(i, j);
        }
        a.swap(up - 1, i);
        let p = i;
        if p - lo < up - p {
            if p > 0 {
                aux(a, lo, p - 1, lt);
            }
            lo = p + 1;
        } else {
            aux(a, p + 1, up, lt);
            if p == 0 {
                break;
            }
            up = p - 1;
        }
    }
}
