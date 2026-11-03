//! Diagnostics: drive a circuit through its Mark's test bench and compare
//! every step with the reference model.

mod bench;
mod cpu;
mod machine;
mod program;
mod steps;

use serde::Serialize;

use mark_one_engine::circuit::Circuit;
use mark_one_engine::signal::Sig;

use crate::catalog::levels;
use crate::level::Test;
use crate::progress::Ctx;

/// Cycles per test program for the CPU and the computer.
const CYCLES: usize = 48;

#[derive(Serialize, Debug)]
pub struct TraceRow {
    pub inputs: Vec<Sig>,
    pub expected: Vec<Sig>,
}

#[derive(Serialize, Debug)]
pub struct Failure {
    pub step: usize,
    pub total: usize,
    pub inputs: Vec<(String, Sig)>,
    pub expected: Vec<(String, Sig)>,
    pub got: Vec<(String, Sig)>,
    pub note: String,
    /// Recent history, for sequential levels.
    pub trace: Vec<TraceRow>,
}

#[derive(Serialize, Debug)]
pub struct ProgramReport {
    pub outs: Vec<u8>,
    pub want: Vec<u8>,
    pub halted: bool,
    pub cycles: usize,
}

#[derive(Serialize, Debug)]
pub struct VerifyResult {
    pub pass: bool,
    pub cost: u32,
    pub par: u32,
    /// What cost counts: "transistors" or "cycles".
    pub unit: &'static str,
    pub checked: usize,
    pub error: Option<String>,
    pub failure: Option<Failure>,
    pub program: Option<ProgramReport>,
}

pub fn verify(idx: usize, c: &Circuit, ctx: &Ctx) -> VerifyResult {
    let l = &levels()[idx];
    let mut res = VerifyResult {
        pass: false,
        cost: ctx.circuit_cost(c),
        par: l.par,
        unit: "transistors",
        checked: 0,
        error: None,
        failure: None,
        program: None,
    };
    match &l.test {
        Test::Sandbox => {
            res.error = Some("The workshop has no diagnostics. Build anything.".into());
            return res;
        }
        Test::Program(want) => return program::verify_program(idx, c, ctx, want, res),
        _ => {}
    }
    let outcome = in_both_orders(|rev| {
        res.checked = 0;
        match &l.test {
            Test::Cpu => cpu::verify_cpu(idx, c, ctx, &mut res, rev),
            Test::Machine => machine::verify_machine(idx, c, ctx, &mut res, rev),
            _ => steps::verify_steps(idx, c, ctx, &mut res, rev),
        }
    });
    match outcome {
        Ok(None) => res.pass = true,
        Ok(Some(f)) => res.failure = Some(f),
        Err(e) => res.error = Some(e),
    }
    res
}

/// Run a bench with chips evaluated first to last, then last to first. The
/// simulator has no delays, so a circuit that only works in one order depends
/// on a race or a glitch, and would not work reliably in silicon.
pub(crate) fn in_both_orders(mut run: impl FnMut(bool) -> Outcome) -> Outcome {
    match run(false)? {
        Some(f) => Ok(Some(f)),
        None => Ok(run(true)?.map(|mut f| {
            f.note = format!(
                "Passes only when chips are evaluated in one particular order: the design depends on a race or a glitch, which real gate delays make unreliable. {}",
                f.note
            );
            f
        })),
    }
}

pub(crate) type Outcome = Result<Option<Failure>, String>;
