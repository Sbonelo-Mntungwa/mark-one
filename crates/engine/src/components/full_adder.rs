//! Full adder: adds A, B and a carry in.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "full_adder",
    label: "FA",
    name: "Full adder",
    inputs: &["A", "B", "CIN"],
    outputs: &["S", "COUT"],
    sequential: false,
    transistors: 50,
    summary: "Adds A + B + CIN.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    let r = b[0] as u32 + b[1] as u32 + b[2] as u32;
    vec![r & 1 == 1, r >= 2]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        // A, B, CIN -> S, COUT
        table(
            "full_adder",
            &[
                ("000", "00"),
                ("100", "10"),
                ("010", "10"),
                ("110", "01"),
                ("001", "10"),
                ("101", "01"),
                ("011", "01"),
                ("111", "11"),
            ],
        );
    }

    #[test]
    #[allow(clippy::nonminimal_bool)] // the textbook majority form, on purpose
    fn sum_is_parity_and_carry_is_majority() {
        exhaustive("full_adder", |b| {
            let (a, x, c) = (b[0], b[1], b[2]);
            vec![a ^ x ^ c, (a && x) || (a && c) || (x && c)]
        });
    }
}
