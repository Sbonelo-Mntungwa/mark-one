//! Byte-wide static RAM in four sizes: 2, 4, 8 and 16 bytes.
//! Pins: [A.., D0..D7, WE, CLK] -> [Q0..Q7]. Reads are combinational: Q always
//! shows the byte at A. On a rising edge of CLK with WE at 1, D is written to A.
//! Every byte powers up as 0.

use super::{bools, from_num, sigs, to_num, Internals, Mem, Model, Spec};
use crate::signal::Sig;

const Q: &[&str] = &["Q0", "Q1", "Q2", "Q3", "Q4", "Q5", "Q6", "Q7"];

pub const RAM2: Spec = Spec {
    id: "ram2",
    label: "RAM2",
    name: "Memory pair",
    inputs: &[
        "A0", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "WE", "CLK",
    ],
    outputs: Q,
    sequential: true,
    transistors: 1086,
    summary: "2 bytes of memory. Q shows the byte at A; writes D on a clock edge when WE.",
};
pub const RAM4: Spec = Spec {
    id: "ram4",
    label: "RAM4",
    name: "Memory quad",
    inputs: &[
        "A0", "A1", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "WE", "CLK",
    ],
    outputs: Q,
    sequential: true,
    transistors: 2298,
    summary: "4 bytes of memory.",
};
pub const RAM8: Spec = Spec {
    id: "ram8",
    label: "RAM8",
    name: "Memory bank",
    inputs: &[
        "A0", "A1", "A2", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "WE", "CLK",
    ],
    outputs: Q,
    sequential: true,
    transistors: 4722,
    summary: "8 bytes of memory.",
};
pub const RAM16: Spec = Spec {
    id: "ram16",
    label: "RAM16",
    name: "Main memory",
    inputs: &[
        "A0", "A1", "A2", "A3", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "WE", "CLK",
    ],
    outputs: Q,
    sequential: true,
    transistors: 9570,
    summary: "16 bytes of memory.",
};

pub struct Ram {
    abits: usize,
    mem: Vec<u8>,
    master: Option<Vec<Sig>>,
    prev_clk: bool,
}

impl Ram {
    pub fn new(abits: usize) -> Ram {
        Ram {
            abits,
            mem: vec![0; 1 << abits],
            master: None,
            prev_clk: false,
        }
    }
}

impl Model for Ram {
    fn n_in(&self) -> usize {
        self.abits + 10
    }
    fn n_out(&self) -> usize {
        8
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        let n = ins.len();
        let a = self.abits;
        if let Some(clk) = ins[n - 1].as_bool() {
            if !clk {
                self.master = Some(ins[..n - 1].to_vec());
            } else if !self.prev_clk {
                if let Some(m) = &self.master {
                    if let (Some(addr), Some(d), Some(true)) =
                        (bools(&m[..a]), bools(&m[a..a + 8]), m[a + 8].as_bool())
                    {
                        self.mem[to_num(&addr) as usize] = to_num(&d) as u8;
                    }
                }
            }
            self.prev_clk = clk;
        }
        match bools(&ins[..a]) {
            Some(addr) => sigs(&from_num(self.mem[to_num(&addr) as usize] as u32, 8)),
            None => vec![Sig::X; 8],
        }
    }
    fn internals(&self) -> Internals {
        Internals {
            regs: vec![],
            mems: vec![Mem {
                name: "MEM".into(),
                bytes: self.mem.clone(),
            }],
        }
    }
}

fn ram2() -> Box<dyn Model> {
    Box::new(Ram::new(1))
}
fn ram4() -> Box<dyn Model> {
    Box::new(Ram::new(2))
}
fn ram8() -> Box<dyn Model> {
    Box::new(Ram::new(3))
}
fn ram16() -> Box<dyn Model> {
    Box::new(Ram::new(4))
}

/// The RAM with `abits` address bits (1 to 4).
pub fn component(abits: usize) -> super::Component {
    match abits {
        1 => super::Component {
            spec: RAM2,
            make: ram2,
        },
        2 => super::Component {
            spec: RAM4,
            make: ram4,
        },
        3 => super::Component {
            spec: RAM8,
            make: ram8,
        },
        _ => super::Component {
            spec: RAM16,
            make: ram16,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::testkit::*;

    fn write(m: &mut Ram, a: u32, v: u32, we: bool) -> u32 {
        let mut ins = from_num(a, m.abits);
        ins.extend(from_num(v, 8));
        ins.push(we);
        to_num(&bools(&clock(m, &ins)).unwrap())
    }

    fn read(m: &mut Ram, a: u32) -> u32 {
        let mut ins = from_num(a, m.abits);
        ins.extend([false; 9]);
        to_num(&bools(&m.eval(&sigs(&ins))).unwrap())
    }

    #[test]
    fn every_size_stores_every_address_independently() {
        for abits in 1..=4 {
            let mut m = Ram::new(abits);
            let n = 1u32 << abits;
            for a in 0..n {
                assert_eq!(read(&mut m, a), 0, "powers up as 0");
                write(&mut m, a, 37 * a + 11, true);
            }
            for a in 0..n {
                assert_eq!(read(&mut m, a), (37 * a + 11) & 0xff, "RAM{n} address {a}");
            }
            // Without WE nothing is written.
            write(&mut m, 0, 0xEE, false);
            assert_eq!(read(&mut m, 0), 11);
        }
    }

    #[test]
    fn writes_happen_only_on_the_rising_edge() {
        let mut m = Ram::new(1);
        // Address 1, data 0xFF, WE, with CLK high from the start: no edge.
        let mut ins = from_num(1, 1);
        ins.extend(from_num(0xFF, 8));
        ins.extend([true, true]);
        m.eval(&sigs(&ins));
        assert_eq!(read(&mut m, 1), 0);
    }
}
