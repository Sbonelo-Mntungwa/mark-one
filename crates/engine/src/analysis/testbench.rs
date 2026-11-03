//! Test benches: an expected input/output sequence for any circuit, written
//! as text, run step by step on one powered-up copy of the circuit.
//!
//! ```text
//! # Comments start with # or ;
//! A[3:0] B[3:0] CIN | S[3:0] COUT
//! 3      4      0   | 7      0
//! 0xF    1      0   | 0      1
//! ```
//!
//! - The header names columns: input pins, then `|`, then output pins. A bus
//!   column `S[3:0]` stands for pins S3, S2, S1, S0 and takes a number:
//!   decimal, `0x` hex or `0b` binary.
//! - Each row applies its inputs, lets the circuit settle and checks its
//!   outputs. Rows run in order on the same circuit, so state carries over:
//!   a bench can check counters and registers as well as gates.
//! - Inputs: `0`, `1`, a number for a bus, `-` to keep the previous value, or
//!   `P` on a one-bit input to pulse it (0, then 1, then 0) before checking.
//! - Outputs: `0`, `1`, a number for a bus, `X` or `Z` to expect an unknown
//!   or floating line, `-` to skip the check.
//! - A line `reset` powers the circuit up afresh.
//!
//! Every bench runs twice, with chips evaluated first to last and last to
//! first, as diagnostics do. A row that passes only one way is reported as a
//! race.

use serde::Serialize;

use crate::circuit::Circuit;
use crate::components::{from_num, sigs};
use crate::signal::Sig;
use crate::sim::Sim;
use crate::workbench::{self, Library};

