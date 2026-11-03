//! The CPU Mark: diagnostics act as program memory, feeding the instruction
//! at the reference CPU's PC.

use mark_one_engine::circuit::Circuit;
use mark_one_engine::components::{from_num, sigs, Cpu, Model};

use super::bench::Bench;
use super::{Outcome, VerifyResult, CYCLES};
use crate::catalog::levels;
use crate::programs::test_programs;
use crate::progress::Ctx;

pub(super) fn verify_cpu(
    idx: usize,
    c: &Circuit,
    ctx: &Ctx,
    res: &mut VerifyResult,
    reverse: bool,
) -> Outcome {
    let l = &levels()[idx];
    let progs = test_programs(l.id);
    let mut b = Bench {
        l,
        trace: vec![],
        keep_trace: true,
        step: 0,
        total: progs.len() * CYCLES * 2,
    };
    for (pi, prog) in progs.iter().enumerate() {
        let mut sim = ctx.build_sim(idx, c)?;
        sim.set_reverse_order(reverse);
        let mut reference = Cpu::default();
        b.trace.clear();
        for cyc in 0..CYCLES {
            let instr = prog[reference.pc as usize];
            for clk in [false, true] {
                let mut ins = from_num(instr as u32, 8);
                ins.push(clk);
                sim.set_inputs(&ins);
                sim.settle();
                let ins = sigs(&ins);
                let exp = reference.eval(&ins);
                let prefix = format!(
                    "Test program {}, cycle {}, instruction {:02X}: ",
                    pi + 1,
                    cyc + 1,
                    instr
                );
                if let Some(f) = b.check(&sim, &ins, exp, true, &prefix) {
                    return Ok(Some(f));
                }
                res.checked += 1;
            }
        }
    }
    Ok(None)
}
