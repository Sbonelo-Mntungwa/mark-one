//! Equivalence checking: do two circuits behave identically?
//!
//! Pins are matched by name, so the two designs may number and place them
//! differently but must name the same inputs and outputs.
//!
//! - If neither circuit has memory and they have at most
//!   [`MAX_PROOF_INPUTS`] inputs, every input combination is applied to a
//!   freshly powered copy of each, in both evaluation orders. Agreement on
//!   all of them is a proof.
//! - Otherwise (memory, or too many inputs) the check is bounded: seeded
//!   random sequences from power-up, compared after every settle. Agreement
//!   is strong evidence, not a proof, and the report says so.
//!
//! An input named CLK is never changed in the same settle as the others, as
//! in diagnostics. Data changes while CLK is low and, at random, while it is
//! high, so a level-sensitive latch is told apart from an edge-triggered
//! flip-flop.

use serde::Serialize;

use crate::circuit::{Circuit, Part, PartKind};
use crate::components::from_num;
use crate::signal::Sig;
use crate::sim::Sim;
use crate::workbench::{self, Library};

/// Exhaustive proofs cover up to this many inputs (65,536 combinations).
pub const MAX_PROOF_INPUTS: usize = 16;
/// Random sequences in a bounded check, and steps in each.
pub const SEQUENCES: usize = 200;
pub const STEPS: usize = 64;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// Equal on every input combination.
    Proved,
    /// No difference found in the random sequences tried.
    Bounded,
    /// A difference was found; see the counterexample.
    Different,
}

/// One step of a counterexample.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CexStep {
    pub ins: Vec<bool>,
    pub a: Vec<Sig>,
    pub b: Vec<Sig>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Equivalence {
    pub verdict: Verdict,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    /// Input combinations (proof) or settled steps (bounded) compared.
    pub checked: u64,
    /// Plain-language summary.
    pub detail: String,
    /// The shortest sequence from power-up that tells them apart.
    pub counterexample: Vec<CexStep>,
}

/// An open circuit containing just component `id`, with pins named as the
/// component names them. Lets a design be compared with a library or custom
/// component directly.
pub fn component_circuit(id: &str, lib: &Library) -> Result<Circuit, String> {
    let (ins, outs) = lib
        .pins(id)
        .ok_or_else(|| format!("Unknown component '{id}'."))?;
    let mut c = Circuit::default();
    c.parts.push(Part::chip("dut", id));
    for (k, n) in ins.iter().enumerate() {
        let pid = format!("i{k}");
        c.parts.push(Part::pin(&pid, PartKind::In, k, Some(n)));
        c.connect((&pid, 0), ("dut", k));
    }
    for (k, n) in outs.iter().enumerate() {
        let pid = format!("o{k}");
        c.parts.push(Part::pin(&pid, PartKind::Out, k, Some(n)));
        c.connect(("dut", ins.len() + k), (&pid, 0));
    }
    Ok(c)
}

/// For each name in `want`, its index in `have`.
fn mapping(want: &[String], have: &[String], what: &str) -> Result<Vec<usize>, String> {
    let mut sorted_w = want.to_vec();
    let mut sorted_h = have.to_vec();
    sorted_w.sort();
    sorted_h.sort();
    if sorted_w != sorted_h {
        return Err(format!(
            "The designs must have the same {what} pins. First: {}. Second: {}.",
            want.join(", "),
            have.join(", ")
        ));
    }
    let mut dup = sorted_w.clone();
    dup.dedup();
    if dup.len() != sorted_w.len() {
        return Err(format!(
            "Two {what} pins share a name; name each one differently."
        ));
    }
    Ok(want
        .iter()
        .map(|n| have.iter().position(|h| h == n).expect("same names"))
        .collect())
}

struct Pair<'a> {
    a: &'a Circuit,
    b: &'a Circuit,
    lib: &'a Library<'a>,
    /// Index in b of each of a's inputs and outputs.
    imap: Vec<usize>,
    omap: Vec<usize>,
}

