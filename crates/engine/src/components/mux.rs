//! 2-to-1 multiplexer. Y = A when SEL is 0, B when SEL is 1.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "mux",
    label: "MUX",
    name: "Selector",
    inputs: &["A", "B", "SEL"],
    outputs: &["Y"],
    sequential: false,
    transistors: 14,
    summary: "Y = A when SEL is 0, B when SEL is 1",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![if b[2] { b[1] } else { b[0] }]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        // Inputs in pin order: A, B, SEL.
        table(
            "mux",
            &[
                ("000", "0"),
                ("100", "1"),
                ("010", "0"),
                ("110", "1"),
                ("001", "0"),
                ("101", "0"),
                ("011", "1"),
                ("111", "1"),
            ],
        );
    }

    #[test]
    fn equal_inputs_pass_through_an_unknown_select() {
        assert_eq!(super::model().eval(&parse("11X")), parse("1"));
        assert_eq!(super::model().eval(&parse("10X")), parse("X"));
    }
}
