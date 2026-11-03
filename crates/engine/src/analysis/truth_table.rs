//! Truth tables: every input combination, each applied to a freshly powered
//! copy of the circuit.

use serde::Serialize;

use crate::circuit::Circuit;
use crate::components::from_num;
use crate::signal::Sig;
use crate::workbench::{self, Library};

/// Truth tables sweep every input combination up to this many inputs.
pub const MAX_TABLE_INPUTS: usize = 12;

#[derive(Serialize, Debug)]
pub struct Row {
    pub ins: Vec<bool>,
    pub outs: Vec<Sig>,
    pub short: bool,
    pub unstable: bool,
    /// Outputs driven only through a wrong-polarity transistor.
    pub degraded: bool,
    /// Evaluating chips in the opposite order gives different outputs: the
    /// circuit depends on a race or a glitch.
    pub race: bool,
}

#[derive(Serialize, Debug)]
pub struct Table {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub rows: Vec<Row>,
    /// The circuit has memory: it contains a stateful chip, or its outputs
    /// depend on the inputs that came before. Each row is measured from
    /// power-up, so it shows the response to that input only.
    pub stateful: bool,
}

/// Truth table with every built-in component available.
pub fn truth_table(c: &Circuit) -> Result<Table, String> {
    truth_table_with(c, &Library::open())
}

/// Truth table with the parts `lib` allows, including custom components.
pub fn truth_table_with(c: &Circuit, lib: &Library) -> Result<Table, String> {
    let ins = workbench::inputs(c);
    if ins.len() > MAX_TABLE_INPUTS {
        return Err(format!(
            "A truth table covers up to {MAX_TABLE_INPUTS} inputs; this circuit has {}.",
            ins.len()
        ));
    }
    let rows: Vec<Row> = (0..1u32 << ins.len())
        .map(|k| {
            let b = from_num(k, ins.len());
            let run = |reverse: bool| -> Result<crate::sim::Sim, String> {
                let mut sim = lib.build(c)?;
                sim.set_reverse_order(reverse);
                sim.set_inputs(&b);
                sim.settle();
                Ok(sim)
            };
            let (sim, back) = (run(false)?, run(true)?);
            Ok(Row {
                outs: sim.outputs(),
                short: sim.short,
                unstable: sim.unstable,
                degraded: sim.degraded_outputs().iter().any(|&d| d),
                race: back.outputs() != sim.outputs(),
                ins: b,
            })
        })
        .collect::<Result<_, String>>()?;
    let stateful = lib.has_memory(c) || depends_on_history(c, lib, &rows)?;
    Ok(Table {
        inputs: ins,
        outputs: workbench::outputs(c),
        rows,
        stateful,
    })
}

/// Sweep every input row on one powered-up circuit, forward and then back.
/// If any row then differs from a freshly powered copy, the outputs depend on
/// history: the circuit stores state, for example through feedback.
fn depends_on_history(c: &Circuit, lib: &Library, rows: &[Row]) -> Result<bool, String> {
    let mut sim = lib.build(c)?;
    let order = (0..rows.len()).chain((0..rows.len()).rev());
    for k in order {
        sim.set_inputs(&rows[k].ins);
        sim.settle();
        if sim.outputs() != rows[k].outs {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Part, PartKind};

    /// A CMOS NAND wired by hand, the way a player would.
    fn cmos_nand() -> Circuit {
        let mut c = Circuit {
            parts: vec![
                Part::pin("a", PartKind::In, 0, Some("A")),
                Part::pin("b", PartKind::In, 1, Some("B")),
                Part::pin("y", PartKind::Out, 0, Some("Y")),
                Part::new("v", PartKind::Vcc),
                Part::new("g", PartKind::Gnd),
                Part::new("p1", PartKind::Pmos),
                Part::new("p2", PartKind::Pmos),
                Part::new("n1", PartKind::Nmos),
                Part::new("n2", PartKind::Nmos),
            ],
            wires: vec![],
        };
        // Transistor pins: G = 0, S = 1, D = 2.
        for (t, i) in [("p1", "a"), ("n1", "a"), ("p2", "b"), ("n2", "b")] {
            c.connect((i, 0), (t, 0));
        }
        c.connect(("v", 0), ("p1", 1));
        c.connect(("v", 0), ("p2", 1));
        c.connect(("p1", 2), ("y", 0));
        c.connect(("p2", 2), ("y", 0));
        c.connect(("y", 0), ("n1", 1));
        c.connect(("n1", 2), ("n2", 1));
        c.connect(("n2", 2), ("g", 0));
        c
    }

    #[test]
    fn table_of_a_hand_built_nand() {
        let t = truth_table(&cmos_nand()).unwrap();
        assert_eq!(t.inputs, ["A", "B"]);
        assert_eq!(t.outputs, ["Y"]);
        let ys: Vec<Sig> = t.rows.iter().map(|r| r.outs[0]).collect();
        assert_eq!(ys, [Sig::H, Sig::H, Sig::H, Sig::L]);
        assert!(t.rows.iter().all(|r| !r.short && !r.unstable));
        assert!(!t.stateful);
    }

    #[test]
    fn a_pass_transistor_and_is_flagged_degraded() {
        let mut c = Circuit {
            parts: vec![
                Part::pin("a", PartKind::In, 0, None),
                Part::pin("b", PartKind::In, 1, None),
                Part::pin("y", PartKind::Out, 0, None),
                Part::new("g", PartKind::Gnd),
                Part::new("n", PartKind::Nmos),
                Part::new("p", PartKind::Pmos),
            ],
            wires: vec![],
        };
        c.connect(("a", 0), ("n", 0));
        c.connect(("b", 0), ("n", 1));
        c.connect(("n", 2), ("y", 0));
        c.connect(("a", 0), ("p", 0));
        c.connect(("y", 0), ("p", 1));
        c.connect(("p", 2), ("g", 0));
        let t = truth_table(&c).unwrap();
        assert!(t.rows.iter().any(|r| r.degraded));
    }

    #[test]
    fn memory_is_reported() {
        let mut c = Circuit::default();
        c.parts.push(Part::chip("d", "dff"));
        assert!(truth_table(&c).unwrap().stateful);
    }

    #[test]
    fn too_many_inputs() {
        let mut c = Circuit::default();
        for k in 0..13 {
            c.parts
                .push(Part::pin(&format!("i{k}"), PartKind::In, k, None));
        }
        assert!(truth_table(&c).is_err());
    }
}
