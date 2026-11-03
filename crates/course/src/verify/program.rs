//! Software Marks: run the ROM program on the board's own computer, cycle by
//! cycle beside the reference CPU.

use mark_one_engine::circuit::{Circuit, PartKind};
use mark_one_engine::components::{from_num, op, run_program, sigs, Cpu};
use mark_one_engine::signal::Sig;

use super::bench::Bench;
use super::{Outcome, ProgramReport, VerifyResult};
use crate::catalog::levels;
use crate::progress::Ctx;

pub(super) fn verify_program(
    idx: usize,
    c: &Circuit,
    ctx: &Ctx,
    want: &[u8],
    mut res: VerifyResult,
) -> VerifyResult {
    res.unit = "cycles";
    if c.parts.iter().filter(|p| p.kind == PartKind::Rom).count() > 1 {
        res.error = Some(
            "Use one ROM on a software Mark, so it is clear which program is being scored.".into(),
        );
        return res;
    }
    let Some(rom) = c.parts.iter().find(|p| p.kind == PartKind::Rom) else {
        res.error =
            Some("No ROM on the board. Your program lives in a ROM wired to your CPU.".into());
        return res;
    };
    let mut prog = [0u8; 16];
    for (d, s) in prog.iter_mut().zip(rom.data.as_deref().unwrap_or(&[])) {
        *d = *s;
    }
    let run = run_program(&prog, 2000);
    res.cost = run.cycles as u32;
    res.checked = run.outs.len();
    res.program = Some(ProgramReport {
        outs: run.outs.clone(),
        want: want.to_vec(),
        halted: run.halted,
        cycles: run.cycles,
    });
    // The program must run on the board itself, not just on the reference CPU.
    match super::in_both_orders(|rev| run_on_board(idx, c, ctx, &prog, run.cycles + 1, rev)) {
        Ok(None) => res.pass = run.halted && run.outs == want,
        Ok(Some(f)) => res.failure = Some(f),
        Err(e) => res.error = Some(e),
    }
    res
}

/// Clock the board for `cycles` cycles beside the reference CPU running the
/// same ROM, comparing OUT and HALT every half cycle.
fn run_on_board(
    idx: usize,
    c: &Circuit,
    ctx: &Ctx,
    prog: &[u8; 16],
    cycles: usize,
    reverse: bool,
) -> Outcome {
    let l = &levels()[idx];
    let mut sim = ctx.build_sim(idx, c)?;
    sim.set_reverse_order(reverse);
    let mut emu = Cpu::default();
    let mut b = Bench {
        l,
        trace: vec![],
        keep_trace: false,
        step: 0,
        total: cycles * 2,
    };
    for cyc in 0..cycles {
        for clk in [false, true] {
            sim.set_inputs(&[clk]);
            sim.settle();
            if clk {
                emu.step(prog[emu.pc as usize]);
            }
            let mut exp = sigs(&from_num(emu.out as u32, 8));
            exp.push(Sig::from_bool(prog[emu.pc as usize] >> 4 == op::HLT));
            let prefix = format!(
                "Your computer did not run this program correctly. Cycle {}: ",
                cyc + 1
            );
            if let Some(f) = b.check(&sim, &[Sig::from_bool(clk)], exp, true, &prefix) {
                return Ok(Some(f));
            }
        }
    }
    Ok(None)
}
