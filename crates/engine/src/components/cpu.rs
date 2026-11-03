//! The Mark One CPU: an 8-bit, single-cycle accumulator machine.
//! [I0..I7, CLK] -> [PC0..PC3, OUT0..OUT7, HALT]. The instruction on I
//! executes on each rising edge of CLK. One byte per instruction: opcode in
//! the high nibble, operand n in the low nibble. 16 bytes of internal RAM.

use super::{bools, from_num, sigs, to_num, Internals, Mem, Model, Reg, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "cpu",
    label: "CPU",
    name: "Central processing unit",
    inputs: &["I0", "I1", "I2", "I3", "I4", "I5", "I6", "I7", "CLK"],
    outputs: &[
        "PC0", "PC1", "PC2", "PC3", "OUT0", "OUT1", "OUT2", "OUT3", "OUT4", "OUT5", "OUT6", "OUT7",
        "HALT",
    ],
    sequential: true,
    transistors: 11998,
    summary: "Executes the instruction on I each clock edge. PC says which instruction it wants.",
};

/// Opcodes (high nibble of each instruction byte).
pub mod op {
    pub const NOP: u8 = 0;
    pub const LDI: u8 = 1;
    pub const ADI: u8 = 2;
    pub const ADD: u8 = 3;
    pub const SUB: u8 = 4;
    pub const LDA: u8 = 5;
    pub const STA: u8 = 6;
    pub const JMP: u8 = 7;
    pub const JZ: u8 = 8;
    pub const OUT: u8 = 9;
    pub const HLT: u8 = 15;
}

#[derive(Default, Clone)]
pub struct Cpu {
    pub pc: u8,
    pub acc: u8,
    pub out: u8,
    pub ram: [u8; 16],
    master: Option<Vec<Sig>>,
    prev_clk: bool,
}

impl Cpu {
    /// Execute one instruction.
    pub fn step(&mut self, instr: u8) {
        let n = instr & 15;
        let mut next = (self.pc + 1) & 15;
        match instr >> 4 {
            op::LDI => self.acc = n,
            op::ADI => self.acc = self.acc.wrapping_add(n),
            op::ADD => self.acc = self.acc.wrapping_add(self.ram[n as usize]),
            op::SUB => self.acc = self.acc.wrapping_sub(self.ram[n as usize]),
            op::LDA => self.acc = self.ram[n as usize],
            op::STA => self.ram[n as usize] = self.acc,
            op::JMP => next = n,
            op::JZ => {
                if self.acc == 0 {
                    next = n
                }
            }
            op::OUT => self.out = self.acc,
            op::HLT => next = self.pc,
            _ => {}
        }
        self.pc = next;
    }
}

