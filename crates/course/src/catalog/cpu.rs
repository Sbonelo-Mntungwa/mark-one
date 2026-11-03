//! Chapter: The CPU.

use crate::level::{bus, cat, lv, s, Level, Test};

pub const CHAPTER: &str = "The CPU";

#[rustfmt::skip]
pub fn levels() -> Vec<Level> {
    vec![
        lv("ctrl", "Control unit", CHAPTER, "CTRL", cat(&[bus("OP", 4), s(&["Z"])]), s(&["BIMM", "PASS", "SUB", "ACCW", "RAMW", "JUMP", "OUTW", "HALT"]), true, 242, Test::Comb,
            "The control unit reads the opcode and drives every switch in the CPU. BIMM: B is the operand, not RAM. PASS: ACC takes B directly instead of the ALU result. SUB: the ALU subtracts. ACCW, RAMW, OUTW: write ACC, RAM or OUT this cycle. JUMP: PC loads the operand. HALT: stop. Z is 1 when ACC is 0. The Instruction set panel lists which signals each opcode raises."),
        lv("cpu", "Central processing unit", CHAPTER, "CPU", cat(&[bus("I", 8), s(&["CLK"])]), cat(&[bus("PC", 4), bus("OUT", 8), s(&["HALT"])]), true, 11998, Test::Cpu,
            "Build the CPU. Each clock cycle it executes the instruction on I: the low 4 bits are the operand n, the high 4 bits the opcode. PC is the address of the instruction it wants next. OUT is the output register. Diagnostics act as program memory: they feed you the instruction stored at your PC."),
    ]
}
