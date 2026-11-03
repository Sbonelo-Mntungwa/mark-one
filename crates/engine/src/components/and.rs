//! AND. Y = A AND B.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "and",
    label: "AND",
    name: "AND",
    inputs: &["A", "B"],
    outputs: &["Y"],
    sequential: false,
    transistors: 6,
    summary: "Y = A AND B",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![b[0] && b[1]]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table("and", &[("00", "0"), ("10", "0"), ("01", "0"), ("11", "1")]);
    }
}
