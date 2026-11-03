//! Control unit: turns an opcode and the zero flag into the switch settings
//! of the CPU datapath.

use super::cpu::op;
use super::{to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "ctrl",
    label: "CTRL",
    name: "Control unit",
    inputs: &["OP0", "OP1", "OP2", "OP3", "Z"],
    outputs: &[
        "BIMM", "PASS", "SUB", "ACCW", "RAMW", "JUMP", "OUTW", "HALT",
    ],
    sequential: false,
    transistors: 242,
    summary: "Turns an opcode and the Z flag into control signals.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let o = to_num(&b[..4]) as u8;
    let z = b[4];
    vec![
        o == op::LDI || o == op::ADI,
        o == op::LDI || o == op::LDA,
        o == op::SUB,
        matches!(o, op::LDI | op::ADI | op::ADD | op::SUB | op::LDA),
        o == op::STA,
        o == op::JMP || (o == op::JZ && z),
        o == op::OUT,
        o == op::HLT,
    ]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    /// The instruction table, one row per opcode: the signals it raises.
    /// Columns: BIMM PASS SUB ACCW RAMW JUMP OUTW HALT.
    const TABLE: [(u32, &str); 16] = [
        (0x0, "00000000"), // NOP
        (0x1, "11010000"), // LDI n
        (0x2, "10010000"), // ADI n
        (0x3, "00010000"), // ADD n
        (0x4, "00110000"), // SUB n
        (0x5, "01010000"), // LDA n
        (0x6, "00001000"), // STA n
        (0x7, "00000100"), // JMP n
        (0x8, "00000000"), // JZ n, with Z = 0
        (0x9, "00000010"), // OUT
        (0xA, "00000000"),
        (0xB, "00000000"),
        (0xC, "00000000"),
        (0xD, "00000000"),
        (0xE, "00000000"),
        (0xF, "00000001"), // HLT
    ];

    #[test]
    fn instruction_table() {
        let mut m = super::model();
        for (opc, want) in TABLE {
            for z in [false, true] {
                let mut ins = super::super::from_num(opc, 4);
                ins.push(z);
                let mut want = want.to_string();
                if opc == 0x8 && z {
                    want = "00000100".into();
                }
                let got: String = m
                    .eval(&super::super::sigs(&ins))
                    .into_iter()
                    .map(sig_char)
                    .collect();
                assert_eq!(got, want, "opcode {opc:X}, Z={}", z as u8);
            }
        }
    }
}
