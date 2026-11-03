//! Zero detector: Z is 1 when all eight bits are 0.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "zero8",
    label: "ZERO8",
    name: "Zero detector",
    inputs: &["X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7"],
    outputs: &["Z"],
    sequential: false,
    transistors: 44,
    summary: "Z is 1 when all 8 bits are 0.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![b.iter().all(|x| !x)]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn only_zero_is_zero() {
        exhaustive("zero8", |b| vec![super::super::to_num(b) == 0]);
    }

    #[test]
    fn a_known_one_decides_despite_unknowns() {
        assert_eq!(super::model().eval(&parse("XXXXXX01")), parse("0"));
    }

    #[test]
    fn past_six_unknowns_the_model_is_conservative() {
        // Hardware would give 0 here. The model reports X rather than
        // enumerate 2^7 cases: unknown, never wrong.
        assert_eq!(super::model().eval(&parse("XXXXXXX1")), parse("X"));
    }
}
