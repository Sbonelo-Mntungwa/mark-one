//! Chapter: Memory.

use crate::level::{bus, cat, lv, reg_init, s, Level, Test};

pub const CHAPTER: &str = "Memory";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("sr_latch", "SR latch", CHAPTER, "SR", s(&["S", "R"]), s(&["Q", "QN"]), true, 8, Test::Sr,
            "S sets Q to 1, R resets it to 0, and with both at 0 the latch remembers. QN is NOT Q. Diagnostics never raise S and R together."),
        lv("d_latch", "D latch", CHAPTER, "DLATCH", s(&["D", "E"]), s(&["Q"]), true, 22,
            Test::Clocked { ctrl: 1, init: vec![vec![false, true], vec![false, false]], steps: 160 },
            "A latch with a gate. While E is 1, Q follows D. When E drops to 0, Q holds its last value."),
        lv("dff", "D flip-flop", CHAPTER, "DFF", s(&["D", "CLK"]), s(&["Q"]), true, 46,
            Test::Clocked { ctrl: 1, init: vec![vec![false, false], vec![false, true], vec![false, false]], steps: 200 },
            "Q takes the value of D only at the instant CLK rises from 0 to 1, and ignores D the rest of the time. Use the Clock and Pulse buttons to drive CLK."),
        lv("reg_bit", "Register bit", CHAPTER, "REG1", s(&["D", "LOAD", "CLK"]), s(&["Q"]), true, 60,
            Test::Clocked { ctrl: 2, init: reg_init(3, 1, 2), steps: 200 },
            "One bit of a register. On each rising clock edge: if LOAD is 1, store D, otherwise keep the current value."),
        lv("reg4", "Nibble register", CHAPTER, "REG4", cat(&[bus("D", 4), s(&["LOAD", "CLK"])]), bus("Q", 4), true, 240,
            Test::Clocked { ctrl: 5, init: reg_init(6, 4, 5), steps: 200 },
            "Four bits that load together."),
        lv("reg8", "Byte register", CHAPTER, "REG8", cat(&[bus("D", 8), s(&["LOAD", "CLK"])]), bus("Q", 8), true, 480,
            Test::Clocked { ctrl: 9, init: reg_init(10, 8, 9), steps: 200 },
            "A full byte of memory."),
    ]
}
