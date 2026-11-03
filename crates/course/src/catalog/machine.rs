//! Chapter: The machine.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "The machine";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("accum", "Accumulator machine", CHAPTER, "ACC8", cat(&[bus("B", 8), s(&["SUB", "CLK"])]), cat(&[bus("ACC", 8), s(&["Z"])]), true, 1096,
            Test::Clocked { ctrl: 9, init: vec![], steps: 400 },
            "Your first machine. ACC starts at 0. On every rising clock edge it becomes ACC + B, or ACC - B when SUB is 1. Z is 1 while ACC is 0."),
    ]
}
