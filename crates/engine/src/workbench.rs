//! Open circuits: no level. Input and output parts are numbered by their
//! `pin` and named by their `label`. A [`Library`] decides which parts are
//! available and resolves custom components.

use std::collections::BTreeMap;

use crate::circuit::{Circuit, Part, PartKind};
use crate::components::{self, Model};
use crate::custom::{CustomDef, SubCircuit};
use crate::sim::Sim;

fn names(c: &Circuit, kind: PartKind, prefix: &str) -> Vec<String> {
    let n = c
        .parts
        .iter()
        .filter(|p| p.kind == kind)
        .filter_map(|p| p.pin)
        .map(|k| k + 1)
        .max()
        .unwrap_or(0);
    (0..n)
        .map(|k| {
            c.parts
                .iter()
                .find(|p| p.kind == kind && p.pin == Some(k))
                .and_then(|p| p.label.clone())
                .unwrap_or_else(|| format!("{prefix}{k}"))
        })
        .collect()
}

/// Input pin names, in pin order. Unnamed pins are IN0, IN1 ...
pub fn inputs(c: &Circuit) -> Vec<String> {
    names(c, PartKind::In, "IN")
}

/// Output pin names, in pin order. Unnamed pins are OUT0, OUT1 ...
pub fn outputs(c: &Circuit) -> Vec<String> {
    names(c, PartKind::Out, "OUT")
}

/// Which parts may be used, and the custom components that exist.
pub struct Library<'a> {
    customs: &'a BTreeMap<String, CustomDef>,
    allow: &'a dyn Fn(&Part) -> Result<(), String>,
}

static NO_CUSTOMS: BTreeMap<String, CustomDef> = BTreeMap::new();

fn allow_all(_: &Part) -> Result<(), String> {
    Ok(())
}

impl<'a> Library<'a> {
    /// Every built-in component, no custom ones, nothing locked.
    pub fn open() -> Library<'static> {
        Library {
            customs: &NO_CUSTOMS,
            allow: &allow_all,
        }
    }

    /// `allow` decides each part, including those inside custom components.
    pub fn new(
        customs: &'a BTreeMap<String, CustomDef>,
        allow: &'a dyn Fn(&Part) -> Result<(), String>,
    ) -> Library<'a> {
        Library { customs, allow }
    }

    pub fn custom(&self, id: &str) -> Option<&CustomDef> {
        self.customs.get(id)
    }

    /// Every part allowed and known, through every custom component, with no
    /// component containing itself.
    pub fn check(&self, c: &Circuit) -> Result<(), String> {
        self.walk(c, &mut vec![])
    }

    /// As `check`, for a circuit that is about to become component `id`.
    pub fn check_within(&self, c: &Circuit, id: &str) -> Result<(), String> {
        self.walk(c, &mut vec![id.to_string()])
    }

    fn walk(&self, c: &Circuit, stack: &mut Vec<String>) -> Result<(), String> {
        for p in &c.parts {
            (self.allow)(p)?;
            if p.kind != PartKind::Chip {
                continue;
            }
            let id = p.chip.as_deref().unwrap_or("");
            if components::find(id).is_some() {
                continue;
            }
            let def = self
                .customs
                .get(id)
                .ok_or_else(|| format!("Unknown component '{id}'."))?;
            if stack.iter().any(|s| s == id) {
                return Err(format!("Component '{}' would contain itself.", def.name));
            }
            stack.push(id.to_string());
            self.walk(&def.circuit, stack)?;
            stack.pop();
        }
        Ok(())
    }

    /// Whether the circuit contains a stateful library chip, directly or
    /// inside a custom component.
    pub fn has_memory(&self, c: &Circuit) -> bool {
        c.parts
            .iter()
            .filter(|p| p.kind == PartKind::Chip)
            .any(|p| {
                let id = p.chip.as_deref().unwrap_or("");
                match components::find(id) {
                    Some(k) => k.spec.sequential,
                    None => self
                        .customs
                        .get(id)
                        .is_some_and(|d| self.has_memory(&d.circuit)),
                }
            })
    }

    /// A fresh model of a built-in or custom component.
    pub fn model(&self, id: &str) -> Option<Box<dyn Model>> {
        components::model(id).or_else(|| {
            let def = self.customs.get(id)?;
            SubCircuit::new(def, self)
                .ok()
                .map(|m| Box::new(m) as Box<dyn Model>)
        })
    }

    /// Pin names of a built-in or custom component.
    pub fn pins(&self, id: &str) -> Option<(Vec<String>, Vec<String>)> {
        match components::find(id) {
            Some(k) => Some((
                k.spec.inputs.iter().map(|s| s.to_string()).collect(),
                k.spec.outputs.iter().map(|s| s.to_string()).collect(),
            )),
            None => self
                .customs
                .get(id)
                .map(|d| (d.inputs.clone(), d.outputs.clone())),
        }
    }

    /// Simulator for an open circuit.
    pub fn build(&self, c: &Circuit) -> Result<Sim, String> {
        self.check(c)?;
        Sim::build(
            c,
            inputs(c).len(),
            outputs(c).len(),
            &|id| self.model(id),
            &|_| Ok(()),
        )
    }

    /// Transistor count: built-in chips at their reference cost, custom
    /// components at the cost of what is inside them.
    pub fn cost(&self, c: &Circuit) -> u32 {
        c.parts
            .iter()
            .map(|p| match p.kind {
                PartKind::Nmos | PartKind::Pmos => 1,
                PartKind::Chip => {
                    let id = p.chip.as_deref().unwrap_or("");
                    match components::find(id) {
                        Some(k) => k.spec.transistors,
                        None => self.customs.get(id).map_or(0, |d| self.cost(&d.circuit)),
                    }
                }
                _ => 0,
            })
            .sum()
    }
}

/// Simulator for an open circuit with every built-in component allowed.
pub fn build(c: &Circuit) -> Result<Sim, String> {
    Library::open().build(c)
}

/// Transistor count with every chip at its reference cost.
pub fn cost(c: &Circuit) -> u32 {
    Library::open().cost(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Part;

    #[test]
    fn every_component_is_available() {
        for k in components::catalog() {
            let mut c = Circuit::default();
            c.parts.push(Part::chip("x", k.spec.id));
            assert!(build(&c).is_ok(), "{}", k.spec.id);
        }
    }

    #[test]
    fn unnamed_pins_get_default_names() {
        let mut c = Circuit::default();
        c.parts.push(Part::pin("i", PartKind::In, 1, None));
        c.parts.push(Part::pin("o", PartKind::Out, 0, Some("Y")));
        assert_eq!(inputs(&c), ["IN0", "IN1"]);
        assert_eq!(outputs(&c), ["Y"]);
    }

    #[test]
    fn cost_counts_transistors_and_reference_chips() {
        let mut c = Circuit::default();
        c.parts.push(Part::new("n", PartKind::Nmos));
        c.parts.push(Part::chip("g", "nand"));
        assert_eq!(cost(&c), 1 + 4);
    }
}
