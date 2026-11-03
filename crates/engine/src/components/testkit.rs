//! Helpers for component tests.

use super::*;
use crate::circuit::{Circuit, End, Part, PartKind, Wire};
use crate::sim::Sim;

/// Every input combination must give exactly `spec`'s outputs.
pub fn exhaustive(id: &str, spec: impl Fn(&[bool]) -> Vec<bool>) {
    let c = find(id).unwrap_or_else(|| panic!("no component {id}"));
    let n = c.spec.inputs.len();
    assert!(n <= 20, "{id}: too many inputs for an exhaustive check");
    let mut m = (c.make)();
    for k in 0..1u32 << n {
        let ins = from_num(k, n);
        let want = sigs(&spec(&ins));
        assert_eq!(
            m.eval(&sigs(&ins)),
            want,
            "{id} with inputs {}",
            show(c.spec.inputs, &ins)
        );
    }
}

/// Literal truth table: rows of (inputs, outputs) as 0/1 strings, inputs in
/// pin order.
pub fn table(id: &str, rows: &[(&str, &str)]) {
    let c = find(id).unwrap();
    assert_eq!(
        rows.len(),
        1 << c.spec.inputs.len(),
        "{id}: the table must list every row"
    );
    let mut m = (c.make)();
    for (i, o) in rows {
        let got: String = m.eval(&parse(i)).iter().map(|s| sig_char(*s)).collect();
        assert_eq!(&got, o, "{id} with inputs {i}");
    }
}

/// Drive a stateful model through a script of (inputs, expected outputs).
/// Expected uses 0, 1, X and Z; `-` skips a check.
pub fn script(m: &mut dyn Model, steps: &[(&str, &str)]) {
    for (n, (i, o)) in steps.iter().enumerate() {
        let got: String = m.eval(&parse(i)).iter().map(|s| sig_char(*s)).collect();
        if *o != "-" {
            assert_eq!(&got, o, "step {} with inputs {i}", n + 1);
        }
    }
}

/// One rising clock edge: inputs `data` with CLK appended low, then high.
pub fn clock(m: &mut dyn Model, data: &[bool]) -> Vec<Sig> {
    let mut v = sigs(data);
    v.push(Sig::L);
    m.eval(&v);
    *v.last_mut().unwrap() = Sig::H;
    m.eval(&v)
}

pub fn parse(s: &str) -> Vec<Sig> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '0' => Sig::L,
            '1' => Sig::H,
            'Z' => Sig::Z,
            _ => Sig::X,
        })
        .collect()
}

pub fn sig_char(s: Sig) -> char {
    match s {
        Sig::L => '0',
        Sig::H => '1',
        Sig::Z => 'Z',
        Sig::X => 'X',
    }
}

