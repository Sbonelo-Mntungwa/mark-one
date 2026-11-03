//! Custom components: a circuit saved under a name and reused as a chip.
//!
//! A custom component's pins are its design's INPUT and OUTPUT parts, in pin
//! order. Like every chip it is directional: inputs are read, outputs are
//! driven. Inside, it is simulated in full by its own simulator, so it may
//! hold state, use transistors, or contain other custom components (but never
//! itself).

use serde::{Deserialize, Serialize};

use crate::circuit::Circuit;
use crate::components::{Internals, Model};
use crate::signal::Sig;
use crate::sim::Sim;
use crate::workbench::{self, Library};

/// Ids of custom components start with this, so they never collide with the
/// built-in library.
pub const PREFIX: &str = "custom:";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomDef {
    pub id: String,
    pub name: String,
    /// Short name printed on the chip.
    pub label: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub circuit: Circuit,
}

impl CustomDef {
    /// Turn a design into a component, checking that it can be built with
    /// `lib` and does not contain itself.
    pub fn from_design(
        id: &str,
        name: &str,
        label: &str,
        circuit: Circuit,
        lib: &Library,
    ) -> Result<CustomDef, String> {
        if !id.starts_with(PREFIX) {
            return Err(format!("custom component ids start with {PREFIX}"));
        }
        let (inputs, outputs) = (workbench::inputs(&circuit), workbench::outputs(&circuit));
        if outputs.is_empty() {
            return Err("A component needs at least one OUTPUT pin.".into());
        }
        let label = label.trim();
        if label.is_empty() || label.chars().count() > 8 {
            return Err("A chip label is 1 to 8 characters.".into());
        }
        lib.check_within(&circuit, id)?;
        let def = CustomDef {
            id: id.into(),
            name: name.trim().into(),
            label: label.into(),
            inputs,
            outputs,
            circuit,
        };
        lib.build(&def.circuit)?;
        Ok(def)
    }
}

/// A custom component's behaviour: its circuit, simulated.
pub struct SubCircuit {
    sim: Sim,
    n_in: usize,
    n_out: usize,
    /// Inputs at the last evaluation; the circuit only settles again when
    /// they change, so stored charge inside ages per input change.
    last: Option<Vec<Sig>>,
    out: Vec<Sig>,
    /// Critical path through the circuit, its delay as one chip.
    delay: u32,
}

impl SubCircuit {
    pub fn new(def: &CustomDef, lib: &Library) -> Result<SubCircuit, String> {
        let sim = lib.build(&def.circuit)?;
        Ok(SubCircuit {
            sim,
            n_in: def.inputs.len(),
            n_out: def.outputs.len(),
            last: None,
            out: vec![Sig::X; def.outputs.len()],
            delay: crate::analysis::timing::component_delay(&def.id, lib).unwrap_or(1),
        })
    }
}

