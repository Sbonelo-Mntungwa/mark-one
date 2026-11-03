//! POST /api/sim: simulate a board. Each board keeps a live simulator, rebuilt
//! when its wiring changes, so stored state survives input changes.
//!
//! Besides pin values it can return:
//! - `probes`: the value of chosen pins after every settle, for the
//!   waveform view (`trace`), with event times in timing mode;
//! - `internals`: registers and memories inside every chip, for the
//!   program debugger;
//! - breakpoints: a multi-cycle run stops before the ROM fetches one of
//!   the listed addresses.

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use mark_one_engine::circuit::{Circuit, PartKind};
use mark_one_engine::signal::Sig;
use mark_one_engine::sim::timed::{TimedRun, Violation};
use mark_one_engine::sim::Sim;
use mark_one_engine::workbench::{self, Library};

use crate::app::{level_idx, ApiErr, St};

/// Experiment mode uses this key in place of a level id.
const FREE: &str = "free";
/// Most clock cycles one request may run.
const MAX_CYCLES: u32 = 5000;
/// Most trace samples one request returns.
const MAX_SAMPLES: usize = 20_000;

#[derive(Deserialize)]
pub struct Probe {
    part: String,
    pin: usize,
}

#[derive(Deserialize)]
pub struct SimReq {
    level: String,
    circuit: Circuit,
    #[serde(default)]
    inputs: Vec<bool>,
    #[serde(default)]
    reset: bool,
    /// Run this many full clock cycles (stops early on HALT or a breakpoint).
    #[serde(default)]
    cycles: Option<u32>,
    /// Pins to sample after every settle.
    #[serde(default)]
    probes: Vec<Probe>,
    /// Simulate with gate delays.
    #[serde(default)]
    timed: bool,
    /// ROM addresses to stop at during a multi-cycle run.
    #[serde(default)]
    breakpoints: Vec<u8>,
    /// Return chip internals.
    #[serde(default)]
    inspect: bool,
}

/// One settle, as the waveform draws it.
#[derive(Serialize)]
struct Sample {
    /// Probe values after the settle.
    v: Vec<Sig>,
    /// Inputs applied, in pin order.
    ins: Vec<bool>,
    /// Timing mode: when the step started and ended, and every probe
    /// change in between as [time, probe, value].
    #[serde(skip_serializing_if = "Option::is_none")]
    t0: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    t1: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ev: Vec<(u64, usize, Sig)>,
}

struct Run<'a> {
    sim: &'a mut Sim,
    nets: Vec<Option<usize>>,
    timed: bool,
    samples: Vec<Sample>,
    violations: Vec<Violation>,
    /// Probes that changed more than once within one step.
    glitchy: Vec<bool>,
    max_settle: u64,
}

impl Run<'_> {
    fn settle(&mut self, ins: &[bool]) {
        self.sim.set_inputs(ins);
        let mut s = Sample {
            v: vec![],
            ins: ins.to_vec(),
            t0: None,
            t1: None,
            ev: vec![],
        };
        if self.timed {
            let r: TimedRun = self.sim.settle_timed();
            for c in &r.changes {
                for (k, n) in self.nets.iter().enumerate() {
                    if *n == Some(c.net) {
                        s.ev.push((c.t, k, c.v));
                    }
                }
            }
            for (net, _) in r.glitches() {
                for (k, n) in self.nets.iter().enumerate() {
                    if *n == Some(net) {
                        self.glitchy[k] = true;
                    }
                }
            }
            self.max_settle = self.max_settle.max(r.end - r.start);
            self.violations.extend(r.violations);
            s.t0 = Some(r.start);
            s.t1 = Some(r.end);
        } else {
            self.sim.settle();
        }
        s.v = self
            .nets
            .iter()
            .map(|n| n.map_or(Sig::Z, |n| self.sim.net_value(n)))
            .collect();
        if self.samples.len() < MAX_SAMPLES {
            self.samples.push(s);
        }
    }
}