/// One column of the header: a pin, or a bus of pins (most significant
/// first, as written).
#[derive(Clone, Debug, PartialEq)]
struct Col {
    text: String,
    /// Pin indices, least significant bit first.
    pins: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
enum In {
    Keep,
    Pulse,
    Value(u64),
}

#[derive(Clone, Debug, PartialEq)]
enum Out {
    Any,
    Value(u64),
    All(Sig),
}

/// One row's result: each output column's bits, and whether the row
/// shorted or failed to settle.
type RowOut = (Vec<Vec<Sig>>, bool, bool);

#[derive(Clone, Debug, PartialEq)]
enum Line {
    Reset,
    Step {
        line: usize,
        ins: Vec<In>,
        outs: Vec<Out>,
    },
}

/// A parsed bench, bound to one circuit's pin names.
#[derive(Debug)]
pub struct TestBench {
    ins: Vec<Col>,
    outs: Vec<Col>,
    lines: Vec<Line>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StepResult {
    /// Line number in the bench text, from 1.
    pub line: usize,
    pub pass: bool,
    /// Passes in one evaluation order and fails in the other.
    pub race: bool,
    /// Expected and actual value of each output column, as written.
    pub expected: Vec<String>,
    pub got: Vec<String>,
    pub short: bool,
    pub unstable: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct BenchReport {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub steps: Vec<StepResult>,
    pub passed: usize,
    pub failed: usize,
}

impl BenchReport {
    pub fn pass(&self) -> bool {
        self.failed == 0 && !self.steps.is_empty()
    }
}

fn parse_num(t: &str) -> Option<u64> {
    let t = t.to_ascii_lowercase();
    if let Some(h) = t.strip_prefix("0x") {
        u64::from_str_radix(h, 16).ok()
    } else if let Some(b) = t.strip_prefix("0b") {
        u64::from_str_radix(b, 2).ok()
    } else {
        t.parse().ok()
    }
}

/// Resolve a header token against pin names. `S[3:0]` is a bus of S3..S0,
/// `S[0:3]` the same bits in the other order.
fn column(tok: &str, names: &[String], what: &str) -> Result<Col, String> {
    let find = |n: &str| {
        names
            .iter()
            .position(|x| x == n)
            .ok_or_else(|| format!("The circuit has no {what} pin called {n}."))
    };
    if let Some((base, rest)) = tok.split_once('[') {
        let range = rest
            .strip_suffix(']')
            .ok_or_else(|| format!("Bus {tok} should look like {base}[7:0]."))?;
        let (a, b) = range
            .split_once(':')
            .ok_or_else(|| format!("Bus {tok} should look like {base}[7:0]."))?;
        let (a, b): (usize, usize) = match (a.trim().parse(), b.trim().parse()) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return Err(format!("Bus {tok} needs numeric bounds.")),
        };
        // Written most significant first; stored least significant first.
        let order: Vec<usize> = if a >= b {
            (b..=a).collect()
        } else {
            (a..=b).rev().collect()
        };
        if order.len() > 64 {
            return Err(format!("Bus {tok} is wider than 64 bits."));
        }
        let pins = order
            .iter()
            .map(|k| find(&format!("{base}{k}")))
            .collect::<Result<_, _>>()?;
        Ok(Col {
            text: tok.into(),
            pins,
        })
    } else {
        Ok(Col {
            text: tok.into(),
            pins: vec![find(tok)?],
        })
    }
}

fn strip_comment(l: &str) -> &str {
    l.split(['#', ';']).next().unwrap_or("").trim()
}

impl TestBench {
    /// Parse bench text for a circuit with these input and output names.
    pub fn parse(src: &str, inputs: &[String], outputs: &[String]) -> Result<TestBench, String> {
        let mut rows = src
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, strip_comment(l)))
            .filter(|(_, l)| !l.is_empty());
        let (hl, header) = rows
            .next()
            .ok_or("The bench is empty. Start with a header such as: A B | Y")?;
        let (hi, ho) = header.split_once('|').ok_or(format!(
            "Line {hl}: the header needs a | between inputs and outputs."
        ))?;
        let ins: Vec<Col> = hi
            .split_whitespace()
            .map(|t| column(t, inputs, "input"))
            .collect::<Result<_, _>>()
            .map_err(|e| format!("Line {hl}: {e}"))?;
        let outs: Vec<Col> = ho
            .split_whitespace()
            .map(|t| column(t, outputs, "output"))
            .collect::<Result<_, _>>()
            .map_err(|e| format!("Line {hl}: {e}"))?;
        if outs.is_empty() {
            return Err(format!("Line {hl}: the header names no outputs to check."));
        }
        let mut lines = Vec::new();
        for (ln, l) in rows {
            if l.eq_ignore_ascii_case("reset") {
                lines.push(Line::Reset);
                continue;
            }
            let (li, lo) = l.split_once('|').ok_or(format!(
                "Line {ln}: a row needs a | between inputs and outputs."
            ))?;
            let (ti, to): (Vec<&str>, Vec<&str>) = (
                li.split_whitespace().collect(),
                lo.split_whitespace().collect(),
            );
            if ti.len() != ins.len() || to.len() != outs.len() {
                return Err(format!(
                    "Line {ln}: expected {} input and {} output values, found {} and {}.",
                    ins.len(),
                    outs.len(),
                    ti.len(),
                    to.len()
                ));
            }
            let fits = |v: u64, c: &Col| c.pins.len() >= 64 || v < 1u64 << c.pins.len();
            let mut iv = Vec::new();
            for (t, c) in ti.iter().zip(&ins) {
                iv.push(match *t {
                    "-" => In::Keep,
                    "P" | "p" if c.pins.len() == 1 => In::Pulse,
                    "P" | "p" => {
                        return Err(format!("Line {ln}: only a one-bit input can be pulsed."))
                    }
                    t => match parse_num(t) {
                        Some(v) if fits(v, c) => In::Value(v),
                        Some(v) => {
                            return Err(format!("Line {ln}: {v} does not fit in {}.", c.text))
                        }
                        None => {
                            return Err(format!(
                                "Line {ln}: \"{t}\" is not a value for {}.",
                                c.text
                            ))
                        }
                    },
                });
            }
            let mut ov = Vec::new();
            for (t, c) in to.iter().zip(&outs) {
                ov.push(match *t {
                    "-" => Out::Any,
                    "X" | "x" => Out::All(Sig::X),
                    "Z" | "z" => Out::All(Sig::Z),
                    t => match parse_num(t) {
                        Some(v) if fits(v, c) => Out::Value(v),
                        Some(v) => {
                            return Err(format!("Line {ln}: {v} does not fit in {}.", c.text))
                        }
                        None => {
                            return Err(format!(
                                "Line {ln}: \"{t}\" is not a value for {}.",
                                c.text
                            ))
                        }
                    },
                });
            }
            lines.push(Line::Step {
                line: ln,
                ins: iv,
                outs: ov,
            });
        }
        if !lines.iter().any(|l| matches!(l, Line::Step { .. })) {
            return Err("The bench has a header but no rows.".into());
        }
        Ok(TestBench { ins, outs, lines })
    }

