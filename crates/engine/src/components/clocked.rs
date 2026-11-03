//! Edge-triggered state with master-slave semantics, shared by every
//! clocked component. The last input is CLK. While CLK is 0 the other inputs
//! are sampled (the master is open); when CLK rises 0 -> 1 the sample is
//! committed (the slave takes it). Changing inputs while CLK is 1 does nothing.

use super::{Internals, Model, Reg};
use crate::signal::Sig;

pub struct Clocked {
    n_in: usize,
    n_out: usize,
    q: Vec<Sig>,
    master: Option<Vec<Sig>>,
    prev_clk: bool,
    /// What a debugger calls the stored state.
    name: &'static str,
    /// Next state from the sampled inputs and the current state.
    next: fn(&[Sig], &[Sig]) -> Vec<Sig>,
    /// Outputs from the current state.
    outs: fn(&[Sig]) -> Vec<Sig>,
}

impl Clocked {
    pub fn new(
        n_in: usize,
        n_out: usize,
        init: Vec<Sig>,
        next: fn(&[Sig], &[Sig]) -> Vec<Sig>,
        outs: fn(&[Sig]) -> Vec<Sig>,
    ) -> Clocked {
        Clocked {
            n_in,
            n_out,
            q: init,
            master: None,
            prev_clk: false,
            name: "Q",
            next,
            outs,
        }
    }

    /// Name the stored state for debuggers (the default is Q).
    pub fn named(mut self, name: &'static str) -> Clocked {
        self.name = name;
        self
    }
}

/// Outputs equal to the stored state.
pub fn state(q: &[Sig]) -> Vec<Sig> {
    q.to_vec()
}

impl Model for Clocked {
    fn n_in(&self) -> usize {
        self.n_in
    }
    fn n_out(&self) -> usize {
        self.n_out
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        let n = ins.len();
        if let Some(clk) = ins[n - 1].as_bool() {
            if !clk {
                self.master = Some(ins[..n - 1].to_vec());
            } else if !self.prev_clk {
                if let Some(m) = &self.master {
                    self.q = (self.next)(m, &self.q);
                }
            }
            self.prev_clk = clk;
        }
        (self.outs)(&self.q)
    }
    fn internals(&self) -> Internals {
        Internals {
            regs: vec![Reg {
                name: self.name.into(),
                bits: self.q.clone(),
            }],
            mems: vec![],
        }
    }
}
