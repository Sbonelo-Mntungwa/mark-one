//! Combinational and clocked Marks: a fixed sequence of input steps.

use mark_one_engine::circuit::Circuit;
use mark_one_engine::components::sigs;

use super::bench::Bench;
use super::{Outcome, VerifyResult};
use crate::bench::steps;
use crate::catalog::levels;
use crate::level::Test;
use crate::progress::Ctx;
use crate::reference::model;

pub(super) fn verify_steps(
    idx: usize,
    c: &Circuit,
    ctx: &Ctx,
    res: &mut VerifyResult,
    reverse: bool,
) -> Outcome {
    let l = &levels()[idx];
    let mut sim = ctx.build_sim(idx, c)?;
    sim.set_reverse_order(reverse);
    let mut reference = model(l.id).expect("every circuit level has a model");
    let all = steps(l);
    let mut b = Bench {
        l,
        trace: vec![],
        keep_trace: !matches!(l.test, Test::Comb),
        step: 0,
        total: all.len(),
    };
    for st in &all {
        sim.set_inputs(&st.ins);
        sim.settle();
        let ins = sigs(&st.ins);
        let exp = reference.eval(&ins);
        if let Some(f) = b.check(&sim, &ins, exp, st.check, "") {
            return Ok(Some(f));
        }
        if st.check {
            res.checked += 1;
        }
    }
    Ok(None)
}
