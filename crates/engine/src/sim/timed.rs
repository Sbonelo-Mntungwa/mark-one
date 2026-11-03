//! Timing mode: event-driven simulation with gate delays.
//!
//! The zero-delay simulator settles everything at once. Here time advances in
//! gate delays (see [`crate::timing`]): a chip's outputs change its delay
//! after its inputs do, and a transistor's channel responds one unit after
//! its gate. Nets joined by conducting channels still resolve instantly.
//!
//! Delays are inertial, as in Verilog: a pulse shorter than a chip's delay
//! never reaches its output. A pulse at least that long does, so glitches
//! from unequal path delays (static hazards) appear on the outputs.
//!
//! Edge-triggered chips are checked for setup violations: a data input that
//! changes within [`SETUP`] units before CLK reaches the chip, or at the same
//! moment. The engine's flip-flops then still take the old value, but real
//! hardware might take either or go metastable.
//!
//! Each timed settle is one step of the stimulus. The next step's inputs
//! arrive `SETUP + 1` units after the circuit last moved, as from a test
//! bench that waits for the outputs to settle.

use serde::Serialize;

use super::Sim;
use crate::signal::Sig;

/// Units before a clock edge in which a data input must not change.
pub const SETUP: u64 = 1;

/// Timed runs stop after this many units with events still pending: the
/// circuit is oscillating (or very slow).
pub const HORIZON: u64 = 5000;

#[derive(Default)]
pub(super) struct State {
    now: u64,
    /// Gate value each transistor currently responds to.
    gate_seen: Vec<Sig>,
    /// Pending change per transistor gate: (time, value).
    gate_pending: Vec<Option<(u64, Sig)>>,
    /// Pending change per chip output: (time, value).
    out_pending: Vec<Vec<Option<(u64, Sig)>>>,
    /// Inputs each chip last saw.
    last_ins: Vec<Option<Vec<Sig>>>,
    /// When each net last changed.
    last_change: Vec<Option<u64>>,
}

/// One change of one net.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Change {
    pub t: u64,
    pub net: usize,
    pub v: Sig,
}

/// A data input changing too close to its clock edge.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Violation {
    /// Part id of the chip.
    pub part: String,
    /// When CLK rose at the chip, and when the data input changed.
    pub clk_at: u64,
    pub data_at: u64,
    pub input: usize,
}

/// What one timed settle did.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TimedRun {
    /// Time the inputs changed.
    pub start: u64,
    /// Time of the last change; equal to `start` if nothing moved.
    pub end: u64,
    /// Every net change, in time order.
    pub changes: Vec<Change>,
    /// Events were still pending at the horizon: oscillation.
    pub unsettled: bool,
    pub violations: Vec<Violation>,
}

impl TimedRun {
    /// Nets that reached a 0 or 1 more than once: a glitch (a pulse that
    /// came and went) or a hazard on the way to the final value. Passing
    /// through X on the way to a value, as at power-up, is not a glitch.
    /// Returns (net, number of changes to 0 or 1).
    pub fn glitches(&self) -> Vec<(usize, usize)> {
        let mut count = std::collections::BTreeMap::new();
        for c in self.changes.iter().filter(|c| c.v.as_bool().is_some()) {
            *count.entry(c.net).or_insert(0) += 1;
        }
        count.into_iter().filter(|&(_, n)| n > 1).collect()
    }
}