impl Model for Cpu {
    fn n_in(&self) -> usize {
        9
    }
    fn n_out(&self) -> usize {
        13
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        if let Some(clk) = ins[8].as_bool() {
            if !clk {
                self.master = Some(ins[..8].to_vec());
            } else if !self.prev_clk {
                if let Some(i) = self.master.as_ref().and_then(|m| bools(m)) {
                    self.step(to_num(&i) as u8);
                }
            }
            self.prev_clk = clk;
        }
        let mut o = sigs(&from_num(self.pc as u32, 4));
        o.extend(sigs(&from_num(self.out as u32, 8)));
        o.push(match bools(&ins[4..8]) {
            Some(b) => Sig::from_bool(to_num(&b) == op::HLT as u32),
            None => Sig::X,
        });
        o
    }
    fn internals(&self) -> Internals {
        let reg = |name: &str, v: u8, n: usize| Reg {
            name: name.into(),
            bits: sigs(&from_num(v as u32, n)),
        };
        Internals {
            regs: vec![
                reg("PC", self.pc, 4),
                reg("ACC", self.acc, 8),
                reg("OUT", self.out, 8),
            ],
            mems: vec![Mem {
                name: "RAM".into(),
                bytes: self.ram.to_vec(),
            }],
        }
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(Cpu::default())
}

pub fn component() -> super::Component {
    super::Component {
        spec: SPEC,
        make: model,
    }
}

pub struct Run {
    pub outs: Vec<u8>,
    pub halted: bool,
    /// Instructions executed before reaching HLT.
    pub cycles: usize,
}

/// Run a ROM image on the reference CPU, recording every OUT.
pub fn run_program(rom: &[u8], max: usize) -> Run {
    let mut c = Cpu::default();
    let mut outs = Vec::new();
    for cycles in 0..max {
        let instr = rom.get(c.pc as usize).copied().unwrap_or(0);
        if instr >> 4 == op::HLT {
            return Run {
                outs,
                halted: true,
                cycles,
            };
        }
        if instr >> 4 == op::OUT {
            outs.push(c.acc);
        }
        c.step(instr);
    }
    Run {
        outs,
        halted: false,
        cycles: max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::testkit::clock;

    #[test]
    fn each_instruction() {
        let mut c = Cpu {
            acc: 5,
            ..Cpu::default()
        };
        c.step(0x13); // LDI 3
        assert_eq!((c.acc, c.pc), (3, 1));
        c.step(0x2F); // ADI 15
        assert_eq!(c.acc, 18);
        c.step(0x64); // STA 4
        assert_eq!(c.ram[4], 18);
        c.step(0x34); // ADD 4
        assert_eq!(c.acc, 36);
        c.step(0x44); // SUB 4
        assert_eq!(c.acc, 18);
        c.step(0x10); // LDI 0
        c.step(0x44); // SUB 4: 0 - 18 wraps
        assert_eq!(c.acc, 256u16.wrapping_sub(18) as u8);
        c.step(0x54); // LDA 4
        assert_eq!(c.acc, 18);
        c.step(0x90); // OUT
        assert_eq!(c.out, 18);
        c.step(0x8C); // JZ 12, ACC is not 0: no jump
        assert_eq!(c.pc, 10);
        c.step(0x10); // LDI 0
        c.step(0x8C); // JZ 12, ACC is 0: jump
        assert_eq!(c.pc, 12);
        c.step(0x73); // JMP 3
        assert_eq!(c.pc, 3);
        c.step(0x00); // NOP
        assert_eq!((c.pc, c.acc), (4, 0));
        c.step(0xF0); // HLT: PC stays
        assert_eq!(c.pc, 4);
    }

    #[test]
    fn pc_wraps_after_address_15() {
        let mut c = Cpu {
            pc: 15,
            ..Cpu::default()
        };
        c.step(0x00);
        assert_eq!(c.pc, 0);
    }

    #[test]
    fn executes_on_the_rising_edge_only() {
        let mut c = Cpu::default();
        let ldi7 = from_num(0x17, 8);
        clock(&mut c, &ldi7);
        assert_eq!(c.acc, 7);
        // Holding CLK high with another instruction does nothing.
        let mut held = sigs(&from_num(0x19, 8));
        held.push(Sig::H);
        c.eval(&held);
        assert_eq!(c.acc, 7);
    }

    #[test]
    fn halt_line_follows_the_opcode() {
        let mut c = Cpu::default();
        let mut ins = sigs(&from_num(0xF0, 8));
        ins.push(Sig::L);
        assert_eq!(c.eval(&ins)[12], Sig::H);
    }

    #[test]
    fn internals_show_the_registers_and_ram() {
        let mut c = Cpu::default();
        for i in [0x1Bu8, 0x63, 0x90] {
            clock(&mut c, &from_num(i as u32, 8)); // LDI 11, STA 3, OUT
        }
        let s = c.internals();
        assert_eq!(s.reg("PC"), Some(3));
        assert_eq!(s.reg("ACC"), Some(11));
        assert_eq!(s.reg("OUT"), Some(11));
        assert_eq!(s.mems[0].bytes[3], 11);
    }

    #[test]
    fn run_program_counts_cycles_and_outputs() {
        // LDI 3, OUT, HLT
        let r = run_program(&[0x13, 0x90, 0xF0], 100);
        assert_eq!((r.outs, r.halted, r.cycles), (vec![3], true, 2));
        let r = run_program(&[0x70], 50); // JMP 0 forever
        assert!(!r.halted);
    }
}
