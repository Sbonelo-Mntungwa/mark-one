//! Switch-level resolution, in the style of IRSIM/COSMOS.
//!
//! - A transistor whose gate is a known value is on or off. One whose gate is
//!   X or Z might be either, so the network is solved both ways and any net
//!   whose value depends on it becomes X.
//! - A conducting group takes the value of its drivers. Drivers of 1 and 0 in
//!   one group are a short (supply to ground), even through ratioed devices;
//!   the nets between them read X. Driver nets themselves are ideal sources.
//! - Strength: a 1 that reaches a net only through an NMOS, or a 0 only through
//!   a PMOS, arrives about one threshold voltage short of the rail. Such a net
//!   is marked degraded.
//! - An undriven group keeps its stored charge but counts as floating. Charge
//!   that differs between merged nets becomes X (charge sharing).

use super::union_find::Uf;
use super::{Fet, Sim, Src};
use crate::signal::Sig;

/// Combine two drivers on the same net. Z yields; equal values agree;
/// anything else is a conflict.
pub(super) fn merge(a: Sig, b: Sig, short: &mut bool) -> Sig {
    match (a, b) {
        (Sig::Z, x) | (x, Sig::Z) => x,
        (Sig::X, _) | (_, Sig::X) => Sig::X,
        (x, y) if x == y => x,
        _ => {
            *short = true;
            Sig::X
        }
    }
}

/// Whether a transistor conducts for a gate value: Some(true) on, Some(false)
/// off, None unknown.
fn conducts(f: &Fet, g: Sig) -> Option<bool> {
    g.as_bool().map(|b| if f.p { !b } else { b })
}

/// Net values for one choice of which transistors conduct.
struct Solved {
    val: Vec<Sig>,
    floating: Vec<bool>,
    short: bool,
}

fn solve(
    n: usize,
    base: &[(Sig, bool)],
    prev: &[Sig],
    fets: &[Fet],
    gates: &[Sig],
    maybe_on: bool,
) -> Solved {
    // Driven nets (rails, input pins, chip outputs) are sources. Channels join
    // undriven nets into groups; a source touching a group drives it but never
    // joins two groups together, just as a rail does not short two gates.
    let source = |i: usize| base[i].0 != Sig::Z;
    let on: Vec<&Fet> = fets
        .iter()
        .zip(gates)
        .filter(|(f, &g)| conducts(f, g).unwrap_or(maybe_on))
        .map(|(f, _)| f)
        .collect();
    let mut uf = Uf::new(n);
    for f in &on {
        if !source(f.a) && !source(f.b) {
            uf.union(f.a, f.b);
        }
    }
    let mut gd = vec![Sig::Z; n];
    let mut gc = vec![Sig::Z; n];
    let mut gshort = vec![false; n];
    let mut short = base.iter().any(|&(_, s)| s);
    for f in &on {
        match (source(f.a), source(f.b)) {
            (true, true) => {
                let mut sh = false;
                merge(base[f.a].0, base[f.b].0, &mut sh);
                short |= sh;
            }
            (true, false) | (false, true) => {
                let (s, g) = if source(f.a) { (f.a, f.b) } else { (f.b, f.a) };
                let r = uf.find(g);
                let mut sh = false;
                gd[r] = merge(gd[r], base[s].0, &mut sh);
                gshort[r] |= sh;
            }
            (false, false) => {}
        }
    }
    for i in (0..n).filter(|&i| !source(i)) {
        let r = uf.find(i);
        let mut ignore = false;
        gc[r] = merge(gc[r], prev[i], &mut ignore);
    }
    let mut out = Solved {
        val: vec![Sig::Z; n],
        floating: vec![false; n],
        short,
    };
    #[allow(clippy::needless_range_loop)] // i indexes several parallel arrays
    for i in 0..n {
        if source(i) {
            out.val[i] = base[i].0;
            continue;
        }
        let r = uf.find(i);
        out.short |= gshort[r];
        out.floating[i] = gd[r] == Sig::Z;
        out.val[i] = if gd[r] == Sig::Z { gc[r] } else { gd[r] };
    }
    out
}

