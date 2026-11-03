//! What the player has finished, what that unlocks, and the save file.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use mark_one_engine::circuit::{Circuit, Part, PartKind};
use mark_one_engine::components;
use mark_one_engine::custom::CustomDef;
use mark_one_engine::sim::Sim;

use crate::catalog::{index_of, level, levels};
use crate::reference::model;

/// Transistor cost charged for a ROM part.
pub const ROM_COST: u32 = 128;

/// What the player has finished, used to decide what is in the parts bin.
pub struct Ctx<'a> {
    pub done: &'a BTreeMap<String, u32>,
    pub unlock_all: bool,
}

impl Ctx<'_> {
    pub fn level_open(&self, idx: usize) -> bool {
        self.unlock_all || idx == 0 || self.done.contains_key(levels()[idx - 1].id)
    }

    pub fn chip_ok(&self, idx: usize, chip: &str) -> bool {
        match index_of(chip) {
            Some(j) => {
                levels()[j].chip && j < idx && (self.unlock_all || self.done.contains_key(chip))
            }
            None => false,
        }
    }

    /// Experiment mode: transistors unlock with the Charge chapter.
    pub fn transistors_unlocked(&self) -> bool {
        self.unlock_all || self.done.contains_key("relay")
    }

    /// Experiment mode: ROM and displays unlock when the CPU passes.
    pub fn rom_unlocked(&self) -> bool {
        self.unlock_all || self.done.contains_key("cpu")
    }

    /// Experiment mode: a library component unlocks when its Mark passes.
    pub fn component_unlocked(&self, id: &str) -> bool {
        self.unlock_all || self.done.contains_key(id)
    }

    /// The Experiment mode rule for one part. Custom components are always
    /// allowed; the parts inside them are checked by this same rule.
    pub fn experiment_allow(&self, p: &Part) -> Result<(), String> {
        let mk = |id: &str| index_of(id).map_or(0, |i| i + 1);
        match p.kind {
            PartKind::Nmos | PartKind::Pmos if !self.transistors_unlocked() => Err(format!(
                "Transistors unlock when you finish the Charge missions (Mk {}).",
                mk("relay")
            )),
            PartKind::Rom | PartKind::Display if !self.rom_unlocked() => Err(format!(
                "ROM and displays unlock when your CPU passes Mk {}.",
                mk("cpu")
            )),
            PartKind::Chip => {
                let id = p.chip.as_deref().unwrap_or("");
                match components::find(id) {
                    Some(k) if !self.component_unlocked(id) => Err(format!(
                        "{} unlocks when you pass Mk {}.",
                        k.spec.label,
                        mk(id)
                    )),
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }

    pub fn chip_cost(&self, chip: &str) -> u32 {
        self.done
            .get(chip)
            .copied()
            .unwrap_or_else(|| level(chip).map_or(0, |l| l.par))
    }

    pub fn circuit_cost(&self, c: &Circuit) -> u32 {
        c.parts
            .iter()
            .map(|p| match p.kind {
                PartKind::Nmos | PartKind::Pmos => 1,
                PartKind::Chip => p.chip.as_deref().map_or(0, |id| self.chip_cost(id)),
                PartKind::Rom => ROM_COST,
                _ => 0,
            })
            .sum()
    }

    pub fn build_sim(&self, idx: usize, c: &Circuit) -> Result<Sim, String> {
        let l = &levels()[idx];
        let allow = |p: &Part| -> Result<(), String> {
            match p.kind {
                PartKind::Nmos | PartKind::Pmos if !l.fets => {
                    Err("Transistors are not unlocked on this level yet.".into())
                }
                PartKind::Rom | PartKind::Display if !l.rom => {
                    Err("ROM and displays unlock when your CPU passes diagnostics.".into())
                }
                PartKind::Chip => {
                    let id = p.chip.as_deref().unwrap_or("");
                    if self.chip_ok(idx, id) {
                        Ok(())
                    } else {
                        Err(format!("Chip '{id}' is not available on this level."))
                    }
                }
                _ => Ok(()),
            }
        };
        Sim::build(c, l.inputs.len(), l.outputs.len(), &model, &allow)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Save {
    /// Level id -> best (lowest) transistor cost.
    #[serde(default)]
    pub done: BTreeMap<String, u32>,
    #[serde(default)]
    pub drafts: BTreeMap<String, Circuit>,
    /// Experiment designs by name.
    #[serde(default)]
    pub designs: BTreeMap<String, Circuit>,
    /// Custom components by id.
    #[serde(default)]
    pub custom: BTreeMap<String, CustomDef>,
    /// Test bench text for each Experiment design, by design name.
    #[serde(default)]
    pub benches: BTreeMap<String, String>,
}

impl Save {
    pub fn load(path: &Path) -> Save {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn store(&self, path: &Path) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_unlock_in_order() {
        let mut done = BTreeMap::new();
        let ctx = Ctx {
            done: &done,
            unlock_all: false,
        };
        assert!(ctx.level_open(0));
        assert!(!ctx.level_open(1));
        done.insert("power".to_string(), 0);
        let ctx = Ctx {
            done: &done,
            unlock_all: false,
        };
        assert!(ctx.level_open(1));
        assert!(!ctx.level_open(2));
    }

    #[test]
    fn chips_need_their_mark_and_come_from_earlier_marks() {
        let mut done = BTreeMap::new();
        let and = index_of("and").unwrap();
        assert!(!Ctx {
            done: &done,
            unlock_all: false
        }
        .chip_ok(and, "nand"));
        done.insert("nand".to_string(), 4);
        let ctx = Ctx {
            done: &done,
            unlock_all: false,
        };
        assert!(ctx.chip_ok(and, "nand"));
        assert!(
            !ctx.chip_ok(index_of("nand").unwrap(), "nand"),
            "a Mark cannot use its own chip"
        );
    }

    #[test]
    fn experiment_parts_unlock_with_their_missions() {
        let mut done = BTreeMap::new();
        let ctx = Ctx {
            done: &done,
            unlock_all: false,
        };
        let nmos = Part::new("t", PartKind::Nmos);
        assert!(ctx.experiment_allow(&nmos).unwrap_err().contains("Mk 2"));
        assert!(ctx
            .experiment_allow(&Part::chip("g", "nand"))
            .unwrap_err()
            .contains("Mk 4"));
        assert!(ctx
            .experiment_allow(&Part::new("r", PartKind::Rom))
            .is_err());
        assert!(ctx.experiment_allow(&Part::new("v", PartKind::Vcc)).is_ok());
        assert!(ctx
            .experiment_allow(&Part::chip("c", "custom:mine"))
            .is_ok());
        done.insert("relay".to_string(), 0);
        done.insert("nand".to_string(), 4);
        let ctx = Ctx {
            done: &done,
            unlock_all: false,
        };
        assert!(ctx.experiment_allow(&nmos).is_ok());
        assert!(ctx.experiment_allow(&Part::chip("g", "nand")).is_ok());
        assert!(ctx.experiment_allow(&Part::chip("g", "xor")).is_err());
    }

    #[test]
    fn chip_cost_is_the_players_best() {
        let mut done = BTreeMap::new();
        let ctx = Ctx {
            done: &done,
            unlock_all: true,
        };
        assert_eq!(ctx.chip_cost("xor"), 16, "unbuilt chips cost par");
        done.insert("xor".to_string(), 12);
        assert_eq!(
            Ctx {
                done: &done,
                unlock_all: true
            }
            .chip_cost("xor"),
            12
        );
    }
}