impl Pair<'_> {
    fn build(&self, reverse: bool) -> Result<(Sim, Sim), String> {
        let (mut x, mut y) = (self.lib.build(self.a)?, self.lib.build(self.b)?);
        x.set_reverse_order(reverse);
        y.set_reverse_order(reverse);
        Ok((x, y))
    }

    /// Apply `ins` (in a's pin order) to both and compare. Returns both
    /// circuits' outputs in a's order, and whether they agree.
    fn step(&self, x: &mut Sim, y: &mut Sim, ins: &[bool]) -> (Vec<Sig>, Vec<Sig>, bool) {
        let mut ib = vec![false; ins.len()];
        for (k, &v) in ins.iter().enumerate() {
            ib[self.imap[k]] = v;
        }
        x.set_inputs(ins);
        x.settle();
        y.set_inputs(&ib);
        y.settle();
        let oa = x.outputs();
        let ob_raw = y.outputs();
        let ob: Vec<Sig> = self.omap.iter().map(|&j| ob_raw[j]).collect();
        let same = oa == ob && x.short == y.short;
        (oa, ob, same)
    }
}

/// Small seeded generator, so a bounded check is reproducible.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Compare circuits `a` and `b`, both built with `lib`.
pub fn check_equivalence(a: &Circuit, b: &Circuit, lib: &Library) -> Result<Equivalence, String> {
    let (ia, oa) = (workbench::inputs(a), workbench::outputs(a));
    let (ib, ob) = (workbench::inputs(b), workbench::outputs(b));
    if oa.is_empty() {
        return Err("The designs have no OUTPUT pins to compare.".into());
    }
    let pair = Pair {
        a,
        b,
        lib,
        imap: mapping(&ia, &ib, "input")?,
        omap: mapping(&oa, &ob, "output")?,
    };
    pair.build(false)?;
    let n = ia.len();
    let memory = lib.has_memory(a) || lib.has_memory(b);
    let report = |verdict, checked, detail: String, cex| Equivalence {
        verdict,
        inputs: ia.clone(),
        outputs: oa.clone(),
        checked,
        detail,
        counterexample: cex,
    };

    if !memory && n <= MAX_PROOF_INPUTS {
        for k in 0..1u64 << n {
            let ins = from_num(k as u32, n);
            for reverse in [false, true] {
                let (mut x, mut y) = pair.build(reverse)?;
                let (va, vb, same) = pair.step(&mut x, &mut y, &ins);
                if !same {
                    return Ok(report(
                        Verdict::Different,
                        k + 1,
                        format!(
                            "They differ on input combination {} of {}{}.",
                            k + 1,
                            1u64 << n,
                            if reverse {
                                " when chips are evaluated last to first (a race in one of them)"
                            } else {
                                ""
                            }
                        ),
                        vec![CexStep { ins, a: va, b: vb }],
                    ));
                }
            }
        }
        // No chip stores state, but transistor feedback could. One long
        // sweep on a single powered-up copy catches outputs that depend on
        // what came before; if it finds none the proof stands.
        if let Some(e) = bounded(&pair, &ia, 4, STEPS)? {
            return Ok(report(Verdict::Different, 1u64 << n, e.0, e.1));
        }
        return Ok(report(
            Verdict::Proved,
            1u64 << n,
            format!(
                "Equivalent: identical outputs on all {} input combinations, in both evaluation orders.",
                1u64 << n
            ),
            vec![],
        ));
    }

    if let Some(e) = bounded(&pair, &ia, SEQUENCES, STEPS)? {
        return Ok(report(Verdict::Different, 0, e.0, e.1));
    }
    let why = if memory {
        "at least one design stores state"
    } else {
        "there are too many inputs to try every combination"
    };
    Ok(report(
        Verdict::Bounded,
        (SEQUENCES * STEPS * 2) as u64,
        format!(
            "No difference in {SEQUENCES} random sequences of {STEPS} steps from power-up, in both evaluation orders. This is not a proof: {why}."
        ),
        vec![],
    ))
}

type Found = Option<(String, Vec<CexStep>)>;

