//! Chapter: Software.

use crate::level::{io_machine, lv, Level, Test};

pub const CHAPTER: &str = "Software";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("prog_answer", "The answer", CHAPTER, "P1", io_machine().0, io_machine().1, false, 4, Test::Program(vec![42]),
            "Write a program that outputs 42, then halts. It runs on the computer you built. Operands are 4 bits: 0 to 15."),
        lv("prog_countdown", "Countdown", CHAPTER, "P2", io_machine().0, io_machine().1, false, 25, Test::Program((0..=9).rev().collect()),
            "Output 9, 8, 7 and so on down to 0, then halt."),
        lv("prog_powers", "Powers of two", CHAPTER, "P3", io_machine().0, io_machine().1, false, 28, Test::Program((0..8).map(|i| 1u8 << i).collect()),
            "Output 1, 2, 4, 8 up to 128, then halt."),
        lv("prog_table", "Times table", CHAPTER, "P4", io_machine().0, io_machine().1, false, 74, Test::Program((1..=10).map(|i| 7 * i).collect()),
            "Output the 7 times table: 7, 14, 21 up to 70, then halt. ROM holds 16 instructions, so space is tight."),
    ]
}
