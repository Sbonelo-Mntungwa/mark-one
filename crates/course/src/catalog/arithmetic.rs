//! Chapter: Arithmetic.

use crate::level::{lv, s, Level, Test};

pub const CHAPTER: &str = "Arithmetic";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("half_adder", "Half adder", CHAPTER, "HA", s(&["A", "B"]), s(&["S", "C"]), true, 22, Test::Comb,
            "Add two bits. S is the sum bit, C is the carry. 1 + 1 = 10 in binary, so S=0 and C=1."),
        lv("full_adder", "Full adder", CHAPTER, "FA", s(&["A", "B", "CIN"]), s(&["S", "COUT"]), true, 50, Test::Comb,
            "Add three bits: A, B and a carry coming in, CIN. S is the sum bit, COUT the carry out."),
    ]
}
