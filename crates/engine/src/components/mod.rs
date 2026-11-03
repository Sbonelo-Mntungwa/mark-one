//! The component library. Each component lives in its own file with its pin
//! specification, its behavioural model, and tests that check the model
//! against an independent statement of what the component must do.
//!
//! Components are the building blocks of every circuit: the learning path
//! (`game`) unlocks them one by one, free build offers all of them.

use serde::Serialize;

use crate::signal::Sig;

pub mod clocked;
#[cfg(test)]
pub mod testkit;

pub mod accum;
pub mod add4;
pub mod add8;
pub mod alu8;
pub mod and;
pub mod cpu;
pub mod ctrl;
pub mod d_latch;
pub mod dec2;
pub mod dec4;
pub mod dff;
pub mod flip4;
pub mod flip8;
pub mod full_adder;
pub mod half_adder;
pub mod inc4;
pub mod mux;
pub mod mux4;
pub mod mux8;
pub mod nand;
pub mod nor;
pub mod not;
pub mod or;
pub mod pc4;
pub mod ram;
pub mod reg4;
pub mod reg8;
pub mod reg_bit;
pub mod rom;
pub mod sr_latch;
pub mod xor;
pub mod zero8;

pub use cpu::{op, run_program, Cpu, Run};
pub use rom::Rom;

/// A component's behaviour: inputs in, outputs out. Stateful components keep
/// their state between calls.
pub trait Model: Send {
    fn n_in(&self) -> usize;
    fn n_out(&self) -> usize;
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig>;
    /// Internal state a debugger can show: registers and memories. Reading
    /// it never changes the component. Stateless components have none.
    fn internals(&self) -> Internals {
        Internals::default()
    }
    /// Gate delays from inputs to outputs in timing mode, if the component
    /// knows its own (custom components measure theirs). Library components
    /// use the table in `timing`.
    fn delay(&self) -> Option<u32> {
        None
    }
}

/// A register inside a component. Bits are little-endian: index 0 is bit 0.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Reg {
    pub name: String,
    pub bits: Vec<Sig>,
}

/// A memory inside a component, one byte per address.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Mem {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Everything a component stores, by name.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Internals {
    pub regs: Vec<Reg>,
    pub mems: Vec<Mem>,
}

impl Internals {
    pub fn is_empty(&self) -> bool {
        self.regs.is_empty() && self.mems.is_empty()
    }

    /// The register called `name`, as a number, if every bit is known.
    pub fn reg(&self, name: &str) -> Option<u32> {
        let r = self.regs.iter().find(|r| r.name == name)?;
        bools(&r.bits).map(|b| to_num(&b))
    }

    pub fn reg_named(name: &str, bits: Vec<Sig>) -> Reg {
        Reg {
            name: name.into(),
            bits,
        }
    }
}

/// What a component looks like from the outside.
pub struct Spec {
    pub id: &'static str,
    /// Short name printed on the chip.
    pub label: &'static str,
    pub name: &'static str,
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    /// Clocked or otherwise stateful.
    pub sequential: bool,
    /// Transistor count of the reference design.
    pub transistors: u32,
    pub summary: &'static str,
}

pub struct Component {
    pub spec: Spec,
    pub make: fn() -> Box<dyn Model>,
}

/// Every component, in build order: each can be made from those before it.
pub fn catalog() -> &'static [Component] {
    static C: std::sync::OnceLock<Vec<Component>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        vec![
            not::component(),
            nand::component(),
            nor::component(),
            and::component(),
            or::component(),
            xor::component(),
            mux::component(),
            half_adder::component(),
            full_adder::component(),
            add4::component(),
            add8::component(),
            flip4::component(),
            flip8::component(),
            zero8::component(),
            alu8::component(),
            sr_latch::component(),
            d_latch::component(),
            dff::component(),
            reg_bit::component(),
            reg4::component(),
            reg8::component(),
            accum::component(),
            inc4::component(),
            mux4::component(),
            mux8::component(),
            pc4::component(),
            dec2::component(),
            dec4::component(),
            ram::component(1),
            ram::component(2),
            ram::component(3),
            ram::component(4),
            ctrl::component(),
            cpu::component(),
        ]
    })
}

pub fn find(id: &str) -> Option<&'static Component> {
    catalog().iter().find(|c| c.spec.id == id)
}