/// Random sequences from power-up. Returns the first difference, cut to the
/// steps that lead to it.
fn bounded(pair: &Pair, ins: &[String], seqs: usize, steps: usize) -> Result<Found, String> {
    let n = ins.len();
    let clk = ins.iter().position(|s| s == "CLK");
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for s in 0..seqs {
        for reverse in [false, true] {
            let (mut x, mut y) = pair.build(reverse)?;
            let mut cur = vec![false; n];
            let mut trail = Vec::new();
            for _ in 0..steps {
                // With a clock, one step is: CLK falls (data held), new data
                // with CLK low, CLK rises (data held), and half the time new
                // data with CLK high. Data and CLK never change together,
                // but data does change while CLK is high, which tells a
                // latch from a flip-flop.
                let mut actions = vec![];
                if clk.is_some() {
                    actions.push((Some(false), false));
                    actions.push((None, true));
                    actions.push((Some(true), false));
                    if rng.next() & 1 == 1 {
                        actions.push((None, true));
                    }
                } else {
                    actions.push((None, true));
                }
                for (level, new_data) in actions {
                    if new_data {
                        let r = rng.next();
                        for (k, v) in cur.iter_mut().enumerate() {
                            if Some(k) != clk {
                                *v = (r >> (k % 64)) & 1 == 1;
                            }
                        }
                    }
                    if let (Some(c), Some(l)) = (clk, level) {
                        cur[c] = l;
                    }
                    let (va, vb, same) = pair.step(&mut x, &mut y, &cur);
                    trail.push(CexStep {
                        ins: cur.clone(),
                        a: va,
                        b: vb,
                    });
                    if !same {
                        return Ok(Some((
                            format!(
                                "They differ after {} settle{} of random sequence {} from power-up.",
                                trail.len(),
                                if trail.len() == 1 { "" } else { "s" },
                                s + 1
                            ),
                            trail,
                        )));
                    }
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// XOR from four NANDs, pins A, B -> Y, with the inputs placed in the
    /// opposite order to show pins are matched by name.
    fn xor_from_nands() -> Circuit {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("b", PartKind::In, 0, Some("B")));
        c.parts.push(Part::pin("a", PartKind::In, 1, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        for g in ["n1", "n2", "n3", "n4"] {
            c.parts.push(Part::chip(g, "nand"));
        }
        c.connect(("a", 0), ("n1", 0));
        c.connect(("b", 0), ("n1", 1));
        c.connect(("a", 0), ("n2", 0));
        c.connect(("n1", 2), ("n2", 1));
        c.connect(("b", 0), ("n3", 0));
        c.connect(("n1", 2), ("n3", 1));
        c.connect(("n2", 2), ("n4", 0));
        c.connect(("n3", 2), ("n4", 1));
        c.connect(("n4", 2), ("y", 0));
        c
    }

    #[test]
    fn four_nands_are_proved_to_be_an_xor() {
        let lib = Library::open();
        let xor = component_circuit("xor", &lib).unwrap();
        let r = check_equivalence(&xor_from_nands(), &xor, &lib).unwrap();
        assert_eq!(r.verdict, Verdict::Proved, "{}", r.detail);
        assert_eq!(r.checked, 4);
    }

    #[test]
    fn a_wrong_gate_gives_a_counterexample() {
        let lib = Library::open();
        let or = component_circuit("or", &lib).unwrap();
        let r = check_equivalence(&xor_from_nands(), &or, &lib).unwrap();
        assert_eq!(r.verdict, Verdict::Different);
        let c = &r.counterexample[0];
        // Inputs in the first design's order: B, A. Only 1,1 differs.
        assert_eq!(c.ins, [true, true]);
        assert_eq!((c.a[0], c.b[0]), (Sig::L, Sig::H));
    }

    #[test]
    fn sequential_designs_get_a_bounded_check() {
        let lib = Library::open();
        let dff = component_circuit("dff", &lib).unwrap();
        let r = check_equivalence(&dff, &dff.clone(), &lib).unwrap();
        assert_eq!(r.verdict, Verdict::Bounded);
        assert!(r.detail.contains("not a proof"));
        // A D latch is not a flip-flop: it is transparent while its enable is
        // high. Rename its pins so they line up with the flip-flop's.
        let mut latch = component_circuit("d_latch", &lib).unwrap();
        for p in &mut latch.parts {
            if p.label.as_deref() == Some("E") {
                p.label = Some("CLK".into());
            }
        }
        let r = check_equivalence(&dff, &latch, &lib).unwrap();
        assert_eq!(r.verdict, Verdict::Different, "{}", r.detail);
        assert!(!r.counterexample.is_empty());
    }

    #[test]
    fn pins_must_match_by_name() {
        let lib = Library::open();
        let and = component_circuit("and", &lib).unwrap();
        let mux = component_circuit("mux", &lib).unwrap();
        let e = check_equivalence(&and, &mux, &lib).unwrap_err();
        assert!(e.contains("same input pins"), "{e}");
    }
}