/// The address the first ROM on the board is reading, if every bit is known.
fn rom_addr(sim: &Sim, c: &Circuit) -> Option<u8> {
    let rom = c.parts.iter().find(|p| p.kind == PartKind::Rom)?;
    (0..4).try_fold(0u8, |a, k| match sim.pin_value(&rom.id, k)? {
        Sig::H => Some(a | 1 << k),
        Sig::L => Some(a),
        _ => None,
    })
}

pub async fn simulate(State(a): St, Json(r): Json<SimReq>) -> Result<Json<Value>, ApiErr> {
    let free = r.level == FREE;
    let idx = if free { 0 } else { level_idx(&r.level)? };
    let key = r.circuit.topology_key();
    let save = a.save.lock().unwrap();
    let ctx = a.ctx(&save);
    let allow = |p: &_| ctx.experiment_allow(p);
    let lib = Library::new(&save.custom, &allow);
    let mut sims = a.sims.lock().unwrap();
    let stale = r.reset || sims.get(&r.level).is_none_or(|(k, _)| *k != key);
    if stale {
        let built = if free {
            lib.build(&r.circuit)
        } else {
            ctx.build_sim(idx, &r.circuit)
        };
        match built {
            Ok(s) => {
                sims.insert(r.level.clone(), (key, s));
            }
            Err(e) => {
                sims.remove(&r.level);
                return Ok(Json(json!({ "error": e })));
            }
        }
    }
    let (_, s) = sims.get_mut(&r.level).expect("sim present");
    let (ins_names, outs_names) = if free {
        (
            workbench::inputs(&r.circuit),
            workbench::outputs(&r.circuit),
        )
    } else {
        let l = &mark_one_course::levels()[idx];
        (l.inputs.clone(), l.outputs.clone())
    };
    let nets = r.probes.iter().map(|p| s.net_of(&p.part, p.pin)).collect();
    let mut run = Run {
        sim: s,
        nets,
        timed: r.timed,
        samples: vec![],
        violations: vec![],
        glitchy: vec![false; r.probes.len()],
        max_settle: 0,
    };
    let mut ins = r.inputs.clone();
    ins.resize(ins_names.len(), false);
    run.settle(&ins);
    let clk = ins_names.iter().position(|n| n == "CLK");
    let halt = outs_names.iter().position(|n| n == "HALT");
    let mut ran = 0u32;
    let mut stopped = Value::Null;
    if let (Some(n), Some(ci)) = (r.cycles, clk) {
        for _ in 0..n.min(MAX_CYCLES) {
            if halt.is_some_and(|h| run.sim.outputs()[h] == Sig::H) {
                stopped = json!("halt");
                break;
            }
            for v in [true, false] {
                ins[ci] = v;
                run.settle(&ins);
            }
            ran += 1;
            if let Some(pc) = rom_addr(run.sim, &r.circuit) {
                if r.breakpoints.contains(&pc) {
                    stopped = json!({"breakpoint": pc});
                    break;
                }
            }
        }
        if stopped.is_null() && halt.is_some_and(|h| run.sim.outputs()[h] == Sig::H) {
            stopped = json!("halt");
        }
    }
    let (samples, violations, glitchy, max_settle) =
        (run.samples, run.violations, run.glitchy, run.max_settle);
    let internals: Vec<Value> = if r.inspect {
        s.internals()
            .into_iter()
            .map(|(part, st)| json!({"part": part, "regs": st.regs, "mems": st.mems}))
            .collect()
    } else {
        vec![]
    };
    Ok(Json(json!({
        "cycles_run": ran,
        "stopped": stopped,
        "pc": rom_addr(s, &r.circuit),
        "pins": s.pin_values(),
        "outputs": s.outputs(),
        "short": s.short,
        "degraded": s.degraded_outputs().iter().any(|&d| d),
        "unstable": s.unstable,
        "cost": if free { lib.cost(&r.circuit) } else { ctx.circuit_cost(&r.circuit) },
        "trace": samples,
        "timing": if r.timed {
            json!({"now": s.now(), "violations": violations, "glitchy": glitchy, "max_settle": max_settle})
        } else {
            Value::Null
        },
        "internals": internals,
    })))
}
