//! Chapter: Workshop.

use crate::level::{io_machine, lv, Level, Test};

pub const CHAPTER: &str = "Workshop";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("workshop", "Workshop", CHAPTER, "WS", io_machine().0, io_machine().1, false, 0, Test::Sandbox,
            "Free build. Every chip you fabricated is here. Write your own programs, extend the computer, or build something new. No diagnostics. Ideas: Fibonacci numbers, multiplying two numbers, a carry flag for numbers above 255."),
    ]
}
