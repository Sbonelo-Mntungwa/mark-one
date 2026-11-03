//! The computer Mark: diagnostics load test programs into every ROM on the
//! board and compare OUT and HALT each cycle.

use mark_one_engine::circuit::{Circuit, PartKind};
use mark_one_engine::components::{from_num, op, sigs, Cpu};
use mark_one_engine::signal::Sig;

use super::bench::Bench;
use super::{Outcome, VerifyResult, CYCLES};
use crate::catalog::levels;
use crate::programs::test_programs;
use crate::progress::Ctx;

pub(super) fn verify_machine(
    idx: usize,
    c: &Circuit,
    ctx: &Ctx,
    res: &mut VerifyResult,
    reverse: bool,
) -> Outcome {
    let l = &levels()[idx];
    if !c.parts.iter().any(|p| p.kind == PartKind::Rom) {
        return Err("Place a ROM on the board: the computer needs program memory.".into());
    }
    let progs = test_programs(l.id);
    let mut b = Bench {
        l,
        trace: vec![],
        keep_trace: false,
        step: 0,
        total: progs.len() * CYCLES * 2,
    };
    for (pi, prog) in progs.iter().enumerate() {
        let mut cc = c.clone();
        for p in cc.parts.iter_mut().filter(|p| p.kind == PartKind::Rom) {
            p.data = Some(prog.to_vec());
        }
        let mut sim = ctx.build_sim(idx, &cc)?;
        sim.set_reverse_order(reverse);
        let mut emu = Cpu::default();
        for cyc in 0..CYCLES {
            for clk in [false, true] {
                sim.set_inputs(&[clk]);
                sim.settle();
                if clk {
                    emu.step(prog[emu.pc as usize]);
                }
                let mut exp = sigs(&from_num(emu.out as u32, 8));
                exp.push(Sig::from_bool(prog[emu.pc as usize] >> 4 == op::HLT));
                let prefix = format!("Test program {}, cycle {}: ", pi + 1, cyc + 1);
                if let Some(f) = b.check(&sim, &[Sig::from_bool(clk)], exp, true, &prefix) {
                    return Ok(Some(f));
                }
                res.checked += 1;
            }
        }
    }
    Ok(None)
}
