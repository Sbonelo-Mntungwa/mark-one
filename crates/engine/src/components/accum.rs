//! Accumulator machine: on each rising edge, ACC becomes ACC + B, or
//! ACC - B when SUB is 1 (mod 256). Z is 1 while ACC is 0. Starts at 0.

use super::clocked::Clocked;
use super::{bools, from_num, sigs, to_num, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "accum",
    label: "ACC8",
    name: "Accumulator machine",
    inputs: &["B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "SUB", "CLK"],
    outputs: &[
        "ACC0", "ACC1", "ACC2", "ACC3", "ACC4", "ACC5", "ACC6", "ACC7", "Z",
    ],
    sequential: true,
    transistors: 1096,
    summary: "ACC = ACC +/- B on each clock edge.",
};

fn next(m: &[Sig], q: &[Sig]) -> Vec<Sig> {
    match (bools(m), bools(q)) {
        (Some(m), Some(q)) => {
            let a = to_num(&q);
            let b = to_num(&m[0..8]);
            let r = if m[8] {
                a.wrapping_sub(b)
            } else {
                a.wrapping_add(b)
            };
            sigs(&from_num(r & 0xff, 8))
        }
        _ => vec![Sig::X; 8],
    }
}

fn outs(q: &[Sig]) -> Vec<Sig> {
    let mut o = q.to_vec();
    o.push(match bools(q) {
        Some(b) => Sig::from_bool(b.iter().all(|x| !x)),
        None => Sig::X,
    });
    o
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(10, 9, vec![Sig::L; 8], next, outs).named("ACC"))
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
    use crate::components::{bools, from_num, to_num};

    #[test]
    fn running_total_matches_u8_arithmetic() {
        let mut m = super::model();
        let mut acc: u8 = 0;
        let mut seed = 0x2545_f491u32;
        for _ in 0..500 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let (b, sub) = ((seed & 0xff) as u8, seed & 0x100 != 0);
            acc = if sub {
                acc.wrapping_sub(b)
            } else {
                acc.wrapping_add(b)
            };
            let mut ins = from_num(b as u32, 8);
            ins.push(sub);
            let out = bools(&clock(&mut *m, &ins)).unwrap();
            assert_eq!(to_num(&out[..8]) as u8, acc);
            assert_eq!(out[8], acc == 0);
        }
    }
}