fn show(names: &[&str], b: &[bool]) -> String {
    names
        .iter()
        .zip(b)
        .map(|(n, &v)| format!("{n}={}", v as u8))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A transistor-level circuit for checking a gate in the switch-level
/// simulator. Nets are named; `n` and `p` add transistors.
pub struct Fab {
    c: Circuit,
    nets: Vec<String>,
    n_in: usize,
}

impl Fab {
    pub fn new(inputs: &[&str], outputs: &[&str]) -> Fab {
        let mut f = Fab {
            c: Circuit::default(),
            nets: vec![],
            n_in: inputs.len(),
        };
        f.part("vcc", PartKind::Vcc, None);
        f.part("gnd", PartKind::Gnd, None);
        for (k, n) in inputs.iter().enumerate() {
            f.part(&format!("in{k}"), PartKind::In, Some(k));
            f.join(n, &format!("in{k}"), 0);
        }
        for (k, n) in outputs.iter().enumerate() {
            f.part(&format!("out{k}"), PartKind::Out, Some(k));
            f.join(n, &format!("out{k}"), 0);
        }
        f.join("VCC", "vcc", 0);
        f.join("GND", "gnd", 0);
        f
    }
    fn part(&mut self, id: &str, kind: PartKind, pin: Option<usize>) {
        self.c.parts.push(Part {
            id: id.into(),
            kind,
            chip: None,
            pin,
            data: None,
            src: None,
            rot: 0,
            label: None,
            x: 0.0,
            y: 0.0,
        });
    }
    /// Connect pin `k` of `part` to the named net.
    fn join(&mut self, net: &str, part: &str, k: usize) {
        let to = End {
            part: part.into(),
            pin: k,
        };
        // Each net is anchored on the first pin that joins it.
        match self.nets.iter().position(|n| n == net) {
            Some(i) => {
                let anchor = self
                    .c
                    .parts
                    .iter()
                    .find(|p| p.id == format!("net{i}"))
                    .map(|p| p.id.clone());
                let from = End {
                    part: anchor.unwrap(),
                    pin: 0,
                };
                self.c.wires.push(Wire { from, to });
            }
            None => {
                let i = self.nets.len();
                self.nets.push(net.into());
                // A display reads its pins without driving them: a passive anchor.
                self.part(&format!("net{i}"), PartKind::Display, None);
                self.c.wires.push(Wire {
                    from: End {
                        part: format!("net{i}"),
                        pin: 0,
                    },
                    to,
                });
            }
        }
    }
    fn fet(&mut self, kind: PartKind, g: &str, s: &str, d: &str) {
        let id = format!("t{}", self.c.parts.len());
        self.part(&id, kind, None);
        self.join(g, &id, 0);
        self.join(s, &id, 1);
        self.join(d, &id, 2);
    }
    pub fn n(&mut self, g: &str, s: &str, d: &str) -> &mut Self {
        self.fet(PartKind::Nmos, g, s, d);
        self
    }
    pub fn p(&mut self, g: &str, s: &str, d: &str) -> &mut Self {
        self.fet(PartKind::Pmos, g, s, d);
        self
    }

    /// The transistor circuit must match component `id` on every input:
    /// fully driven outputs, no shorts, no oscillation.
    pub fn proves(&self, id: &str) {
        let c = find(id).unwrap();
        let n_out = c.spec.outputs.len();
        let mut sim = Sim::build(&self.c, self.n_in, n_out, &|_| None, &|_| Ok(())).expect("build");
        let mut m = (c.make)();
        // Every input ascending, then descending: a circuit that only works
        // after one particular history is caught.
        let all = 1u32 << self.n_in;
        for k in (0..all).chain((0..all).rev()) {
            let ins = from_num(k, self.n_in);
            sim.set_inputs(&ins);
            sim.settle();
            let at = show(c.spec.inputs, &ins);
            assert!(!sim.short, "{id}: short circuit with inputs {at}");
            assert!(!sim.unstable, "{id}: oscillates with inputs {at}");
            assert!(
                !sim.degraded_outputs().contains(&true),
                "{id}: degraded output with inputs {at}"
            );
            assert_eq!(
                sim.outputs(),
                m.eval(&sigs(&ins)),
                "{id}: transistor circuit vs model, inputs {at}"
            );
        }
    }

    /// Outputs after each input step, in order, as 0/1/Z/X strings. Fails on a
    /// short or an oscillation.
    pub fn sequence(&self, n_out: usize, steps: &[&str]) -> Vec<String> {
        let mut sim = Sim::build(&self.c, self.n_in, n_out, &|_| None, &|_| Ok(())).expect("build");
        steps
            .iter()
            .map(|s| {
                let ins: Vec<bool> = parse(s).iter().map(|x| *x == Sig::H).collect();
                sim.set_inputs(&ins);
                sim.settle();
                assert!(!sim.short && !sim.unstable, "short or oscillation at {s}");
                sim.outputs().into_iter().map(sig_char).collect()
            })
            .collect()
    }

    /// Outputs on every input row, as 0/1/Z/X strings.
    pub fn outputs(&self, n_out: usize) -> Vec<(String, bool)> {
        let mut sim = Sim::build(&self.c, self.n_in, n_out, &|_| None, &|_| Ok(())).expect("build");
        (0..1u32 << self.n_in)
            .map(|k| {
                sim.set_inputs(&from_num(k, self.n_in));
                sim.settle();
                (sim.outputs().into_iter().map(sig_char).collect(), sim.short)
            })
            .collect()
    }
}
