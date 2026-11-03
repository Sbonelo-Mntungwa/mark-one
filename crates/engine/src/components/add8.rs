//! 8-bit ripple-carry adder: S = A + B + CIN, COUT is the carry out of the top bit.

use super::{from_num, to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "add8",
    label: "ADD8",
    name: "Byte adder",
    inputs: &[
        "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "B0", "B1", "B2", "B3", "B4", "B5", "B6",
        "B7", "CIN",
    ],
    outputs: &["S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "COUT"],
    sequential: false,
    transistors: 400,
    summary: "Adds two 8-bit numbers with carry.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let r = to_num(&b[..8]) + to_num(&b[8..16]) + b[16] as u32;
    from_num(r, 9)
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    /// Column-by-column addition, the way it is done by hand.
    fn ripple(b: &[bool]) -> Vec<bool> {
        let mut c = b[16];
        let mut s = vec![];
        for i in 0..8 {
            let (x, y) = (b[i], b[8 + i]);
            s.push(x ^ y ^ c);
            c = (x && y) || (c && (x ^ y));
        }
        s.push(c);
        s
    }

    #[test]
    fn matches_column_addition_on_every_input() {
        exhaustive("add8", ripple);
    }

    #[test]
    fn known_sums() {
        let mut m = super::model();
        // 200 + 100 = 300 = 256 + 44
        let (a, b) = (200, 100);
        let mut ins = super::from_num(a, 8);
        ins.extend(super::from_num(b, 8));
        ins.push(false);
        let out = super::super::bools(&m.eval(&super::super::sigs(&ins))).unwrap();
        assert_eq!(super::to_num(&out), a + b);
    }
}
