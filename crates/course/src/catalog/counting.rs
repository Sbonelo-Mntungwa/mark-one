//! Chapter: Counting.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "Counting";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("inc4", "Incrementer", CHAPTER, "INC4", bus("A", 4), bus("Y", 4), true, 88, Test::Comb,
            "Count up by one. Y = A + 1, and 15 wraps around to 0."),
        lv("mux4", "Nibble selector", CHAPTER, "MUX4", cat(&[bus("A", 4), bus("B", 4), s(&["SEL"])]), bus("Y", 4), true, 56, Test::Comb,
            "Choose between two 4-bit buses. Y = A when SEL is 0, Y = B when SEL is 1."),
        lv("mux8", "Byte selector", CHAPTER, "MUX8", cat(&[bus("A", 8), bus("B", 8), s(&["SEL"])]), bus("Y", 8), true, 112, Test::Comb,
            "The same, a byte wide."),
        lv("pc4", "Program counter", CHAPTER, "PC4", cat(&[bus("D", 4), s(&["LOAD", "EN", "CLK"])]), bus("Q", 4), true, 384,
            Test::Clocked { ctrl: 6, init: vec![], steps: 300 },
            "The program counter holds the address of the next instruction. On each rising clock edge: if EN is 0, hold. Otherwise, if LOAD is 1, jump to D. Otherwise count up by one."),
    ]
}
