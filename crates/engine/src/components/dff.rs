//! D flip-flop: Q takes D at the rising edge of CLK and holds otherwise.

use super::clocked::{state, Clocked};
use super::{norm, Model, Spec};
use crate::signal::Sig;

pub const SPEC: Spec = Spec {
    id: "dff",
    label: "DFF",
    name: "D flip-flop",
    inputs: &["D", "CLK"],
    outputs: &["Q"],
    sequential: true,
    transistors: 46,
    summary: "Stores D on the rising edge of CLK.",
};

fn next(m: &[Sig], _q: &[Sig]) -> Vec<Sig> {
    vec![norm(m[0])]
}

pub fn model() -> Box<dyn Model> {
    Box::new(Clocked::new(2, 1, vec![Sig::L], next, state))
}

pub fn component() -> super::Component {
    super::Component {
        spec: SPEC,
        make: model,
    }
}

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn captures_only_on_the_rising_edge() {
        // D CLK -> Q
        script(
            &mut *super::model(),
            &[
                ("10", "0"), // D=1, clock low: nothing yet
                ("11", "1"), // rising edge: capture
                ("01", "1"), // D changes while CLK high: ignored
                ("00", "1"), // falling edge: nothing
                ("01", "0"), // rising edge: capture 0
                ("11", "0"), // ignored
            ],
        );
    }

    #[test]
    fn holds_with_the_clock_low() {
        script(
            &mut *super::model(),
            &[("10", "0"), ("00", "0"), ("10", "0")],
        );
    }
}
