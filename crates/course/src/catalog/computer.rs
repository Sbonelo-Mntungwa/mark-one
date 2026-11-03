//! Chapter: The computer.

use crate::level::{io_machine, lv, Level, Test};

pub const CHAPTER: &str = "The computer";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("computer", "The computer", CHAPTER, "MK1", io_machine().0, io_machine().1, false, 12126, Test::Machine,
            "Build a computer around your CPU. A ROM holds the program: give it an address, it returns the byte stored there. Diagnostics load their own test programs into your ROM and check OUT and HALT every cycle."),
    ]
}
