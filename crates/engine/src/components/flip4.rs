//! 4-bit conditional inverter: Y = X when INV is 0, NOT X when INV is 1.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "flip4",
    label: "FLIP4",
    name: "Nibble flipper",
    inputs: &["X0", "X1", "X2", "X3", "INV"],
    outputs: &["Y0", "Y1", "Y2", "Y3"],
    sequential: false,
    transistors: 64,
    summary: "Inverts the 4-bit bus X when INV is 1.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    b[..4].iter().map(|&x| x ^ b[4]).collect()
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn passes_or_inverts_every_value() {
        exhaustive("flip4", |b| {
            let x = super::super::to_num(&b[..4]);
            let y = if b[4] { !x & 15 } else { x };
            super::super::from_num(y, 4)
        });
    }
}
