//! NAND. Y = NOT (A AND B). Universal: every other gate can be built from it.

use super::Spec;

pub const SPEC: Spec = Spec {
    id: "nand",
    label: "NAND",
    name: "NAND",
    inputs: &["A", "B"],
    outputs: &["Y"],
    sequential: false,
    transistors: 4,
    summary: "Y = NOT (A AND B)",
};

pub fn f(b: &[bool]) -> Vec<bool> {
    vec![!(b[0] && b[1])]
}

crate::comb_component!(SPEC, f);

#[cfg(test)]
mod tests {
    use crate::components::testkit::*;

    #[test]
    fn truth_table() {
        table(
            "nand",
            &[("00", "1"), ("10", "1"), ("01", "1"), ("11", "0")],
        );
    }

    #[test]
    fn cmos_nand_is_a_nand() {
        // Pull-up: PMOS in parallel. Pull-down: NMOS in series.
        Fab::new(&["A", "B"], &["Y"])
            .p("A", "VCC", "Y")
            .p("B", "VCC", "Y")
            .n("A", "Y", "M")
            .n("B", "M", "GND")
            .proves("nand");
    }

    #[test]
    fn swapped_networks_are_not_a_nand() {
        // NMOS in parallel, PMOS in series: that circuit is a NOR.
        let rows = Fab::new(&["A", "B"], &["Y"])
            .p("A", "VCC", "M")
            .p("B", "M", "Y")
            .n("A", "Y", "GND")
            .n("B", "Y", "GND")
            .outputs(1);
        let ys: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(
            ys,
            ["1", "0", "0", "0"],
            "the swapped circuit is exactly a NOR"
        );
    }

    #[test]
    fn a_zero_input_decides_the_output() {
        assert_eq!(super::model().eval(&parse("0X")), parse("1"));
        assert_eq!(super::model().eval(&parse("1X")), parse("X"));
    }
}
