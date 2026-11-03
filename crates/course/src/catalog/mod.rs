//! The learning path: every Mark, in order, one file per chapter.

use std::sync::OnceLock;

use crate::level::Level;

mod arithmetic;
mod buses;
mod charge;
mod computer;
mod counting;
mod cpu;
mod decoding;
mod gates;
mod machine;
mod memory;
mod memory_banks;
mod software;
mod transistors;
mod workshop;

fn build_levels() -> Vec<Level> {
    let mut v: Vec<Level> = [
        charge::levels(),
        transistors::levels(),
        gates::levels(),
        arithmetic::levels(),
        buses::levels(),
        memory::levels(),
        machine::levels(),
        counting::levels(),
        decoding::levels(),
        memory_banks::levels(),
        cpu::levels(),
        computer::levels(),
        software::levels(),
        workshop::levels(),
    ]
    .into_iter()
    .flatten()
    .collect();
    // Transistors unlock after the first chapter; ROM and display unlock
    // from the computer onwards.
    let rom_from = v
        .iter()
        .position(|l| l.id == "computer")
        .unwrap_or(usize::MAX);
    for (i, l) in v.iter_mut().enumerate() {
        l.fets = l.chapter != charge::CHAPTER;
        l.rom = i >= rom_from;
    }
    v
}

pub fn levels() -> &'static [Level] {
    static L: OnceLock<Vec<Level>> = OnceLock::new();
    L.get_or_init(build_levels)
}

pub fn index_of(id: &str) -> Option<usize> {
    levels().iter().position(|l| l.id == id)
}

pub fn level(id: &str) -> Option<&'static Level> {
    levels().iter().find(|l| l.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_levels_match_their_components() {
        for l in levels().iter().filter(|l| l.chip) {
            let c = mark_one_engine::components::find(l.id)
                .unwrap_or_else(|| panic!("no component for {}", l.id));
            assert_eq!(l.inputs, c.spec.inputs, "{} inputs", l.id);
            assert_eq!(l.outputs, c.spec.outputs, "{} outputs", l.id);
            assert_eq!(l.label, c.spec.label, "{} label", l.id);
            assert_eq!(l.par, c.spec.transistors, "{} transistors", l.id);
        }
    }
}
