//! XOR. Y = 1 when A and B differ.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "xor",
    label: "XOR",
    name: "XOR",
    inputs: &["A", "B"],
    outputs: &["Y"],
    sequential: false,
    transistors: 16,
    summary: "Y = 1 when A and B differ",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![b[0] ^ b[1]]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table("xor", &[("00", "0"), ("10", "1"), ("01", "1"), ("11", "0")]);
    }
}
