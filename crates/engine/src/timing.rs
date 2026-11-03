//! Delays for timing mode.
//!
//! The unit is one gate delay: the time one CMOS stage (an inverter, NAND or
//! NOR, a few transistors switching) takes to respond. Every transistor's
//! gate-to-channel response is one unit.
//!
//! A library component's delay is the longest path through its reference
//! design (the one the course uses to prove each Mark solvable), counting
//! every sub-component at its own delay. The course tests recompute each
//! value from the reference designs, so the table cannot drift from them
//! (`M1_DELAY_SURVEY=1 cargo test -p mark-one-course --test solutions --
//! --nocapture` prints the values the references give).
//! A chip has one delay for all its outputs, so it is pessimistic for fast
//! outputs (the sum of an adder is ready before its carry).
//!
//! Which inputs a chip's outputs respond to (its timing arcs) follows its
//! behaviour: combinational chips and latches respond to every input;
//! edge-triggered chips respond only to CLK, plus the read path of a RAM and
//! the HALT decode of the CPU.

/// Delay of a library component (or `rom`), in gate delays.
pub fn delay(id: &str) -> Option<u32> {
    Some(match id {
        "not" | "nand" | "nor" => 1,
        "and" | "or" => 2,
        "mux" => 3,
        "xor" => 4,
        "half_adder" => 4,
        "flip4" | "flip8" => 4,
        "full_adder" => 10,
        "add4" => 40,
        "add8" => 80,
        "zero8" => 15,
        "alu8" => 99,
        "sr_latch" => 2,
        "d_latch" => 5,
        "dff" => 11,
        "reg_bit" => 11,
        "reg4" => 11,
        "reg8" => 11,
        "accum" => 26,
        "inc4" => 16,
        "mux4" => 3,
        "mux8" => 3,
        "pc4" => 11,
        "dec2" => 5,
        "dec4" => 10,
        "ram2" => 14,
        "ram4" => 17,
        "ram8" => 20,
        "ram16" => 23,
        "ctrl" => 18,
        "cpu" => 44,
        // Nominal: a ROM is placed, not built, so it has no reference
        // design. About a 4-to-16 decoder plus a wired-OR column.
        "rom" => 12,
        _ => return None,
    })
}

/// Timing arcs of a component with these pin names: for each output, the
/// inputs it responds to.
pub fn arcs(id: &str, inputs: &[String], outputs: &[String]) -> Vec<Vec<usize>> {
    let all: Vec<usize> = (0..inputs.len()).collect();
    let Some(clk) = inputs.iter().position(|n| n == "CLK") else {
        return vec![all; outputs.len()];
    };
    outputs
        .iter()
        .map(|o| {
            let mut a = vec![clk];
            if id.starts_with("ram") {
                // Reads are combinational: Q shows the byte at A.
                a.extend(
                    inputs
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| n.starts_with('A'))
                        .map(|(k, _)| k),
                );
            }
            if id == "cpu" && o == "HALT" {
                a.extend(4..8);
            }
            a
        })
        .collect()
}
