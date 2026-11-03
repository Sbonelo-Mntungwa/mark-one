//! Writing a design out for other tools: Verilog for simulators and FPGA
//! flows, and a KiCad netlist for PCB layout.

pub mod kicad;
pub mod verilog;

use crate::circuit::{Circuit, End, PartKind};
use crate::sim::union_find::Uf;
use crate::workbench::Library;

pub use kicad::kicad_netlist;
pub use verilog::{library_module, verilog};

/// Every pin of every part, grouped into nets.
pub(crate) struct Nets {
    /// First slot and pin count of each part, in circuit order.
    base: Vec<(usize, usize)>,
    net_of: Vec<usize>,
    pub count: usize,
}

/// How many pins a part has.
pub(crate) fn pin_count(
    kind: PartKind,
    chip: Option<&str>,
    lib: &Library,
) -> Result<usize, String> {
    Ok(match kind {
        PartKind::Vcc | PartKind::Gnd | PartKind::In | PartKind::Out => 1,
        PartKind::Nmos | PartKind::Pmos => 3,
        PartKind::Rom => 12,
        PartKind::Display => 8,
        PartKind::Chip => {
            let id = chip.unwrap_or("");
            let (i, o) = lib
                .pins(id)
                .ok_or_else(|| format!("Unknown component '{id}'."))?;
            i.len() + o.len()
        }
    })
}

impl Nets {
    pub fn new(c: &Circuit, lib: &Library) -> Result<Nets, String> {
        let mut base = Vec::new();
        let mut slots = 0;
        for p in &c.parts {
            let n = pin_count(p.kind, p.chip.as_deref(), lib)?;
            base.push((slots, n));
            slots += n;
        }
        let slot = |e: &End| -> Result<usize, String> {
            let i = c
                .parts
                .iter()
                .position(|p| p.id == e.part)
                .ok_or_else(|| format!("A wire references missing part {}.", e.part))?;
            let (b, n) = base[i];
            if e.pin >= n {
                return Err(format!("Part {} has no pin {}.", e.part, e.pin));
            }
            Ok(b + e.pin)
        };
        let mut uf = Uf::new(slots);
        for w in &c.wires {
            uf.union(slot(&w.from)?, slot(&w.to)?);
        }
        let mut net_of = vec![0; slots];
        let mut ids = std::collections::HashMap::new();
        for (s, n) in net_of.iter_mut().enumerate() {
            let r = uf.find(s);
            let next = ids.len();
            *n = *ids.entry(r).or_insert(next);
        }
        Ok(Nets {
            base,
            count: ids.len(),
            net_of,
        })
    }

    /// Net of pin `pin` of the part at index `part`.
    pub fn of(&self, part: usize, pin: usize) -> usize {
        self.net_of[self.base[part].0 + pin]
    }

    /// Nets of every pin of the part at index `part`.
    pub fn pins(&self, part: usize) -> Vec<usize> {
        let (b, n) = self.base[part];
        self.net_of[b..b + n].to_vec()
    }
}
