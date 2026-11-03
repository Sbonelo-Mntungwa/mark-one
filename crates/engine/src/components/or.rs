//! OR. Y = A OR B.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "or",
    label: "OR",
    name: "OR",
    inputs: &["A", "B"],
    outputs: &["Y"],
    sequential: false,
    transistors: 6,
    summary: "Y = A OR B",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![b[0] || b[1]]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table("or", &[("00", "0"), ("10", "1"), ("01", "1"), ("11", "1")]);
    }
}