impl Sim {
    /// Settle with gate delays after the inputs change. Time carries on from
    /// the previous timed settle. A zero-delay `settle` in between starts
    /// timing afresh from the settled state.
    pub fn settle_timed(&mut self) -> TimedRun {
        self.age_charge_before();
        let n = self.val.len();
        let mut st = self.timing.take().unwrap_or_else(|| State {
            gate_seen: self.fets.iter().map(|f| self.val[f.g]).collect(),
            gate_pending: vec![None; self.fets.len()],
            out_pending: self.chips.iter().map(|c| vec![None; c.out.len()]).collect(),
            last_ins: vec![None; self.chips.len()],
            last_change: vec![None; n],
            ..State::default()
        });
        self.unstable = false;
        let start = st.now;
        let mut t = start;
        let mut changes = Vec::new();
        let mut violations = Vec::new();
        let mut short = false;
        loop {
            let before = self.val.clone();
            self.resolve_gates(Some(&st.gate_seen));
            short |= self.short;
            for (i, (&a, &b)) in before.iter().zip(&self.val).enumerate() {
                if a != b {
                    changes.push(Change { t, net: i, v: b });
                    st.last_change[i] = Some(t);
                }
            }
            // Chips whose inputs changed respond after their delay.
            for (ci, chip) in self.chips.iter_mut().enumerate() {
                let ins: Vec<Sig> = chip.ins.iter().map(|&k| self.val[k]).collect();
                if st.last_ins[ci].as_deref() == Some(&ins[..]) {
                    continue;
                }
                if let (Some(clk), Some(prev)) = (chip.clk, st.last_ins[ci].as_ref()) {
                    if prev[clk] == Sig::L && ins[clk] == Sig::H {
                        for (k, &net) in chip.ins.iter().enumerate() {
                            if k == clk {
                                continue;
                            }
                            if let Some(at) = st.last_change[net] {
                                if at <= t && t - at <= SETUP {
                                    violations.push(Violation {
                                        part: chip.id.clone(),
                                        clk_at: t,
                                        data_at: at,
                                        input: k,
                                    });
                                }
                            }
                        }
                    }
                }
                let out = chip.model.eval(&ins);
                st.last_ins[ci] = Some(ins);
                for (k, &v) in out.iter().enumerate() {
                    let p = &mut st.out_pending[ci][k];
                    if v == chip.out[k] {
                        *p = None; // inertial: the pulse never arrives
                    } else if p.map(|x| x.1) != Some(v) {
                        *p = Some((t + chip.delay as u64, v));
                    }
                }
            }
            // Transistors whose gate changed respond one unit later.
            for (fi, f) in self.fets.iter().enumerate() {
                let g = self.val[f.g];
                let p = &mut st.gate_pending[fi];
                if g == st.gate_seen[fi] {
                    *p = None;
                } else if p.map(|x| x.1) != Some(g) {
                    *p = Some((t + 1, g));
                }
            }
            let next = st
                .out_pending
                .iter()
                .flatten()
                .chain(&st.gate_pending)
                .filter_map(|p| p.map(|x| x.0))
                .min();
            let Some(next) = next else { break };
            if next > start + HORIZON {
                self.unstable = true;
                break;
            }
            t = next;
            for (ci, chip) in self.chips.iter_mut().enumerate() {
                for (k, p) in st.out_pending[ci].iter_mut().enumerate() {
                    if let Some((at, v)) = *p {
                        if at == t {
                            chip.out[k] = v;
                            *p = None;
                        }
                    }
                }
            }
            for (fi, p) in st.gate_pending.iter_mut().enumerate() {
                if let Some((at, v)) = *p {
                    if at == t {
                        st.gate_seen[fi] = v;
                        *p = None;
                    }
                }
            }
        }
        self.short = short;
        let end = changes.last().map_or(start, |c| c.t);
        // The next inputs arrive once this step has been quiet for longer
        // than the setup window, so a stimulus that waits for the circuit
        // to settle never violates setup by itself.
        st.now = t.max(end) + SETUP + 1;
        self.timing = Some(st);
        self.age_charge_after();
        TimedRun {
            start,
            end,
            changes,
            unsettled: self.unstable,
            violations,
        }
    }

