//! Chapter: Gates.

use crate::level::{lv, s, Level, Test};

pub const CHAPTER: &str = "Gates";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("and", "AND", CHAPTER, "AND", s(&["A", "B"]), s(&["Y"]), true, 6, Test::Comb,
            "Every design you verify becomes a chip in your fabricator. Build AND: Y is 1 only when A and B are both 1."),
        lv("or", "OR", CHAPTER, "OR", s(&["A", "B"]), s(&["Y"]), true, 6, Test::Comb,
            "Y is 1 when A or B (or both) is 1."),
        lv("xor", "XOR", CHAPTER, "XOR", s(&["A", "B"]), s(&["Y"]), true, 16, Test::Comb,
            "Y is 1 when A and B differ."),
        lv("mux", "Selector", CHAPTER, "MUX", s(&["A", "B", "SEL"]), s(&["Y"]), true, 14, Test::Comb,
            "A multiplexer picks one of two lines. Y = A when SEL is 0. Y = B when SEL is 1."),
    ]
}
