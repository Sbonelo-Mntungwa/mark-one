//! KiCad netlist export (the s-expression `.net` format, version "E"), for
//! taking a design to PCB layout.
//!
//! - Transistors become Q1, Q2 ... with pins 1 = G, 2 = S, 3 = D (pin 2 is
//!   the top pin on the board: the source of a PMOS, the drain of an NMOS;
//!   the simulator treats them alike).
//! - Chips, custom components and ROMs become U1, U2 ... with pins numbered
//!   from 1 in the component's pin order, inputs first.
//! - Displays become DS1, DS2 ...
//! - VCC and GND are not parts: they name the nets they sit on. INPUT and
//!   OUTPUT pins name their nets too, so the design's pins appear as net
//!   names to connect a header or the rest of a board to.
//!
//! Mark One parts have no footprints. Assign them in KiCad (a library
//! component is a function, not a package: a 4-bit adder might become a
//! 74HC283).

use std::fmt::Write;

use super::{pin_count, Nets};
use crate::circuit::{Circuit, PartKind};
use crate::workbench::{self, Library};

fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The KiCad netlist of design `c`, named `name`, built with `lib`.
pub fn kicad_netlist(c: &Circuit, lib: &Library, name: &str) -> Result<String, String> {
    lib.check(c)?;
    let nets = Nets::new(c, lib)?;
    let (ins, outs) = (workbench::inputs(c), workbench::outputs(c));

    // Reference designators and pin names, per part.
    let mut refs: Vec<Option<(String, String, Vec<String>)>> = Vec::new();
    let (mut nq, mut nu, mut nd) = (0, 0, 0);
    for p in &c.parts {
        refs.push(match p.kind {
            PartKind::Nmos | PartKind::Pmos => {
                nq += 1;
                let top = if p.kind == PartKind::Pmos {
                    ["S", "D"]
                } else {
                    ["D", "S"]
                };
                let value = if p.kind == PartKind::Pmos {
                    "PMOS"
                } else {
                    "NMOS"
                };
                Some((
                    format!("Q{nq}"),
                    value.into(),
                    vec!["G".into(), top[0].into(), top[1].into()],
                ))
            }
            PartKind::Chip => {
                nu += 1;
                let id = p.chip.as_deref().unwrap_or("");
                let (i, o) = lib
                    .pins(id)
                    .ok_or_else(|| format!("Unknown component '{id}'."))?;
                let value = crate::components::find(id)
                    .map(|k| k.spec.label.to_string())
                    .or_else(|| lib.custom(id).map(|d| d.label.clone()))
                    .unwrap_or_else(|| id.to_string());
                Some((format!("U{nu}"), value, i.into_iter().chain(o).collect()))
            }
            PartKind::Rom => {
                nu += 1;
                let pins = (0..4)
                    .map(|k| format!("A{k}"))
                    .chain((0..8).map(|k| format!("D{k}")))
                    .collect();
                Some((format!("U{nu}"), "ROM16x8".into(), pins))
            }
            PartKind::Display => {
                nd += 1;
                Some((
                    format!("DS{nd}"),
                    "DISPLAY8".into(),
                    (0..8).map(|k| format!("B{k}")).collect(),
                ))
            }
            _ => None,
        });
    }

    // Net names: a design pin, then a supply, then a generated name.
    let mut names: Vec<Option<String>> = vec![None; nets.count];
    for (k, p) in c.parts.iter().enumerate() {
        let n = nets.of(k, 0);
        let name = match p.kind {
            PartKind::In => ins.get(p.pin.unwrap_or(0)).cloned(),
            PartKind::Out => outs.get(p.pin.unwrap_or(0)).cloned(),
            _ => None,
        };
        if let Some(name) = name {
            names[n].get_or_insert(name);
        }
    }
    for (k, p) in c.parts.iter().enumerate() {
        let n = nets.of(k, 0);
        match p.kind {
            PartKind::Vcc => {
                names[n].get_or_insert("VCC".into());
            }
            PartKind::Gnd => {
                names[n].get_or_insert("GND".into());
            }
            _ => {}
        }
    }

    let mut s = String::new();
    writeln!(s, "(export (version \"E\")").ok();
    writeln!(
        s,
        "  (design (source {}) (tool \"Mark One\")\n    (comment (number \"1\") (value \"No footprints: assign them in KiCad.\")))",
        q(name)
    )
    .ok();
    s += "  (components";
    for (p, r) in c.parts.iter().zip(&refs) {
        if let Some((rf, value, _)) = r {
            write!(
                s,
                "\n    (comp (ref {}) (value {}) (footprint \"\") (property (name \"MarkOneId\") (value {})))",
                q(rf),
                q(value),
                q(&p.id)
            )
            .ok();
        }
    }
    s += ")\n  (nets";
    let mut members: Vec<Vec<(String, usize, String)>> = vec![Vec::new(); nets.count];
    for (k, (p, r)) in c.parts.iter().zip(&refs).enumerate() {
        if let Some((rf, _, pins)) = r {
            let n = pin_count(p.kind, p.chip.as_deref(), lib)?;
            for pin in 0..n {
                members[nets.of(k, pin)].push((rf.clone(), pin + 1, pins[pin].clone()));
            }
        }
    }
    let mut code = 0;
    for (n, m) in members.iter().enumerate() {
        // A net with no part pin on it (a lone design pin, or nothing) has
        // nothing to route.
        if m.is_empty() {
            continue;
        }
        code += 1;
        let name = names[n].clone().unwrap_or_else(|| format!("Net{n}"));
        write!(
            s,
            "\n    (net (code {}) (name {})",
            q(&code.to_string()),
            q(&name)
        )
        .ok();
        for (rf, pin, func) in m {
            write!(
                s,
                "\n      (node (ref {}) (pin {}) (pinfunction {}))",
                q(rf),
                q(&pin.to_string()),
                q(func)
            )
            .ok();
        }
        s += ")";
    }
    s += "))\n";
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Part;

    #[test]
    fn a_cmos_inverter_lists_its_transistors_and_named_nets() {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("a", PartKind::In, 0, Some("A")));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        c.parts.push(Part::new("v", PartKind::Vcc));
        c.parts.push(Part::new("g", PartKind::Gnd));
        c.parts.push(Part::new("p", PartKind::Pmos));
        c.parts.push(Part::new("n", PartKind::Nmos));
        c.connect(("a", 0), ("p", 0));
        c.connect(("a", 0), ("n", 0));
        c.connect(("v", 0), ("p", 1));
        c.connect(("p", 2), ("y", 0));
        c.connect(("y", 0), ("n", 1));
        c.connect(("n", 2), ("g", 0));
        let s = kicad_netlist(&c, &Library::open(), "inv \"1\"").unwrap();
        assert!(s.starts_with("(export (version \"E\")"));
        assert!(
            s.contains("(source \"inv \\\"1\\\"\")"),
            "quotes are escaped"
        );
        assert!(s.contains("(comp (ref \"Q1\") (value \"PMOS\")"));
        assert!(s.contains("(comp (ref \"Q2\") (value \"NMOS\")"));
        // Nets named by pin and supply, each with the transistor pins on it.
        let net = |name: &str| {
            let i = s
                .find(&format!("(name \"{name}\")"))
                .unwrap_or_else(|| panic!("no net {name}"));
            s[i..].split("(net ").next().unwrap().to_string()
        };
        assert!(net("A").contains("(ref \"Q1\") (pin \"1\") (pinfunction \"G\")"));
        assert!(net("A").contains("(ref \"Q2\") (pin \"1\")"));
        assert!(net("VCC").contains("(ref \"Q1\") (pin \"2\") (pinfunction \"S\")"));
        assert!(
            net("Y").contains("(ref \"Q1\") (pin \"3\")")
                && net("Y").contains("(ref \"Q2\") (pin \"2\")")
        );
        assert!(net("GND").contains("(ref \"Q2\") (pin \"3\") (pinfunction \"S\")"));
        assert_eq!(s.matches('(').count(), s.matches(')').count(), "balanced");
    }

    #[test]
    fn chips_get_numbered_pins_in_pin_order() {
        let mut c = Circuit::default();
        c.parts.push(Part::chip("x", "nand"));
        c.parts.push(Part::pin("y", PartKind::Out, 0, Some("Y")));
        c.connect(("x", 2), ("y", 0));
        let s = kicad_netlist(&c, &Library::open(), "t").unwrap();
        assert!(s.contains("(comp (ref \"U1\") (value \"NAND\")"));
        assert!(
            s.contains("(name \"Y\")\n      (node (ref \"U1\") (pin \"3\") (pinfunction \"Y\"))")
        );
    }
}