/// A fresh instance of a component's model.
pub fn model(id: &str) -> Option<Box<dyn Model>> {
    find(id).map(|c| (c.make)())
}

pub fn bools(s: &[Sig]) -> Option<Vec<bool>> {
    s.iter().map(|x| x.as_bool()).collect()
}

pub fn sigs(b: &[bool]) -> Vec<Sig> {
    b.iter().map(|&x| Sig::from_bool(x)).collect()
}

/// Little-endian: index 0 is bit 0.
pub fn to_num(b: &[bool]) -> u32 {
    b.iter()
        .enumerate()
        .fold(0, |acc, (i, &x)| acc | ((x as u32) << i))
}

pub fn from_num(v: u32, n: usize) -> Vec<bool> {
    (0..n).map(|i| (v >> i) & 1 == 1).collect()
}

/// A floating input reads as unknown.
pub fn norm(s: Sig) -> Sig {
    if s == Sig::Z {
        Sig::X
    } else {
        s
    }
}

/// Evaluate a pure function on possibly-unknown inputs. If every possible
/// value of the unknown bits gives the same output bit, that bit is known.
/// With more than six unknown inputs every output is reported as X: the
/// result is then pessimistic (unknown where hardware might settle), never
/// wrong.
pub fn three_valued(ins: &[Sig], n_out: usize, f: &dyn Fn(&[bool]) -> Vec<bool>) -> Vec<Sig> {
    let unknown: Vec<usize> = (0..ins.len())
        .filter(|&i| ins[i].as_bool().is_none())
        .collect();
    let mut b: Vec<bool> = ins.iter().map(|s| s.as_bool().unwrap_or(false)).collect();
    if unknown.is_empty() {
        return sigs(&f(&b));
    }
    if unknown.len() > 6 {
        return vec![Sig::X; n_out];
    }
    let mut res: Option<Vec<Sig>> = None;
    for m in 0..(1u32 << unknown.len()) {
        for (j, &i) in unknown.iter().enumerate() {
            b[i] = (m >> j) & 1 == 1;
        }
        let o = sigs(&f(&b));
        res = Some(match res {
            None => o,
            Some(r) => r
                .into_iter()
                .zip(o)
                .map(|(x, y)| if x == y { x } else { Sig::X })
                .collect(),
        });
    }
    res.unwrap_or_default()
}

/// Stateless logic defined by a truth function.
pub struct Comb {
    pub n_in: usize,
    pub n_out: usize,
    pub f: fn(&[bool]) -> Vec<bool>,
}

impl Model for Comb {
    fn n_in(&self) -> usize {
        self.n_in
    }
    fn n_out(&self) -> usize {
        self.n_out
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        let f = self.f;
        three_valued(ins, self.n_out, &|b| f(b))
    }
}

/// Declare a combinational component: its spec, its truth function, and a
/// maker that instantiates it.
#[macro_export]
macro_rules! comb_component {
    ($spec:expr, $f:path) => {
        pub fn model() -> Box<dyn $crate::components::Model> {
            let s = &SPEC;
            Box::new($crate::components::Comb {
                n_in: s.inputs.len(),
                n_out: s.outputs.len(),
                f: $f,
            })
        }
        pub fn component() -> $crate::components::Component {
            $crate::components::Component {
                spec: $spec,
                make: model,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_models_match_their_specs() {
        let mut seen = std::collections::BTreeSet::new();
        for c in catalog() {
            assert!(seen.insert(c.spec.id), "duplicate id {}", c.spec.id);
            let m = (c.make)();
            assert_eq!(m.n_in(), c.spec.inputs.len(), "{} inputs", c.spec.id);
            assert_eq!(m.n_out(), c.spec.outputs.len(), "{} outputs", c.spec.id);
        }
    }

    #[test]
    fn three_valued_resolves_dominant_inputs() {
        let and = |b: &[bool]| vec![b[0] && b[1]];
        assert_eq!(three_valued(&[Sig::L, Sig::X], 1, &and), vec![Sig::L]);
        assert_eq!(three_valued(&[Sig::H, Sig::X], 1, &and), vec![Sig::X]);
    }

    #[test]
    fn numbers_are_little_endian() {
        assert_eq!(to_num(&[true, false, true]), 5);
        assert_eq!(from_num(6, 4), vec![false, true, true, false]);
    }
}
