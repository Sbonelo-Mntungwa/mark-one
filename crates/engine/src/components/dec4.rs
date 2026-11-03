//! 4-to-16 decoder: with EN at 1, only output Y[A] is 1. With EN at 0, all are 0.

use super::{to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "dec4",
    label: "DEC4",
    name: "4-to-16 decoder",
    inputs: &["A0", "A1", "A2", "A3", "EN"],
    outputs: &[
        "Y0", "Y1", "Y2", "Y3", "Y4", "Y5", "Y6", "Y7", "Y8", "Y9", "Y10", "Y11", "Y12", "Y13",
        "Y14", "Y15",
    ],
    sequential: false,
    transistors: 200,
    summary: "Y[A] = EN, every other Y is 0.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let a = to_num(&b[..4]) as usize;
    (0..1 << 4).map(|i| b[4] && i == a).collect()
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn one_hot_when_enabled() {
        exhaustive("dec4", |b| {
            let a = super::to_num(&b[..4]);
            (0..1u32 << 4).map(|i| b[4] && 1 << i == 1 << a).collect()
        });
    }

    #[test]
    fn exactly_one_line_when_enabled() {
        let mut m = super::model();
        for a in 0..1u32 << 4 {
            let mut ins = super::super::from_num(a, 4);
            ins.push(true);
            let out = super::super::bools(&m.eval(&super::super::sigs(&ins))).unwrap();
            assert_eq!(out.iter().filter(|&&x| x).count(), 1);
        }
    }
}
