//! Chapter: Wide buses.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "Wide buses";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("add4", "Nibble adder", CHAPTER, "ADD4", cat(&[bus("A", 4), bus("B", 4), s(&["CIN"])]), cat(&[bus("S", 4), s(&["COUT"])]), true, 200, Test::Comb,
            "Add two 4-bit numbers plus CIN. A0 and B0 are the least significant bits. COUT is 1 when the sum is above 15."),
        lv("add8", "Byte adder", CHAPTER, "ADD8", cat(&[bus("A", 8), bus("B", 8), s(&["CIN"])]), cat(&[bus("S", 8), s(&["COUT"])]), true, 400, Test::Comb,
            "Add two bytes plus CIN. Numbers from 0 to 255; COUT is 1 when the sum is above 255."),
        lv("flip4", "Nibble flipper", CHAPTER, "FLIP4", cat(&[bus("X", 4), s(&["INV"])]), bus("Y", 4), true, 64, Test::Comb,
            "Conditionally invert a bus. When INV is 1 every bit flips. When INV is 0 the bus passes through unchanged."),
        lv("flip8", "Byte flipper", CHAPTER, "FLIP8", cat(&[bus("X", 8), s(&["INV"])]), bus("Y", 8), true, 128, Test::Comb,
            "The same, eight bits wide."),
        lv("zero8", "Zero detector", CHAPTER, "ZERO8", bus("X", 8), s(&["Z"]), true, 44, Test::Comb,
            "Z is 1 when all eight bits are 0. Processors use this flag to decide whether to branch."),
        lv("alu8", "Arithmetic logic unit", CHAPTER, "ALU8", cat(&[bus("A", 8), bus("B", 8), s(&["SUB"])]), cat(&[bus("S", 8), s(&["C", "Z"])]), true, 572, Test::Comb,
            "The ALU. SUB=0: S = A + B. SUB=1: S = A - B, wrapping around in 8 bits (3 - 5 = 254). C is the carry out of bit 7: when subtracting, C is 1 if A is at least B as unsigned numbers (no borrow, the 6502 and ARM convention; x86 inverts it). Z is 1 when S is 0. This ALU only adds and subtracts, and has no overflow or negative flag."),
    ]
}
