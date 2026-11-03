//! Mixed-level simulator.
//!
//! Transistors (NMOS/PMOS) are simulated at switch level with two strengths
//! (see `switch`). Chips are directional behavioural models evaluated in
//! Gauss-Seidel order until nothing changes.
//!
//! This is a functional, zero-delay simulator: it checks what a circuit
//! computes, not when. There are no gate delays, setup or hold times, or
//! metastability. Because chips are evaluated one at a time, a design that
//! depends on a glitch or a race can behave differently depending on the
//! order chips are evaluated; `set_reverse_order` exposes that so
//! diagnostics can reject such designs.

mod netlist;
mod switch;
#[cfg(test)]
mod tests;
pub mod timed;
pub(crate) mod union_find;

use std::collections::HashMap;

use crate::components::{Internals, Model};
use crate::signal::Sig;

enum Src {
    Const(Sig),
    Input(usize),
    ChipOut(usize, usize),
}

struct Fet {
    p: bool,
    g: usize,
    a: usize,
    b: usize,
}

struct ChipInst {
    /// Id of the part this chip is on the board.
    id: String,
    model: Box<dyn Model>,
    ins: Vec<usize>,
    out: Vec<Sig>,
    /// Gate delays from an input change to the outputs (timing mode).
    delay: u32,
    /// The clock input of an edge-triggered chip, checked for setup
    /// violations in timing mode.
    clk: Option<usize>,
}

pub struct Sim {
    drivers: Vec<Vec<Src>>,
    fets: Vec<Fet>,
    chips: Vec<ChipInst>,
    out_nets: Vec<Option<usize>>,
    pin_nets: Vec<(String, Vec<usize>)>,
    inputs: Vec<Sig>,
    val: Vec<Sig>,
    floating: Vec<bool>,
    /// Driven, but only through a wrong-polarity transistor (see `switch`).
    degraded: Vec<bool>,
    /// Settles each net has spent undriven, for charge leakage.
    undriven: Vec<u32>,
    /// Evaluate chips last to first.
    reverse: bool,
    /// A group of nets is pulled to 1 and 0 at the same time.
    pub short: bool,
    /// The circuit never settled (oscillation).
    pub unstable: bool,
    /// Event state while in timing mode.
    timing: Option<timed::State>,
}

const MAX_SWEEPS: usize = 200;

/// Stored charge on an undriven net survives this many settles, then leaks
/// to X. It stands in for leakage: dynamic storage must be refreshed.
pub const CHARGE_SETTLES: u32 = 8;

impl Sim {
    pub fn set_inputs(&mut self, ins: &[bool]) {
        for (k, v) in self.inputs.iter_mut().enumerate() {
            *v = Sig::from_bool(ins.get(k).copied().unwrap_or(false));
        }
    }

    /// Inputs that may be unknown or floating, as from a parent circuit.
    pub fn set_input_sigs(&mut self, ins: &[Sig]) {
        for (k, v) in self.inputs.iter_mut().enumerate() {
            *v = ins.get(k).copied().unwrap_or(Sig::Z);
        }
    }

    /// Evaluate chips in reverse order. A sound design settles the same way
    /// in either order.
    pub fn set_reverse_order(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    /// Propagate until every chip output is stable.
    pub fn settle(&mut self) {
        self.timing = None;
        self.unstable = false;
        self.age_charge_before();
        self.sweep();
        self.age_charge_after();
    }

    /// Stored charge that has sat undriven too long leaks to X.
    fn age_charge_before(&mut self) {
        for (v, &age) in self.val.iter_mut().zip(&self.undriven) {
            if age >= CHARGE_SETTLES {
                *v = Sig::X;
            }
        }
    }

    fn age_charge_after(&mut self) {
        for (age, &f) in self.undriven.iter_mut().zip(&self.floating) {
            *age = if f { age.saturating_add(1) } else { 0 };
        }
    }

    fn sweep(&mut self) {
        self.resolve();
        let n = self.chips.len();
        for _ in 0..MAX_SWEEPS {
            let mut changed = false;
            for k in 0..n {
                let i = if self.reverse { n - 1 - k } else { k };
                let ins: Vec<Sig> = self.chips[i].ins.iter().map(|&n| self.val[n]).collect();
                let out = self.chips[i].model.eval(&ins);
                if out != self.chips[i].out {
                    self.chips[i].out = out;
                    changed = true;
                    self.resolve();
                }
            }
            if !changed {
                return;
            }
        }
        self.unstable = true;
    }

    /// Level outputs. A floating output reads as Z even if it still holds
    /// charge, and a degraded output reads as X: it is not a valid logic level.
    pub fn outputs(&self) -> Vec<Sig> {
        self.out_nets
            .iter()
            .map(|n| match n {
                Some(n) if self.degraded[*n] => Sig::X,
                Some(n) if !self.floating[*n] => self.val[*n],
                _ => Sig::Z,
            })
            .collect()
    }

    /// Which level outputs are driven only through a wrong-polarity transistor.
    pub fn degraded_outputs(&self) -> Vec<bool> {
        self.out_nets
            .iter()
            .map(|n| n.is_some_and(|n| self.degraded[n]))
            .collect()
    }

    /// Value on every pin of every part, for drawing.
    pub fn pin_values(&self) -> HashMap<String, Vec<Sig>> {
        self.pin_nets
            .iter()
            .map(|(id, nets)| (id.clone(), nets.iter().map(|&n| self.val[n]).collect()))
            .collect()
    }

    /// Internal state of every chip that stores any, by part id, in board
    /// order. Custom components report what their own chips store, named
    /// `inner-part/REG`.
    pub fn internals(&self) -> Vec<(String, Internals)> {
        self.chips
            .iter()
            .map(|c| (c.id.clone(), c.model.internals()))
            .filter(|(_, s)| !s.is_empty())
            .collect()
    }

    /// Value on pin `pin` of part `part`, if both exist.
    pub fn pin_value(&self, part: &str, pin: usize) -> Option<Sig> {
        self.net_of(part, pin).map(|n| self.val[n])
    }

    /// The net pin `pin` of part `part` is on, if both exist.
    pub fn net_of(&self, part: &str, pin: usize) -> Option<usize> {
        let (_, nets) = self.pin_nets.iter().find(|(id, _)| id == part)?;
        nets.get(pin).copied()
    }

    /// Value of net `n` right now.
    pub fn net_value(&self, n: usize) -> Sig {
        self.val[n]
    }
}
