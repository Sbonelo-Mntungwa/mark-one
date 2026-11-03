//! Inverter. Y = NOT A.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "not",
    label: "NOT",
    name: "Inverter",
    inputs: &["A"],
    outputs: &["Y"],
    sequential: false,
    transistors: 2,
    summary: "Y = NOT A",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![!b[0]]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table("not", &[("0", "1"), ("1", "0")]);
    }

    #[test]
    fn cmos_inverter_is_an_inverter() {
        Fab::new(&["A"], &["Y"])
            .p("A", "VCC", "Y")
            .n("A", "Y", "GND")
            .proves("not");
    }

    #[test]
    fn half_an_inverter_floats() {
        let rows = Fab::new(&["A"], &["Y"]).p("A", "VCC", "Y").outputs(1);
        assert_eq!(rows[1].0, "Z", "with A=1 nothing drives Y");
    }

    #[test]
    fn unknown_input_gives_unknown_output() {
        assert_eq!(super::model().eval(&parse("X")), parse("X"));
    }
}
