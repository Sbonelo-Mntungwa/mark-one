//! Chapter: Transistors.

use crate::level::{lv, s, Level, Test};

pub const CHAPTER: &str = "Transistors";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("not", "Inverter", CHAPTER, "NOT", s(&["A"]), s(&["Y"]), true, 2, Test::Comb,
            "Build an inverter from transistors: Y is 1 when A is 0, and 0 when A is 1. Y must be driven to 1 or 0 for every input, never left floating."),
        lv("nand", "NAND", CHAPTER, "NAND", s(&["A", "B"]), s(&["Y"]), true, 4, Test::Comb,
            "Y is 0 only when A and B are both 1. Build it from transistors."),
        lv("nor", "NOR", CHAPTER, "NOR", s(&["A", "B"]), s(&["Y"]), true, 4, Test::Comb,
            "Y is 1 only when A and B are both 0. Build it from transistors."),
    ]
}
