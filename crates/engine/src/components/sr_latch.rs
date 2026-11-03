//! SR latch (NOR style): S sets Q, R resets Q, both 0 holds. Powers up reset.
//! S = R = 1 drives both outputs to 0, as two cross-coupled NOR gates do.
//! Releasing both at once from there is a race: a real latch ends up in
//! either state or goes metastable, so the model reports X until the next set
//! or reset.

use super::{sigs, Internals, Model, Reg, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "sr_latch",
    label: "SR",
    name: "SR latch",
    inputs: &["S", "R"],
    outputs: &["Q", "QN"],
    sequential: true,
    transistors: 8,
    summary: "S sets Q, R resets Q, both 0 holds.",
};

#[derive(Default)]
pub struct SrLatch {
    /// None after a release from S = R = 1.
    q: Option<bool>,
    forbidden: bool,
}

impl Model for SrLatch {
    fn n_in(&self) -> usize {
        2
    }
    fn n_out(&self) -> usize {
        2
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        let was_forbidden = std::mem::replace(&mut self.forbidden, false);
        match (ins[0].as_bool(), ins[1].as_bool()) {
            (Some(true), Some(true)) => {
                self.forbidden = true;
                return vec![Sig::L, Sig::L];
            }
            (Some(true), Some(false)) => self.q = Some(true),
            (Some(false), Some(true)) => self.q = Some(false),
            (Some(false), Some(false)) if was_forbidden => self.q = None,
            (Some(false), Some(false)) => {}
            _ => return vec![Sig::X, Sig::X],
        }
        match self.q {
            Some(q) => sigs(&[q, !q]),
            None => vec![Sig::X, Sig::X],
        }
    }
    fn internals(&self) -> Internals {
        Internals {
            regs: vec![Reg {
                name: "Q".into(),
                bits: vec![self.q.map_or(Sig::X, Sig::from_bool)],
            }],
            mems: vec![],
        }
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(SrLatch {
        q: Some(false),
        forbidden: false,
    })
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
    fn set_hold_reset_hold() {
        // S R -> Q QN
        script(
            &mut *super::model(),
            &[
                ("00", "01"), // powers up reset
                ("10", "10"), // set
                ("00", "10"), // hold 1
                ("01", "01"), // reset
                ("00", "01"), // hold 0
                ("11", "00"), // forbidden: both NOR outputs low
                ("00", "XX"), // released together: a race
                ("10", "10"), // a set recovers it
            ],
        );
    }

    #[test]
    fn cross_coupled_nor_transistors_hold_a_bit() {
        // Q = NOR(R, QN), QN = NOR(S, Q), each NOR as four transistors.
        let mut f = Fab::new(&["S", "R"], &["Q", "QN"]);
        f.p("R", "VCC", "m1")
            .p("QN", "m1", "Q")
            .n("R", "Q", "GND")
            .n("QN", "Q", "GND");
        f.p("S", "VCC", "m2")
            .p("Q", "m2", "QN")
            .n("S", "QN", "GND")
            .n("Q", "QN", "GND");
        // S R -> Q QN, in order. It powers up unknown, as silicon does, then
        // sets, holds, resets and holds.
        let rows = f.sequence(2, &["00", "10", "00", "01", "00", "10", "00"]);
        assert_eq!(rows, ["XX", "10", "10", "01", "01", "10", "10"]);
    }
}