    /// Current time in timing mode (0 before the first timed settle).
    pub fn now(&self) -> u64 {
        self.timing.as_ref().map_or(0, |s| s.now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Circuit, Part, PartKind};
    use crate::workbench::build;

    fn pins(c: &mut Circuit, ins: &[&str], outs: &[&str]) {
        for (k, n) in ins.iter().enumerate() {
            c.parts
                .push(Part::pin(&n.to_lowercase(), PartKind::In, k, Some(n)));
        }
        for (k, n) in outs.iter().enumerate() {
            c.parts
                .push(Part::pin(&n.to_lowercase(), PartKind::Out, k, Some(n)));
        }
    }

    /// Changes of the net on pin `pin` of `part`, as (time, value).
    fn trace(s: &Sim, r: &TimedRun, part: &str, pin: usize) -> Vec<(u64, Sig)> {
        let n = s.net_of(part, pin).unwrap();
        r.changes
            .iter()
            .filter(|c| c.net == n)
            .map(|c| (c.t - r.start, c.v))
            .collect()
    }

    #[test]
    fn three_inverters_take_three_delays() {
        let mut c = Circuit::default();
        pins(&mut c, &["A"], &["Y"]);
        for g in ["n1", "n2", "n3"] {
            c.parts.push(Part::chip(g, "not"));
        }
        c.connect(("a", 0), ("n1", 0));
        c.connect(("n1", 1), ("n2", 0));
        c.connect(("n2", 1), ("n3", 0));
        c.connect(("n3", 1), ("y", 0));
        let mut s = build(&c).unwrap();
        s.settle();
        s.set_inputs(&[true]);
        let r = s.settle_timed();
        assert_eq!(trace(&s, &r, "y", 0), [(3, Sig::L)]);
        assert_eq!(r.end - r.start, 3);
        assert!(r.glitches().is_empty() && !r.unsettled);
    }

    /// The textbook static-1 hazard: a 2-to-1 selector from a NOT and three
    /// NANDs. With A = B = 1, Y should stay 1 as SEL falls, but the inverted
    /// path is one gate slower, so Y dips to 0 for one delay.
    #[test]
    fn a_gate_level_selector_glitches_when_sel_falls() {
        let mut c = Circuit::default();
        pins(&mut c, &["A", "B", "SEL"], &["Y"]);
        for (g, k) in [
            ("ns", "not"),
            ("na", "nand"),
            ("nb", "nand"),
            ("ny", "nand"),
        ] {
            c.parts.push(Part::chip(g, k));
        }
        c.connect(("sel", 0), ("ns", 0));
        c.connect(("a", 0), ("na", 0));
        c.connect(("ns", 1), ("na", 1));
        c.connect(("b", 0), ("nb", 0));
        c.connect(("sel", 0), ("nb", 1));
        c.connect(("na", 2), ("ny", 0));
        c.connect(("nb", 2), ("ny", 1));
        c.connect(("ny", 2), ("y", 0));
        let mut s = build(&c).unwrap();
        s.set_inputs(&[true, true, true]);
        s.settle();
        assert_eq!(s.outputs(), [Sig::H]);
        s.set_inputs(&[true, true, false]);
        let r = s.settle_timed();
        assert_eq!(
            trace(&s, &r, "y", 0),
            [(2, Sig::L), (3, Sig::H)],
            "a one-delay glitch"
        );
        let y = s.net_of("y", 0).unwrap();
        assert!(r.glitches().iter().any(|&(n, k)| n == y && k == 2));
        // The library MUX chip has no internal hazard: one change, none here.
        let mut m = Circuit::default();
        pins(&mut m, &["A", "B", "SEL"], &["Y"]);
        m.parts.push(Part::chip("m", "mux"));
        for (k, p) in ["a", "b", "sel"].iter().enumerate() {
            m.connect((p, 0), ("m", k));
        }
        m.connect(("m", 3), ("y", 0));
        let mut s = build(&m).unwrap();
        s.set_inputs(&[true, true, true]);
        s.settle();
        s.set_inputs(&[true, true, false]);
        let r = s.settle_timed();
        assert!(trace(&s, &r, "y", 0).is_empty());
    }

    /// NOR(K, Y) and two inverters in a loop: three inversions. With K = 1
    /// the NOR holds the loop; with K = 0 it rings, toggling every three
    /// gate delays.
    #[test]
    fn a_gated_ring_oscillator_runs_with_period_six() {
        let mut c = Circuit::default();
        pins(&mut c, &["K"], &["Y"]);
        c.parts.push(Part::chip("x", "nor"));
        c.parts.push(Part::chip("n1", "not"));
        c.parts.push(Part::chip("n2", "not"));
        c.connect(("k", 0), ("x", 0));
        c.connect(("x", 2), ("n1", 0));
        c.connect(("n1", 1), ("n2", 0));
        c.connect(("n2", 1), ("x", 1));
        c.connect(("n2", 1), ("y", 0));
        let mut s = build(&c).unwrap();
        s.set_inputs(&[true]);
        s.settle();
        assert!(!s.unstable);
        s.set_inputs(&[false]);
        let r = s.settle_timed();
        assert!(r.unsettled, "never settles");
        let ys = trace(&s, &r, "y", 0);
        assert!(ys.len() > 100);
        let gaps: Vec<u64> = ys.windows(2).map(|w| w[1].0 - w[0].0).collect();
        assert!(gaps.iter().all(|&g| g == 3), "{gaps:?}");
        // Stopping it settles again.
        s.set_inputs(&[true]);
        assert!(!s.settle_timed().unsettled);
    }

    #[test]
    fn a_cmos_inverter_takes_one_delay() {
        let mut c = Circuit::default();
        pins(&mut c, &["A"], &["Y"]);
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        c.parts.push(Part::new("p", PartKind::Pmos));
        c.parts.push(Part::new("n", PartKind::Nmos));
        c.connect(("a", 0), ("p", 0));
        c.connect(("a", 0), ("n", 0));
        c.connect(("v", 0), ("p", 1));
        c.connect(("p", 2), ("y", 0));
        c.connect(("y", 0), ("n", 1));
        c.connect(("n", 2), ("g", 0));
        let mut s = build(&c).unwrap();
        s.settle();
        assert_eq!(s.outputs(), [Sig::H]);
        s.set_inputs(&[true]);
        let r = s.settle_timed();
        assert_eq!(trace(&s, &r, "y", 0), [(1, Sig::L)]);
        assert!(!r.changes.is_empty() && !s.short);
    }

    #[test]
    fn data_changing_with_the_clock_is_a_setup_violation() {
        let mut c = Circuit::default();
        pins(&mut c, &["D", "CLK"], &["Q"]);
        c.parts.push(Part::chip("f", "dff"));
        c.connect(("d", 0), ("f", 0));
        c.connect(("clk", 0), ("f", 1));
        c.connect(("f", 2), ("q", 0));
        let mut s = build(&c).unwrap();
        s.settle();
        // Properly: data first, then the edge.
        s.set_inputs(&[true, false]);
        assert!(s.settle_timed().violations.is_empty());
        s.set_inputs(&[true, true]);
        let r = s.settle_timed();
        assert!(r.violations.is_empty());
        assert_eq!(
            trace(&s, &r, "q", 0),
            [(crate::timing::delay("dff").unwrap() as u64, Sig::H)]
        );
        // Both at once.
        s.set_inputs(&[false, false]);
        s.settle_timed();
        s.set_inputs(&[true, true]);
        let r = s.settle_timed();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].part, "f");
        assert_eq!(r.violations[0].clk_at, r.violations[0].data_at);
    }

    #[test]
    fn timed_and_zero_delay_agree_once_settled() {
        let lib = crate::workbench::Library::open();
        let c = crate::analysis::component_circuit("add8", &lib).unwrap();
        let (mut a, mut b) = (lib.build(&c).unwrap(), lib.build(&c).unwrap());
        let mut x = 0x2545_f491u64;
        for _ in 0..200 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let ins = crate::components::from_num(x as u32 & 0x1ffff, 17);
            a.set_inputs(&ins);
            a.settle();
            b.set_inputs(&ins);
            let r = b.settle_timed();
            assert_eq!(a.outputs(), b.outputs());
            assert!(r.end - r.start <= crate::timing::delay("add8").unwrap() as u64);
        }
    }
}
