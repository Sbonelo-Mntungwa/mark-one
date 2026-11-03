//! Half adder: adds two bits. S is the sum bit, C the carry.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "half_adder",
    label: "HA",
    name: "Half adder",
    inputs: &["A", "B"],
    outputs: &["S", "C"],
    sequential: false,
    transistors: 22,
    summary: "Adds two bits. S is the sum, C the carry.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![b[0] ^ b[1], b[0] && b[1]]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table(
            "half_adder",
            &[("00", "00"), ("10", "10"), ("01", "10"), ("11", "01")],
        );
    }

    #[test]
    fn outputs_are_the_binary_sum() {
        exhaustive("half_adder", |b| {
            let n = b[0] as u8 + b[1] as u8;
            vec![n & 1 == 1, n & 2 == 2]
        });
    }
}
