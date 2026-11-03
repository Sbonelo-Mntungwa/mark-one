//! 8-bit conditional inverter: Y = X when INV is 0, NOT X when INV is 1.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "flip8",
    label: "FLIP8",
    name: "Byte flipper",
    inputs: &["X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7", "INV"],
    outputs: &["Y0", "Y1", "Y2", "Y3", "Y4", "Y5", "Y6", "Y7"],
    sequential: false,
    transistors: 128,
    summary: "Inverts the 8-bit bus X when INV is 1.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    b[..8].iter().map(|&x| x ^ b[8]).collect()
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn passes_or_inverts_every_value() {
        exhaustive("flip8", |b| {
            let x = super::super::to_num(&b[..8]);
            let y = if b[8] { !x & 255 } else { x };
            super::super::from_num(y, 8)
        });
    }
}