    /// Run every row; `build` makes a freshly powered copy of the circuit.
    fn run_once(
        &self,
        build: &dyn Fn() -> Result<Sim, String>,
        n_in: usize,
        reverse: bool,
    ) -> Result<Vec<RowOut>, String> {
        let fresh = || -> Result<Sim, String> {
            let mut s = build()?;
            s.set_reverse_order(reverse);
            Ok(s)
        };
        let mut sim = fresh()?;
        let mut cur = vec![false; n_in];
        let mut res = Vec::new();
        for l in &self.lines {
            let Line::Step { ins, .. } = l else {
                sim = fresh()?;
                cur = vec![false; n_in];
                continue;
            };
            let mut pulses = Vec::new();
            for (v, c) in ins.iter().zip(&self.ins) {
                match v {
                    In::Keep => {}
                    In::Pulse => {
                        cur[c.pins[0]] = false;
                        pulses.push(c.pins[0]);
                    }
                    In::Value(v) => {
                        for (k, &p) in c.pins.iter().enumerate() {
                            cur[p] = (v >> k) & 1 == 1;
                        }
                    }
                }
            }
            let (mut short, mut unstable) = (false, false);
            let mut apply = |sim: &mut Sim, cur: &[bool]| {
                sim.set_inputs(cur);
                sim.settle();
                short |= sim.short;
                unstable |= sim.unstable;
            };
            apply(&mut sim, &cur);
            if !pulses.is_empty() {
                for level in [true, false] {
                    for &p in &pulses {
                        cur[p] = level;
                    }
                    apply(&mut sim, &cur);
                }
            }
            let outs = sim.outputs();
            let cols = self
                .outs
                .iter()
                .map(|c| c.pins.iter().map(|&p| outs[p]).collect())
                .collect();
            res.push((cols, short, unstable));
        }
        Ok(res)
    }

    /// Run the bench in both evaluation orders and compare every row.
    pub fn run_with(
        &self,
        build: &dyn Fn() -> Result<Sim, String>,
        inputs: &[String],
        outputs: &[String],
    ) -> Result<BenchReport, String> {
        let fwd = self.run_once(build, inputs.len(), false)?;
        let back = self.run_once(build, inputs.len(), true)?;
        let steps_in = self.lines.iter().filter_map(|l| match l {
            Line::Step { line, outs, .. } => Some((*line, outs)),
            Line::Reset => None,
        });
        let mut steps = Vec::new();
        for (((line, exp), (got, short, unstable)), (got_b, _, _)) in steps_in.zip(&fwd).zip(&back)
        {
            let ok = |got: &[Vec<Sig>]| {
                exp.iter().zip(got).all(|(e, g)| match e {
                    Out::Any => true,
                    Out::All(s) => g.iter().all(|x| x == s),
                    Out::Value(v) => *g == sigs(&from_num_u64(*v, g.len())),
                })
            };
            let (a, b) = (ok(got), ok(got_b));
            steps.push(StepResult {
                line,
                pass: a && b && !short,
                race: a != b,
                expected: exp
                    .iter()
                    .map(|e| match e {
                        Out::Any => "-".into(),
                        Out::All(s) => sig_text(*s).into(),
                        Out::Value(v) => v.to_string(),
                    })
                    .collect(),
                got: got.iter().map(|g| show(g)).collect(),
                short: *short,
                unstable: *unstable,
            });
        }
        let passed = steps.iter().filter(|s| s.pass).count();
        Ok(BenchReport {
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
            failed: steps.len() - passed,
            passed,
            steps,
        })
    }
}

fn from_num_u64(v: u64, n: usize) -> Vec<bool> {
    (0..n).map(|i| i < 64 && (v >> i) & 1 == 1).collect()
}

fn sig_text(s: Sig) -> &'static str {
    match s {
        Sig::L => "0",
        Sig::H => "1",
        Sig::Z => "Z",
        Sig::X => "X",
    }
}