impl Model for SubCircuit {
    fn n_in(&self) -> usize {
        self.n_in
    }
    fn n_out(&self) -> usize {
        self.n_out
    }
    fn eval(&mut self, ins: &[Sig]) -> Vec<Sig> {
        if self.last.as_deref() != Some(ins) {
            self.sim.set_input_sigs(ins);
            self.sim.settle();
            // A short or oscillation inside makes every output unknown.
            self.out = if self.sim.short || self.sim.unstable {
                vec![Sig::X; self.n_out]
            } else {
                self.sim.outputs()
            };
            self.last = Some(ins.to_vec());
        }
        self.out.clone()
    }
    fn delay(&self) -> Option<u32> {
        Some(self.delay)
    }
    fn internals(&self) -> Internals {
        let mut all = Internals::default();
        for (id, s) in self.sim.internals() {
            all.regs.extend(s.regs.into_iter().map(|mut r| {
                r.name = format!("{id}/{}", r.name);
                r
            }));
            all.mems.extend(s.mems.into_iter().map(|mut m| {
                m.name = format!("{id}/{}", m.name);
                m
            }));
        }
        all
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::circuit::{Part, PartKind};
    use crate::components::{sigs, testkit::parse};

    /// A CMOS inverter design with pins A -> Y.
    fn inverter_design() -> Circuit {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        c.parts.push(Part::new("p", PartKind::Pmos));
        c.parts.push(Part::new("n", PartKind::Nmos));
        for t in ["p", "n"] {
            c.connect(("a", 0), (t, 0));
        }
        c.connect(("v", 0), ("p", 1));
        c.connect(("p", 2), ("y", 0));
        c.connect(("y", 0), ("n", 1));
        c.connect(("n", 2), ("g", 0));
        c
    }

    /// A design wrapping one chip `chip` with one input and one output.
    fn wrap(chip: &str, n: usize) -> Circuit {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        let mut prev = ("a".to_string(), 0);
        for k in 0..n {
            let id = format!("c{k}");
            c.parts.push(Part::chip(&id, chip));
            c.connect((&prev.0, prev.1), (&id, 0));
            prev = (id, 1);
        }
        c.connect((&prev.0, prev.1), ("y", 0));
        c
    }

    fn save(customs: &mut BTreeMap<String, CustomDef>, id: &str, c: Circuit) -> Result<(), String> {
        let def = {
            let lib = Library::new(customs, &|_| Ok(()));
            CustomDef::from_design(id, id, "X", c, &lib)?
        };
        customs.insert(id.into(), def);
        Ok(())
    }

    #[test]
    fn a_saved_inverter_works_as_a_chip() {
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:inv", inverter_design()).unwrap();
        let lib = Library::new(&customs, &|_| Ok(()));
        let mut m = lib.model("custom:inv").unwrap();
        assert_eq!(m.eval(&sigs(&[false])), parse("1"));
        assert_eq!(m.eval(&sigs(&[true])), parse("0"));
        assert_eq!(lib.cost(&wrap("custom:inv", 1)), 2);
        assert_eq!(
            lib.pins("custom:inv").unwrap(),
            (vec!["A".to_string()], vec!["Y".to_string()])
        );
    }

    #[test]
    fn components_nest_and_costs_add_up() {
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:inv", inverter_design()).unwrap();
        save(&mut customs, "custom:buf", wrap("custom:inv", 2)).unwrap();
        let lib = Library::new(&customs, &|_| Ok(()));
        let mut sim = lib.build(&wrap("custom:buf", 1)).unwrap();
        for v in [false, true] {
            sim.set_inputs(&[v]);
            sim.settle();
            assert_eq!(sim.outputs(), sigs(&[v]));
        }
        assert_eq!(lib.cost(&wrap("custom:buf", 1)), 4);
    }

    #[test]
    fn a_component_cannot_contain_itself() {
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:inv", inverter_design()).unwrap();
        // Directly.
        let err = save(&mut customs, "custom:inv", wrap("custom:inv", 1)).unwrap_err();
        assert!(err.contains("contain itself"), "{err}");
        // Through another component.
        save(&mut customs, "custom:outer", wrap("custom:inv", 1)).unwrap();
        let err = save(&mut customs, "custom:inv", wrap("custom:outer", 1)).unwrap_err();
        assert!(err.contains("contain itself"), "{err}");
    }

    #[test]
    fn a_custom_latch_keeps_its_state() {
        // SR latch from two NOR chips: S, R -> Q.
        let mut c = Circuit::default();
        c.parts.push(Part::pin("s", PartKind::In, 0, Some("S")));
        c.parts.push(Part::pin("r", PartKind::In, 1, Some("R")));
        c.parts.push(Part::pin("q", PartKind::Out, 0, Some("Q")));
        c.parts.push(Part::chip("n1", "nor"));
        c.parts.push(Part::chip("n2", "nor"));
        c.connect(("r", 0), ("n1", 0));
        c.connect(("n2", 2), ("n1", 1));
        c.connect(("s", 0), ("n2", 0));
        c.connect(("n1", 2), ("n2", 1));
        c.connect(("n1", 2), ("q", 0));
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:sr", c).unwrap();
        let lib = Library::new(&customs, &|_| Ok(()));
        let mut m = lib.model("custom:sr").unwrap();
        assert_eq!(m.eval(&sigs(&[true, false])), parse("1"), "set");
        assert_eq!(m.eval(&sigs(&[false, false])), parse("1"), "hold");
        assert_eq!(m.eval(&sigs(&[false, true])), parse("0"), "reset");
        assert_eq!(m.eval(&sigs(&[false, false])), parse("0"), "hold");
        assert!(
            m.internals().is_empty(),
            "NOR chips store nothing themselves"
        );
        // Used in a parent design, the truth table notices the memory.
        let mut parent = Circuit::default();
        parent
            .parts
            .push(Part::pin("s", PartKind::In, 0, Some("S")));
        parent
            .parts
            .push(Part::pin("r", PartKind::In, 1, Some("R")));
        parent
            .parts
            .push(Part::pin("q", PartKind::Out, 0, Some("Q")));
        parent.parts.push(Part::chip("x", "custom:sr"));
        parent.connect(("s", 0), ("x", 0));
        parent.connect(("r", 0), ("x", 1));
        parent.connect(("x", 2), ("q", 0));
        assert!(
            crate::analysis::truth_table_with(&parent, &lib)
                .unwrap()
                .stateful
        );
    }

    #[test]
    fn internals_of_nested_chips_are_named_by_path() {
        let mut c = wrap("dff", 0);
        c.parts.push(Part::chip("acc", "accum"));
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:a", c).unwrap();
        let lib = Library::new(&customs, &|_| Ok(()));
        let m = lib.model("custom:a").unwrap();
        let s = m.internals();
        assert_eq!(s.regs.len(), 1);
        assert_eq!(s.regs[0].name, "acc/ACC");
        assert_eq!(s.reg("acc/ACC"), Some(0));
    }

    #[test]
    fn locked_parts_stay_locked_inside_a_component() {
        let mut customs = BTreeMap::new();
        save(&mut customs, "custom:inv", inverter_design()).unwrap();
        let no_fets = |p: &Part| match p.kind {
            PartKind::Nmos | PartKind::Pmos => Err("Transistors are locked.".to_string()),
            _ => Ok(()),
        };
        let lib = Library::new(&customs, &no_fets);
        let err = lib
            .build(&wrap("custom:inv", 1))
            .err()
            .expect("locked parts must not build");
        assert!(err.contains("locked"), "{err}");
    }

    #[test]
    fn a_component_needs_an_output_and_a_label() {
        let lib = Library::open();
        let mut c = inverter_design();
        c.parts.retain(|p| p.kind != PartKind::Out);
        assert!(CustomDef::from_design("custom:x", "x", "X", c, &lib).is_err());
        assert!(CustomDef::from_design("custom:x", "x", "", inverter_design(), &lib).is_err());
        assert!(
            CustomDef::from_design("nand", "x", "X", inverter_design(), &lib).is_err(),
            "ids are prefixed"
        );
    }
}
