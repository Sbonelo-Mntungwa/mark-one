//! Comparing one settled step of a circuit with what was expected.

use mark_one_engine::signal::Sig;
use mark_one_engine::sim::Sim;

use super::{Failure, TraceRow};
use crate::level::Level;

pub(crate) fn named(names: &[String], v: &[Sig]) -> Vec<(String, Sig)> {
    names.iter().cloned().zip(v.iter().copied()).collect()
}

pub(crate) fn note_for(sim: &Sim, check: bool, got: &[Sig], exp: &[Sig]) -> Option<&'static str> {
    if sim.short {
        Some("Short circuit: something is pulling a line to 1 and to 0 at the same time.")
    } else if check && got != exp {
        Some(if sim.unstable {
            "The circuit never settled. A feedback loop is oscillating."
        } else if sim.degraded_outputs().iter().any(|&d| d) {
            "An output is degraded: it is driven only through a transistor that passes that value weakly (a 1 through an NMOS, or a 0 through a PMOS), so it sits about one threshold voltage short of the rail. Drive outputs from a complementary pull-up and pull-down."
        } else if got.contains(&Sig::Z) {
            "An output is floating (Z): for these inputs nothing connects it to 1 or 0."
        } else if got.contains(&Sig::X) {
            "An output is unknown (X): conflicting drivers, an unconnected input upstream, or stored charge that has leaked away."
        } else {
            "Wrong value."
        })
    } else {
        None
    }
}

pub(super) struct Bench<'a> {
    pub l: &'a Level,
    pub trace: Vec<TraceRow>,
    pub keep_trace: bool,
    pub step: usize,
    pub total: usize,
}

impl Bench<'_> {
    /// Compare one settled step. Returns a failure if it does not match.
    pub fn check(
        &mut self,
        sim: &Sim,
        ins: &[Sig],
        exp: Vec<Sig>,
        check: bool,
        prefix: &str,
    ) -> Option<Failure> {
        self.step += 1;
        let got = sim.outputs();
        if self.keep_trace {
            self.trace.push(TraceRow {
                inputs: ins.to_vec(),
                expected: exp.clone(),
            });
            if self.trace.len() > 6 {
                self.trace.remove(0);
            }
        }
        let note = note_for(sim, check, &got, &exp)?;
        Some(Failure {
            step: self.step,
            total: self.total,
            inputs: named(&self.l.inputs, ins),
            expected: named(&self.l.outputs, &exp),
            got: named(&self.l.outputs, &got),
            note: format!("{prefix}{note}"),
            trace: std::mem::take(&mut self.trace),
        })
    }
}
