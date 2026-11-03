//! Gated D latch: while E is 1, Q follows D. While E is 0, Q holds.

use super::{sigs, Internals, Model, Reg, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "d_latch",
    label: "DLATCH",
    name: "D latch",
    inputs: &["D", "E"],
    outputs: &["Q"],
    sequential: true,
    transistors: 22,
    summary: "Q follows D while E is 1, holds when E is 0.",
};

#[derive(Default)]
pub struct DLatch {
    q: bool,
}

impl Model for DLatch {
    fn n_in(&self) -> usize {
        2
    }
    fn n_out(&self) -> usize {
        1
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        match (ins[1].as_bool(), ins[0].as_bool()) {
            (Some(false), _) => {}
            (Some(true), Some(d)) => self.q = d,
            _ => return vec![Sig::X],
        }
        sigs(&[self.q])
    }
    fn internals(&self) -> Internals {
        Internals {
            regs: vec![Reg {
                name: "Q".into(),
                bits: sigs(&[self.q]),
            }],
            mems: vec![],
        }
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(DLatch::default())
}

pub fn component() -> super::Component {
    super::Component {
        spec: SPEC,
        make: model,
    }
}

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn transparent_then_frozen() {
        // D E -> Q
        script(
            &mut *super::model(),
            &[
                ("11", "1"), // open: follows
                ("01", "0"), // still follows
                ("11", "1"),
                ("10", "1"), // close with Q = 1
                ("00", "1"), // D changes, Q holds
                ("01", "0"), // reopen
            ],
        );
    }
}
