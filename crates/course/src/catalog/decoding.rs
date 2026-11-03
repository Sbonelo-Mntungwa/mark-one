//! Chapter: Decoding.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "Decoding";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("dec2", "2-to-4 decoder", CHAPTER, "DEC2", s(&["A0", "A1", "EN"]), bus("Y", 4), true, 40, Test::Comb,
            "A decoder turns a binary number into one active line. With EN at 1, exactly one of Y0 to Y3 is 1: the one numbered by A. With EN at 0, all are 0."),
        lv("dec4", "4-to-16 decoder", CHAPTER, "DEC4", cat(&[bus("A", 4), s(&["EN"])]), bus("Y", 16), true, 200, Test::Comb,
            "Sixteen lines, one for each value of A. With EN at 1, only Y[A] is 1. With EN at 0, all are 0."),
    ]
}
