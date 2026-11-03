//! 4-bit ripple-carry adder: S = A + B + CIN, COUT is the carry out of the top bit.

use super::{from_num, to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "add4",
    label: "ADD4",
    name: "Nibble adder",
    inputs: &["A0", "A1", "A2", "A3", "B0", "B1", "B2", "B3", "CIN"],
    outputs: &["S0", "S1", "S2", "S3", "COUT"],
    sequential: false,
    transistors: 200,
    summary: "Adds two 4-bit numbers with carry.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let r = to_num(&b[..4]) + to_num(&b[4..8]) + b[8] as u32;
    from_num(r, 5)
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    /// Column-by-column addition, the way it is done by hand.
    fn ripple(b: &[bool]) -> Vec<bool> {
        let mut c = b[8];
        let mut s = vec![];
        for i in 0..4 {
            let (x, y) = (b[i], b[4 + i]);
            s.push(x ^ y ^ c);
            c = (x && y) || (c && (x ^ y));
        }
        s.push(c);
        s
    }

    #[test]
    fn matches_column_addition_on_every_input() {
        exhaustive("add4", ripple);
    }

    #[test]
    fn known_sums() {
        let mut m = super::model();
        // 7 + 9 = 16: S = 0000, COUT = 1
        let (a, b) = (7, 9);
        let mut ins = super::from_num(a, 4);
        ins.extend(super::from_num(b, 4));
        ins.push(false);
        let out = super::super::bools(&m.eval(&super::super::sigs(&ins))).unwrap();
        assert_eq!(super::to_num(&out), a + b);
    }
}
