//! Incrementer: Y = A + 1, wrapping 15 to 0.

use super::{from_num, to_num, Spec};

pub const SPEC: Spec = Spec {
    id: "inc4",
    label: "INC4",
    name: "Incrementer",
    inputs: &["A0", "A1", "A2", "A3"],
    outputs: &["Y0", "Y1", "Y2", "Y3"],
    sequential: false,
    transistors: 88,
    summary: "Y = A + 1, wrapping 15 to 0.",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    from_num((to_num(b) + 1) & 15, 4)
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    /// Adding one flips every bit up to and including the lowest 0.
    fn flip_to_first_zero(b: &[bool]) -> Vec<bool> {
        let mut y = b.to_vec();
        for bit in y.iter_mut() {
            *bit = !*bit;
            if *bit {
                break;
            }
        }
        y
    }

    #[test]
    fn every_value() {
        exhaustive("inc4", flip_to_first_zero);
    }
}