impl Sim {
    pub(super) fn drive(&self, n: usize) -> (Sig, bool) {
        let mut v = Sig::Z;
        let mut short = false;
        for s in &self.drivers[n] {
            let x = match *s {
                Src::Const(c) => c,
                Src::Input(k) => self.inputs.get(k).copied().unwrap_or(Sig::Z),
                Src::ChipOut(c, k) => self.chips[c].out[k],
            };
            v = merge(v, x, &mut short);
        }
        (v, short)
    }

    /// Recompute every net value from the current drivers and transistor states.
    pub(super) fn resolve(&mut self) {
        self.resolve_gates(None);
    }

    /// As `resolve`. With `fixed`, each transistor sees that gate value (one
    /// per transistor) instead of its gate net's value: timing mode, where a
    /// gate change reaches the channel one delay later.
    pub(super) fn resolve_gates(&mut self, fixed: Option<&[Sig]>) {
        let n = self.val.len();
        let prev = self.val.clone();
        let base: Vec<(Sig, bool)> = (0..n).map(|i| self.drive(i)).collect();

        if self.fets.is_empty() {
            let mut short = false;
            for (i, &(d, s)) in base.iter().enumerate() {
                short |= s;
                self.floating[i] = d == Sig::Z;
                self.val[i] = if d == Sig::Z { prev[i] } else { d };
            }
            self.short = short;
            self.degraded = vec![false; n];
            return;
        }

        let mut cur = prev.clone();
        let mut floating = vec![false; n];
        let mut short = false;
        let mut stable = false;
        for _ in 0..(self.fets.len() * 2 + 8) {
            let gates: Vec<Sig> = match fixed {
                Some(g) => g.to_vec(),
                None => self.fets.iter().map(|f| cur[f.g]).collect(),
            };
            let on = solve(n, &base, &prev, &self.fets, &gates, false);
            let maybe = solve(n, &base, &prev, &self.fets, &gates, true);
            let mut next = vec![Sig::Z; n];
            for i in 0..n {
                let same = on.val[i] == maybe.val[i] && on.floating[i] == maybe.floating[i];
                next[i] = if same { on.val[i] } else { Sig::X };
                floating[i] = same && on.floating[i];
            }
            short = on.short;
            if fixed.is_some() {
                cur = next;
                stable = true;
                break;
            }
            if next == cur {
                stable = true;
                break;
            }
            cur = next;
        }
        if !stable {
            self.unstable = true;
        }
        let gates: Vec<Sig> = match fixed {
            Some(g) => g.to_vec(),
            None => self.fets.iter().map(|f| cur[f.g]).collect(),
        };
        self.degraded = self.degraded_nets(&base, &gates, &floating, &cur);
        self.val = cur;
        self.floating = floating;
        self.short = short;
    }

    /// Driven nets that no full-strength path reaches: a 1 is strong only if
    /// it arrives from a 1 driver through PMOS channels, a 0 only through NMOS.
    fn degraded_nets(
        &self,
        base: &[(Sig, bool)],
        gates: &[Sig],
        floating: &[bool],
        val: &[Sig],
    ) -> Vec<bool> {
        let n = val.len();
        let strong = |want: Sig, via_p: bool| {
            let mut reach: Vec<bool> = base.iter().map(|&(d, _)| d == want).collect();
            let mut todo: Vec<usize> = (0..n).filter(|&i| reach[i]).collect();
            while let Some(x) = todo.pop() {
                if base[x].0 != Sig::Z && base[x].0 != want {
                    continue;
                }
                for f in self
                    .fets
                    .iter()
                    .zip(gates)
                    .filter(|(f, &g)| f.p == via_p && conducts(f, g) == Some(true))
                    .map(|(f, _)| f)
                {
                    for (a, b) in [(f.a, f.b), (f.b, f.a)] {
                        if a == x && !reach[b] {
                            reach[b] = true;
                            todo.push(b);
                        }
                    }
                }
            }
            reach
        };
        let (s1, s0) = (strong(Sig::H, true), strong(Sig::L, false));
        (0..n)
            .map(|i| !floating[i] && ((val[i] == Sig::H && !s1[i]) || (val[i] == Sig::L && !s0[i])))
            .collect()
    }
}
