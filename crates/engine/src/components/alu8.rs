//! 8-bit ALU. SUB=0: S = A + B. SUB=1: S = A - B (mod 256).
//! C is the carry out of bit 7; when subtracting it is 1 when A >= B as
//! unsigned numbers (no borrow: the 6502 and ARM convention, the inverse of
//! x86's borrow flag). Z is 1 when S is 0. There is no overflow (V) or
//! negative (N) flag and no logic operations: N would be S7, and V the carry
//! into bit 7 XOR the carry out of it.

use super::{from_num, to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "alu8",
    label: "ALU8",
    name: "Arithmetic logic unit",
    inputs: &[
        "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "B0", "B1", "B2", "B3", "B4", "B5", "B6",
        "B7", "SUB",
    ],
    outputs: &["S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "C", "Z"],
    sequential: false,
    transistors: 572,
    summary: "S = A + B, or A - B when SUB is 1. C is carry, Z is zero.",
};

/// Two's complement: A - B = A + NOT B + 1.
pub fn f(b: &[bool]) -> Vec<bool> {
    let sub = b[16];
    let bb = to_num(&b[8..16]) ^ if sub { 0xff } else { 0 };
    let r = to_num(&b[0..8]) + bb + sub as u32;
    let mut o = from_num(r, 8);
    o.push((r >> 8) & 1 == 1);
    o.push(r & 0xff == 0);
    o
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    /// Stated with ordinary wrapping arithmetic, not two's complement.
    fn spec(b: &[bool]) -> Vec<bool> {
        let a = super::to_num(&b[..8]) as u8;
        let x = super::to_num(&b[8..16]) as u8;
        let (s, c) = if b[16] {
            (a.wrapping_sub(x), a >= x)
        } else {
            a.overflowing_add(x)
        };
        let mut o = super::from_num(s as u32, 8);
        o.push(c);
        o.push(s == 0);
        o
    }

    #[test]
    fn every_input_matches_wrapping_arithmetic() {
        exhaustive("alu8", spec);
    }

    #[test]
    fn signed_view() {
        // 3 - 5 = -2, which is 254 as an unsigned byte.
        let mut ins = super::from_num(3, 8);
        ins.extend(super::from_num(5, 8));
        ins.push(true);
        let out = super::super::bools(&super::model().eval(&super::super::sigs(&ins))).unwrap();
        assert_eq!(super::to_num(&out[..8]) as u8 as i8, -2);
        assert!(!out[8], "3 < 5 borrows, so C is 0");
    }
}
