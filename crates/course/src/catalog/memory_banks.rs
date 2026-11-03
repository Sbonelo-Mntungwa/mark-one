//! Chapter: Memory banks.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "Memory banks";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("ram2", "Memory pair", CHAPTER, "RAM2", cat(&[s(&["A0"]), bus("D", 8), s(&["WE", "CLK"])]), bus("Q", 8), true, 1086,
            Test::Clocked { ctrl: 10, init: vec![], steps: 300 },
            "Random access memory: two bytes, picked by address A0. Q always shows the byte at the current address. On a rising clock edge with WE (write enable) at 1, D is written to that address."),
        lv("ram4", "Memory quad", CHAPTER, "RAM4", cat(&[bus("A", 2), bus("D", 8), s(&["WE", "CLK"])]), bus("Q", 8), true, 2298,
            Test::Clocked { ctrl: 11, init: vec![], steps: 300 },
            "Four bytes, picked by A0 and A1. Same rules as the memory pair."),
        lv("ram8", "Memory bank", CHAPTER, "RAM8", cat(&[bus("A", 3), bus("D", 8), s(&["WE", "CLK"])]), bus("Q", 8), true, 4722,
            Test::Clocked { ctrl: 12, init: vec![], steps: 300 },
            "Eight bytes, picked by A0 to A2."),
        lv("ram16", "Main memory", CHAPTER, "RAM16", cat(&[bus("A", 4), bus("D", 8), s(&["WE", "CLK"])]), bus("Q", 8), true, 9570,
            Test::Clocked { ctrl: 13, init: vec![], steps: 300 },
            "Sixteen bytes, picked by A0 to A3: all the data memory your computer will have."),
    ]
}
