//! Register bit: on each rising edge of CLK, Q takes D if LOAD is 1,
//! otherwise keeps its value.

use super::clocked::{state, Clocked};
use super::{norm, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "reg_bit",
    label: "REG1",
    name: "Register bit",
    inputs: &["D", "LOAD", "CLK"],
    outputs: &["Q"],
    sequential: true,
    transistors: 60,
    summary: "Stores D on a clock edge when LOAD is 1.",
};

fn next(m: &[Sig], q: &[Sig]) -> Vec<Sig> {
    match m[1].as_bool() {
        Some(true) => vec![norm(m[0])],
        Some(false) => q.to_vec(),
        None => vec![Sig::X],
    }
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(3, 1, vec![Sig::L], next, state))
}

pub fn component() -> super::Component {
    super::Component {
        spec: SPEC,
        make: model,
    }
}

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn loads_only_when_asked() {
        // D LOAD CLK -> Q
        script(
            &mut *super::model(),
            &[
                ("100", "0"),
                ("101", "0"), // edge without LOAD: keep 0
                ("110", "0"),
                ("111", "1"), // edge with LOAD: take 1
                ("000", "1"),
                ("001", "1"), // edge without LOAD: keep 1
            ],
        );
    }
}
