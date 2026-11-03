//! NOR. Y = NOT (A OR B).

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "nor",
    label: "NOR",
    name: "NOR",
    inputs: &["A", "B"],
    outputs: &["Y"],
    sequential: false,
    transistors: 4,
    summary: "Y = NOT (A OR B)",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![!(b[0] || b[1])]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table("nor", &[("00", "1"), ("10", "0"), ("01", "0"), ("11", "0")]);
    }

    #[test]
    fn cmos_nor_is_a_nor() {
        // Pull-up: PMOS in series. Pull-down: NMOS in parallel.
        Fab::new(&["A", "B"], &["Y"])
            .p("A", "VCC", "M")
            .p("B", "M", "Y")
            .n("A", "Y", "GND")
            .n("B", "Y", "GND")
            .proves("nor");
    }

    #[test]
    fn a_one_input_decides_the_output() {
        assert_eq!(super::model().eval(&parse("X1")), parse("0"));
    }
}
