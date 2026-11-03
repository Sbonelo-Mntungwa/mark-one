//! What a Mark is: its pins, its par, how diagnostics test it, and its brief.

pub enum Test {
    /// Every input combination (or a large random sample past 12 inputs).
    Comb,
    /// Random set/reset/hold sequence. S and R are never raised together.
    Sr,
    /// Random walk where each step either toggles the control pin or
    /// changes the data pins, never both (that would be a setup-time race).
    Clocked {
        ctrl: usize,
        init: Vec<Vec<bool>>,
        steps: usize,
    },
    /// The CPU level: diagnostics act as program memory, feeding the
    /// instruction at the reference PC and checking every half cycle.
    Cpu,
    /// A whole computer: diagnostics load test programs into every ROM
    /// on the board and compare OUT and HALT each cycle.
    Machine,
    /// Software: run the board's ROM program and compare its OUT sequence.
    Program(Vec<u8>),
    /// Free build. No diagnostics.
    Sandbox,
}

pub struct Level {
    pub id: &'static str,
    pub name: &'static str,
    pub chapter: &'static str,
    pub label: &'static str,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    /// Passing this level produces a reusable chip.
    pub chip: bool,
    /// Raw NMOS/PMOS transistors are in the parts bin.
    pub fets: bool,
    /// ROM and display parts are in the parts bin.
    pub rom: bool,
    /// Transistor count of the reference design.
    pub par: u32,
    pub test: Test,
    pub intro: &'static str,
}

// Helpers for writing the catalog.

pub(crate) fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

pub(crate) fn bus(p: &str, n: usize) -> Vec<String> {
    (0..n).map(|i| format!("{p}{i}")).collect()
}

pub(crate) fn cat(parts: &[Vec<String>]) -> Vec<String> {
    parts.concat()
}

pub(crate) fn zeros_with(n: usize, ones: &[usize]) -> Vec<bool> {
    let mut v = vec![false; n];
    for &i in ones {
        v[i] = true;
    }
    v
}

/// Init sequence for a register: data 0, LOAD 1, one clock pulse.
pub(crate) fn reg_init(n: usize, load: usize, clk: usize) -> Vec<Vec<bool>> {
    vec![
        zeros_with(n, &[load]),
        zeros_with(n, &[load, clk]),
        zeros_with(n, &[load]),
    ]
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn lv(
    id: &'static str,
    name: &'static str,
    chapter: &'static str,
    label: &'static str,
    inputs: Vec<String>,
    outputs: Vec<String>,
    chip: bool,
    par: u32,
    test: Test,
    intro: &'static str,
) -> Level {
    Level {
        id,
        name,
        chapter,
        label,
        inputs,
        outputs,
        chip,
        fets: false,
        rom: false,
        par,
        test,
        intro,
    }
}

/// Pins of every computer and software Mark: CLK in, OUT and HALT out.
pub(crate) fn io_machine() -> (Vec<String>, Vec<String>) {
    (s(&["CLK"]), cat(&[bus("OUT", 8), s(&["HALT"])]))
}
