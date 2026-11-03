//! 8-bit 2-to-1 multiplexer: Y = A when SEL is 0, B when SEL is 1.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "mux8",
    label: "MUX8",
    name: "Byte selector",
    inputs: &[
        "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "B0", "B1", "B2", "B3", "B4", "B5", "B6",
        "B7", "SEL",
    ],
    outputs: &["Y0", "Y1", "Y2", "Y3", "Y4", "Y5", "Y6", "Y7"],
    sequential: false,
    transistors: 112,
    summary: "Y = A when SEL is 0, B when SEL is 1 (8 bits).",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    if b[16] {
        b[8..16].to_vec()
    } else {
        b[..8].to_vec()
    }
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn selects_whole_values() {
        exhaustive("mux8", |b| {
            let (a, x) = (
                super::super::to_num(&b[..8]),
                super::super::to_num(&b[8..16]),
            );
            super::super::from_num(if b[16] { x } else { a }, 8)
        });
    }
}
