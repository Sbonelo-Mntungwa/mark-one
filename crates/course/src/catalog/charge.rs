//! Chapter: Charge.

use crate::level::{lv, s, Level, Test};

pub const CHAPTER: &str = "Charge";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("power", "Live wire", CHAPTER, "PWR", vec![], s(&["HI", "LO"]), false, 0, Test::Comb,
            "Every circuit starts with charge. VCC is a line held at 1 (charged). GND is a line held at 0 (drained). Wire VCC to HI and GND to LO."),
        lv("relay", "Relay", CHAPTER, "RLY", s(&["A"]), s(&["Y"]), false, 0, Test::Comb,
            "Wires carry whatever charge is on them. Connect input A to output Y, then click A to toggle it and watch the charge flow."),
    ]
}