/// A column's value as a bench would write it: a number when every bit is
/// known, X or Z when every bit is, otherwise the bits, most significant first.
fn show(bits: &[Sig]) -> String {
    if let Some(b) = crate::components::bools(bits) {
        return b
            .iter()
            .enumerate()
            .fold(0u64, |a, (i, &x)| a | ((x as u64) << i))
            .to_string();
    }
    if let Some(&f) = bits.first() {
        if bits.iter().all(|&x| x == f) {
            return sig_text(f).into();
        }
    }
    bits.iter().rev().map(|&s| sig_text(s)).collect()
}

/// Run bench text against an open circuit built with `lib`.
pub fn run_bench(c: &Circuit, lib: &Library, src: &str) -> Result<BenchReport, String> {
    let (ins, outs) = (workbench::inputs(c), workbench::outputs(c));
    let tb = TestBench::parse(src, &ins, &outs)?;
    lib.build(c)?;
    tb.run_with(&|| lib.build(c), &ins, &outs)
}

/// Rewrite bench text with every checked output replaced by what the circuit
/// does now: a quick way to record a known-good design as the reference.
/// Comments and spacing are not kept.
pub fn record_bench(c: &Circuit, lib: &Library, src: &str) -> Result<String, String> {
    let (ins, outs) = (workbench::inputs(c), workbench::outputs(c));
    let tb = TestBench::parse(src, &ins, &outs)?;
    let rep = tb.run_with(&|| lib.build(c), &ins, &outs)?;
    let header = |cols: &[Col]| {
        cols.iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut s = format!("{} | {}\n", header(&tb.ins), header(&tb.outs));
    let mut steps = rep.steps.iter();
    for l in &tb.lines {
        match l {
            Line::Reset => s.push_str("reset\n"),
            Line::Step { ins, .. } => {
                let st = steps.next().expect("one result per step");
                let iv: Vec<String> = ins
                    .iter()
                    .map(|v| match v {
                        In::Keep => "-".into(),
                        In::Pulse => "P".into(),
                        In::Value(v) => v.to_string(),
                    })
                    .collect();
                s.push_str(&format!("{} | {}\n", iv.join(" "), st.got.join(" ")));
            }
        }
    }
    Ok(s)
}

/// A bench covering every input combination, each from power-up, with the
/// outputs the circuit gives now. Up to `MAX_TABLE_INPUTS` inputs.
pub fn bench_from_table(c: &Circuit, lib: &Library) -> Result<String, String> {
    let (ins, outs) = (workbench::inputs(c), workbench::outputs(c));
    if outs.is_empty() {
        return Err("Add an OUTPUT pin first: a bench checks outputs.".into());
    }
    if ins.len() > super::MAX_TABLE_INPUTS {
        return Err(format!(
            "A full table covers up to {} inputs; write the rows you care about instead.",
            super::MAX_TABLE_INPUTS
        ));
    }
    let mut s = format!("{} | {}\n", ins.join(" "), outs.join(" "));
    let stateful = lib.has_memory(c);
    for k in 0..1u32 << ins.len() {
        if stateful && k > 0 {
            s.push_str("reset\n");
        }
        let row: Vec<String> = from_num(k, ins.len())
            .iter()
            .map(|&b| (b as u8).to_string())
            .collect();
        s.push_str(&format!(
            "{} | {}\n",
            row.join(" "),
            vec!["-"; outs.len()].join(" ")
        ));
    }
    record_bench(c, lib, &s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Part, PartKind};

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// Pins A0..A3, B0..B3 -> S0..S3, C through one ADD4 chip.
    fn adder() -> Circuit {
        let mut c = Circuit::default();
        for k in 0..4 {
            c.parts.push(Part::pin(
                &format!("a{k}"),
                PartKind::In,
                k,
                Some(&format!("A{k}")),
            ));
            c.parts.push(Part::pin(
                &format!("b{k}"),
                PartKind::In,
                4 + k,
                Some(&format!("B{k}")),
            ));
            c.parts.push(Part::pin(
                &format!("s{k}"),
                PartKind::Out,
                k,
                Some(&format!("S{k}")),
            ));
        }
        c.parts.push(Part::pin("cin", PartKind::In, 8, Some("CIN")));
        c.parts.push(Part::pin("c", PartKind::Out, 4, Some("C")));
        c.parts.push(Part::chip("x", "add4"));
        // ADD4 pins: A0..A3, B0..B3, CIN -> S0..S3, COUT.
        for k in 0..4 {
            c.connect((&format!("a{k}"), 0), ("x", k));
            c.connect((&format!("b{k}"), 0), ("x", 4 + k));
            c.connect(("x", 9 + k), (&format!("s{k}"), 0));
        }
        c.connect(("cin", 0), ("x", 8));
        c.connect(("x", 13), ("c", 0));
        c
    }

    #[test]
    fn buses_take_numbers_and_every_row_is_checked() {
        let src = "# a 4-bit adder\nA[3:0] B[3:0] CIN | S[3:0] C\n3 4 0 | 7 0\n0xF 1 0 | 0 1\n0b1010 0b0101 1 | 0 1\n9 9 0 | 3 1 ; wrong on purpose\n";
        let r = run_bench(&adder(), &Library::open(), src).unwrap();
        assert_eq!((r.passed, r.failed), (3, 1));
        let bad = &r.steps[3];
        assert_eq!(bad.line, 6);
        assert_eq!(bad.expected, ["3", "1"]);
        assert_eq!(bad.got, ["2", "1"]);
        assert!(!bad.pass && !bad.race);
    }

    #[test]
    fn a_counter_is_checked_through_pulses() {
        // PC4 with LOAD = 0, EN = 1: each pulse counts up.
        let mut c = Circuit::default();
        c.parts.push(Part::pin("clk", PartKind::In, 0, Some("CLK")));
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        c.parts.push(Part::chip("pc", "pc4"));
        for k in 0..5 {
            c.connect(("g", 0), ("pc", k));
        }
        c.connect(("v", 0), ("pc", 5));
        c.connect(("clk", 0), ("pc", 6));
        for k in 0..4 {
            c.parts.push(Part::pin(
                &format!("q{k}"),
                PartKind::Out,
                k,
                Some(&format!("Q{k}")),
            ));
            c.connect(("pc", 7 + k), (&format!("q{k}"), 0));
        }
        let src = "CLK | Q[3:0]\n0 | 0\nP | 1\nP | 2\n1 | 3\n- | 3\n0 | 3\nreset\n0 | 0\n";
        let r = run_bench(&c, &Library::open(), src).unwrap();
        assert!(r.pass(), "{r:?}");
        let r = run_bench(&c, &Library::open(), "CLK | Q[3:0]\nP | 2\n").unwrap();
        assert_eq!(r.failed, 1);
        assert_eq!(r.steps[0].got, ["1"]);
    }

    #[test]
    fn floating_and_unknown_outputs_can_be_expected() {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        let r = run_bench(&c, &Library::open(), "A | Y\n0 | Z\n1 | 0\n").unwrap();
        assert!(r.steps[0].pass);
        assert!(!r.steps[1].pass);
        assert_eq!(r.steps[1].got, ["Z"]);
    }

    #[test]
    fn mistakes_in_the_text_are_explained() {
        let (i, o) = (names(&["A", "B"]), names(&["Y"]));
        let err = |s: &str| TestBench::parse(s, &i, &o).unwrap_err();
        assert!(err("").contains("empty"));
        assert!(err("A B Y\n").contains("needs a |"));
        assert!(err("A C | Y\n0 0 | 0").contains("no input pin called C"));
        assert!(err("A B | Y\n0 | 1").contains("expected 2 input"));
        assert!(err("A B | Y\n0 2 | 1").contains("does not fit"));
        assert!(err("A B | Y\n0 q | 1").contains("not a value"));
        assert!(err("A B | Y\n").contains("no rows"));
        assert!(err("A[1:0] | Y\n0 | 0").contains("no input pin called A"));
    }

    #[test]
    fn recording_fills_in_the_outputs_and_then_passes() {
        let lib = Library::open();
        let full = bench_from_table(&adder(), &lib).unwrap();
        assert_eq!(full.lines().count(), 1 + 512);
        assert!(full.starts_with("A0 A1 A2 A3 B0 B1 B2 B3 CIN | S0 S1 S2 S3 C\n"));
        assert!(run_bench(&adder(), &lib, &full).unwrap().pass());
        let rec = record_bench(
            &adder(),
            &lib,
            "A[3:0] B[3:0] CIN | S[3:0] C\n7 8 1 | - -\n",
        )
        .unwrap();
        assert_eq!(rec, "A[3:0] B[3:0] CIN | S[3:0] C\n7 8 1 | 0 1\n");
    }
}
