//! Turning a circuit into nets: wires merge pins into nets, then every part
//! becomes drivers, transistors or chip instances on those nets.

use std::collections::HashMap;

use super::union_find::Uf;
use super::{ChipInst, Fet, Sim, Src};
use crate::circuit::{Circuit, End, Part, PartKind};
use crate::components::{Model, Rom};
use crate::signal::Sig;

impl Sim {
    pub fn build(
        c: &Circuit,
        n_in: usize,
        n_out: usize,
        make: &dyn Fn(&str) -> Option<Box<dyn Model>>,
        allow: &dyn Fn(&Part) -> Result<(), String>,
    ) -> Result<Sim, String> {
        let mut base: HashMap<&str, (usize, usize)> = HashMap::new();
        let mut slots = 0usize;
        let mut models: Vec<Option<Box<dyn Model>>> = Vec::new();
        for p in &c.parts {
            allow(p)?;
            let (cnt, m) = match p.kind {
                PartKind::Vcc | PartKind::Gnd => (1, None),
                PartKind::In => {
                    let k = p.pin.ok_or("input part without a pin index")?;
                    if k >= n_in {
                        return Err(format!("input pin {k} does not exist on this level"));
                    }
                    (1, None)
                }
                PartKind::Out => {
                    let k = p.pin.ok_or("output part without a pin index")?;
                    if k >= n_out {
                        return Err(format!("output pin {k} does not exist on this level"));
                    }
                    (1, None)
                }
                PartKind::Nmos | PartKind::Pmos => (3, None),
                PartKind::Display => (8, None),
                PartKind::Rom => {
                    let m: Box<dyn Model> = Box::new(Rom::new(p.data.as_deref().unwrap_or(&[])));
                    (12, Some(m))
                }
                PartKind::Chip => {
                    let id = p
                        .chip
                        .as_deref()
                        .ok_or_else(|| format!("part {} has no chip type", p.id))?;
                    let m = make(id).ok_or_else(|| format!("unknown chip '{id}'"))?;
                    (m.n_in() + m.n_out(), Some(m))
                }
            };
            if base.insert(p.id.as_str(), (slots, cnt)).is_some() {
                return Err(format!("duplicate part id {}", p.id));
            }
            slots += cnt;
            models.push(m);
        }

        let mut uf = Uf::new(slots);
        let slot = |e: &End| -> Result<usize, String> {
            let (b, cnt) = *base
                .get(e.part.as_str())
                .ok_or_else(|| format!("a wire references missing part {}", e.part))?;
            if e.pin >= cnt {
                return Err(format!("part {} has no pin {}", e.part, e.pin));
            }
            Ok(b + e.pin)
        };
        for w in &c.wires {
            let a = slot(&w.from)?;
            let b = slot(&w.to)?;
            uf.union(a, b);
        }

        let mut net_of = vec![0usize; slots];
        let mut root_net: HashMap<usize, usize> = HashMap::new();
        let mut nn = 0usize;
        for (s, net) in net_of.iter_mut().enumerate() {
            let r = uf.find(s);
            *net = *root_net.entry(r).or_insert_with(|| {
                nn += 1;
                nn - 1
            });
        }

        let mut drivers: Vec<Vec<Src>> = (0..nn).map(|_| Vec::new()).collect();
        let mut fets = Vec::new();
        let mut chips = Vec::new();
        let mut out_nets = vec![None; n_out];
        let mut pin_nets = Vec::new();
        for (p, m) in c.parts.iter().zip(models) {
            let (b, cnt) = base[p.id.as_str()];
            let nets: Vec<usize> = (0..cnt).map(|k| net_of[b + k]).collect();
            match p.kind {
                PartKind::Vcc => drivers[nets[0]].push(Src::Const(Sig::H)),
                PartKind::Gnd => drivers[nets[0]].push(Src::Const(Sig::L)),
                PartKind::In => drivers[nets[0]].push(Src::Input(p.pin.unwrap_or(0))),
                PartKind::Out => {
                    let k = p.pin.unwrap_or(0);
                    if out_nets[k].is_none() {
                        out_nets[k] = Some(nets[0]);
                    }
                }
                PartKind::Nmos | PartKind::Pmos => fets.push(Fet {
                    p: p.kind == PartKind::Pmos,
                    g: nets[0],
                    a: nets[1],
                    b: nets[2],
                }),
                PartKind::Display => {}
                PartKind::Chip | PartKind::Rom => {
                    let m = m.expect("chip model");
                    let (ni, no) = (m.n_in(), m.n_out());
                    let ci = chips.len();
                    for k in 0..no {
                        drivers[nets[ni + k]].push(Src::ChipOut(ci, k));
                    }
                    let kind = if p.kind == PartKind::Rom {
                        "rom"
                    } else {
                        p.chip.as_deref().unwrap_or("")
                    };
                    let delay = m
                        .delay()
                        .or_else(|| crate::timing::delay(kind))
                        .unwrap_or(1);
                    let clk = crate::components::find(kind)
                        .and_then(|k| k.spec.inputs.iter().position(|n| *n == "CLK"));
                    chips.push(ChipInst {
                        id: p.id.clone(),
                        model: m,
                        ins: nets[..ni].to_vec(),
                        out: vec![Sig::X; no],
                        delay,
                        clk,
                    });
                }
            }
            pin_nets.push((p.id.clone(), nets));
        }

        Ok(Sim {
            drivers,
            fets,
            chips,
            out_nets,
            pin_nets,
            inputs: vec![Sig::L; n_in],
            val: vec![Sig::Z; nn],
            floating: vec![true; nn],
            degraded: vec![false; nn],
            undriven: vec![0; nn],
            reverse: false,
            short: false,
            unstable: false,
            timing: None,
        })
    }
}
