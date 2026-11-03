//! Everything the client needs to draw the course, as JSON.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use mark_one_engine::components::{from_num, sigs};
use mark_one_engine::custom::CustomDef;
use mark_one_engine::workbench::Library;

use crate::catalog::{index_of, levels};
use crate::level::Test;
use crate::progress::{Ctx, ROM_COST};
use crate::reference::model;

pub fn state_json(ctx: &Ctx, custom: &BTreeMap<String, CustomDef>) -> Value {
    let ls: Vec<Value> = levels()
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let status = if ctx.done.contains_key(l.id) {
                "done"
            } else if ctx.level_open(i) {
                "open"
            } else {
                "locked"
            };
            let mode = match l.test {
                Test::Program(_) => "program",
                Test::Sandbox => "sandbox",
                _ => "circuit",
            };
            let goal = match &l.test {
                Test::Program(w) => Some(w.clone()),
                _ => None,
            };
            let mut palette = vec![
                json!({"kind": "vcc", "label": "VCC", "cost": 0}),
                json!({"kind": "gnd", "label": "GND", "cost": 0}),
            ];
            if l.fets {
                palette.push(json!({"kind": "nmos", "label": "NMOS", "cost": 1}));
                palette.push(json!({"kind": "pmos", "label": "PMOS", "cost": 1}));
            }
            if l.rom {
                palette.push(json!({"kind": "rom", "label": "ROM", "cost": ROM_COST}));
                palette.push(json!({"kind": "display", "label": "DISPLAY", "cost": 0}));
            }
            for c in levels() {
                if ctx.chip_ok(i, c.id) {
                    palette.push(json!({"kind": "chip", "chip": c.id, "label": c.label, "cost": ctx.chip_cost(c.id)}));
                }
            }
            let table = if matches!(l.test, Test::Comb) && l.inputs.len() <= 4 {
                let mut m = model(l.id).expect("model");
                let rows: Vec<Value> = (0..1u32 << l.inputs.len())
                    .map(|k| {
                        let ins = from_num(k, l.inputs.len());
                        let outs = m.eval(&sigs(&ins));
                        json!({"ins": ins, "outs": outs})
                    })
                    .collect();
                Some(rows)
            } else {
                None
            };
            json!({
                "id": l.id, "name": l.name, "chapter": l.chapter, "label": l.label,
                "inputs": l.inputs, "outputs": l.outputs, "chip": l.chip, "fets": l.fets,
                "par": l.par, "intro": l.intro, "status": status,
                "best": ctx.done.get(l.id), "palette": palette, "table": table,
                "sequential": !matches!(l.test, Test::Comb),
                "mode": mode, "rom": l.rom, "goal": goal,
                "unit": if mode == "program" { "cycles" } else { "transistors" },
            })
        })
        .collect();
    let comps: Vec<Value> = mark_one_engine::components::catalog()
        .iter()
        .map(|c| {
            let s = &c.spec;
            json!({
                "id": s.id, "label": s.label, "name": s.name, "inputs": s.inputs, "outputs": s.outputs,
                "sequential": s.sequential, "cost": s.transistors, "summary": s.summary,
                "unlocked": ctx.component_unlocked(s.id), "mark": index_of(s.id).map(|i| i + 1),
            })
        })
        .collect();
    let lib = Library::new(custom, &|_| Ok(()));
    let customs: Vec<Value> = custom
        .values()
        .map(|d| {
            json!({
                "id": d.id, "label": d.label, "name": d.name, "inputs": d.inputs, "outputs": d.outputs,
                "cost": lib.cost(&d.circuit), "custom": true, "unlocked": true,
            })
        })
        .collect();
    let mk = |id: &str| index_of(id).map(|i| i + 1);
    json!({
        "levels": ls, "components": comps, "custom": customs, "unlock_all": ctx.unlock_all,
        "unlocks": {
            "transistors": ctx.transistors_unlocked(), "transistors_mark": mk("relay"),
            "rom": ctx.rom_unlocked(), "rom_mark": mk("cpu"),
        },
    })
}
