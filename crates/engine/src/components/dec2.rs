//! 2-to-4 decoder: with EN at 1, only output Y[A] is 1. With EN at 0, all are 0.

use super::{to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "dec2",
    label: "DEC2",
    name: "2-to-4 decoder",
    inputs: &["A0", "A1", "EN"],
    outputs: &["Y0", "Y1", "Y2", "Y3"],
    sequential: false,
    transistors: 40,
    summary: "Y[A] = EN, every other Y is 0.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let a = to_num(&b[..2]) as usize;
    (0..1 << 2).map(|i| b[2] && i == a).collect()
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn one_hot_when_enabled() {
        exhaustive("dec2", |b| {
            let a = super::to_num(&b[..2]);
            (0..1u32 << 2).map(|i| b[2] && 1 << i == 1 << a).collect()
        });
    }

    #[test]
    fn exactly_one_line_when_enabled() {
        let mut m = super::model();
        for a in 0..1u32 << 2 {
            let mut ins = super::super::from_num(a, 2);
            ins.push(true);
            let out = super::super::bools(&m.eval(&super::super::sigs(&ins))).unwrap();
            assert_eq!(out.iter().filter(|&&x| x).count(), 1);
        }
    }
}
