//! Reference programs, and the program sets diagnostics run.

use crate::rng::Rng;

/// Reference programs: (level id, ROM image). Each is a par solution.
pub const REF_PROGRAMS: [(&str, &[u8]); 4] = [
    // LDI 15, ADI 15, ADI 12, OUT, HLT
    ("prog_answer", &[0x1F, 0x2F, 0x2C, 0x90, 0xF0]),
    // RAM[0] = 1, LDI 9, four unrolled OUT/SUB pairs, then a loop that exits on 0.
    (
        "prog_countdown",
        &[
            0x11, 0x60, 0x19, 0x90, 0x40, 0x90, 0x40, 0x90, 0x40, 0x90, 0x40, 0x90, 0x8F, 0x40,
            0x73, 0xF0,
        ],
    ),
    // Doubling by STA/ADD, unrolled four times; 128 doubled wraps to 0.
    (
        "prog_powers",
        &[
            0x11, 0x90, 0x60, 0x30, 0x90, 0x60, 0x30, 0x90, 0x60, 0x30, 0x90, 0x60, 0x30, 0x8F,
            0x71, 0xF0,
        ],
    ),
    // Build 70 in RAM[1], then count up in sevens until ACC - 70 is 0.
    (
        "prog_table",
        &[
            0x1F, 0x2F, 0x25, 0x61, 0x31, 0x61, 0x17, 0x90, 0x60, 0x41, 0x8E, 0x50, 0x27, 0x77,
            0xF0,
        ],
    ),
];

/// Programs used by the CPU and computer diagnostics: the reference
/// programs plus random ones that exercise every instruction.
pub fn test_programs(id: &str) -> Vec<[u8; 16]> {
    let mut v: Vec<[u8; 16]> = REF_PROGRAMS
        .iter()
        .map(|(_, p)| {
            let mut a = [0u8; 16];
            a[..p.len()].copy_from_slice(p);
            a
        })
        .collect();
    let mut rng = Rng::seeded(id);
    let ops = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 6, 9, 8];
    for _ in 0..12 {
        let mut p = [0u8; 16];
        for b in p.iter_mut() {
            let o = if rng.below(30) == 0 {
                15
            } else {
                ops[rng.below(ops.len() as u64) as usize]
            };
            *b = (o << 4) | rng.below(16) as u8;
        }
        v.push(p);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::level;
    use crate::level::Test;
    use mark_one_engine::components::run_program;

    #[test]
    fn reference_programs_meet_their_goals_at_par() {
        for (id, rom) in REF_PROGRAMS {
            let l = level(id).unwrap();
            let Test::Program(want) = &l.test else {
                panic!("{id} is not a program Mark")
            };
            let run = run_program(rom, 2000);
            assert!(run.halted, "{id} halts");
            assert_eq!(&run.outs, want, "{id} output");
            assert_eq!(run.cycles as u32, l.par, "{id} cycles");
        }
    }

    #[test]
    fn test_programs_include_every_reference_program() {
        let progs = test_programs("cpu");
        for (_, rom) in REF_PROGRAMS {
            assert!(progs.iter().any(|p| p.starts_with(rom)));
        }
    }
}
