//! 4-bit 2-to-1 multiplexer: Y = A when SEL is 0, B when SEL is 1.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "mux4",
    label: "MUX4",
    name: "Nibble selector",
    inputs: &["A0", "A1", "A2", "A3", "B0", "B1", "B2", "B3", "SEL"],
    outputs: &["Y0", "Y1", "Y2", "Y3"],
    sequential: false,
    transistors: 56,
    summary: "Y = A when SEL is 0, B when SEL is 1 (4 bits).",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    if b[8] {
        b[4..8].to_vec()
    } else {
        b[..4].to_vec()
    }
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn selects_whole_values() {
        exhaustive("mux4", |b| {
            let (a, x) = (
                super::super::to_num(&b[..4]),
                super::super::to_num(&b[4..8]),
            );
            super::super::from_num(if b[8] { x } else { a }, 4)
        });
    }
}
