//! Experiment analyses: test benches, equivalence checks, static timing and
//! export. Each uses the parts the player has unlocked and their custom
//! components, like the truth table.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use mark_one_engine::analysis::{
    bench_from_table, check_equivalence, component_circuit, critical_path, record_bench, run_bench,
};
use mark_one_engine::circuit::Circuit;
use mark_one_engine::export::{kicad_netlist, verilog};
use mark_one_engine::workbench::Library;

use crate::app::{ApiErr, St};

/// Run `f` with the Experiment library.
fn with_lib<T>(a: &crate::app::App, f: impl FnOnce(&Library, &mark_one_course::Save) -> T) -> T {
    let save = a.save.lock().unwrap();
    let ctx = a.ctx(&save);
    let allow = |p: &_| ctx.experiment_allow(p);
    let lib = Library::new(&save.custom, &allow);
    f(&lib, &save)
}

fn reply<T: serde::Serialize>(r: Result<T, String>) -> Json<Value> {
    match r {
        Ok(v) => Json(serde_json::to_value(v).unwrap_or(Value::Null)),
        Err(e) => Json(json!({ "error": e })),
    }
}

#[derive(Deserialize)]
pub struct BenchReq {
    circuit: Circuit,
    #[serde(default)]
    bench: String,
}

/// POST /api/bench: run bench text against a circuit.
pub async fn run(State(a): St, Json(r): Json<BenchReq>) -> Json<Value> {
    reply(with_lib(&a, |lib, _| run_bench(&r.circuit, lib, &r.bench)))
}

/// POST /api/bench/record: fill in the expected outputs from what the
/// circuit does now. With no bench text, write every input combination.
pub async fn record(State(a): St, Json(r): Json<BenchReq>) -> Json<Value> {
    reply(with_lib(&a, |lib, _| {
        if r.bench.trim().is_empty() {
            bench_from_table(&r.circuit, lib)
        } else {
            record_bench(&r.circuit, lib, &r.bench)
        }
        .map(|text| json!({ "bench": text }))
    }))
}

/// GET /api/designs/{name}/bench
pub async fn get_bench(State(a): St, Path(name): Path<String>) -> Json<Value> {
    let s = a.save.lock().unwrap();
    Json(json!({ "bench": s.benches.get(&name).cloned().unwrap_or_default() }))
}

#[derive(Deserialize)]
pub struct BenchText {
    bench: String,
}

/// PUT /api/designs/{name}/bench
pub async fn put_bench(
    State(a): St,
    Path(name): Path<String>,
    Json(b): Json<BenchText>,
) -> Result<StatusCode, ApiErr> {
    if b.bench.len() > 200_000 {
        return Err((StatusCode::BAD_REQUEST, "A bench is at most 200 KB.".into()));
    }
    let mut s = a.save.lock().unwrap();
    if b.bench.trim().is_empty() {
        s.benches.remove(&name);
    } else {
        s.benches.insert(name, b.bench);
    }
    s.store(&a.path);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Against {
    Design(String),
    Component(String),
}

#[derive(Deserialize)]
pub struct EquivReq {
    circuit: Circuit,
    against: Against,
}

/// POST /api/equiv: is the circuit equivalent to a saved design or a
/// component?
pub async fn equiv(State(a): St, Json(r): Json<EquivReq>) -> Json<Value> {
    reply(with_lib(&a, |lib, save| {
        let other = match &r.against {
            Against::Design(n) => save
                .designs
                .get(n)
                .cloned()
                .ok_or_else(|| format!("No design called {n}.")),
            Against::Component(id) => component_circuit(id, lib),
        }?;
        check_equivalence(&r.circuit, &other, lib)
    }))
}

/// POST /api/timing: static critical path.
pub async fn timing(State(a): St, Json(c): Json<Circuit>) -> Json<Value> {
    reply(with_lib(&a, |lib, _| critical_path(&c, lib)))
}

#[derive(Deserialize)]
pub struct ExportReq {
    circuit: Circuit,
    name: String,
    format: String,
}

/// POST /api/export: Verilog or a KiCad netlist, with a file name.
pub async fn export(State(a): St, Json(r): Json<ExportReq>) -> Json<Value> {
    let stem: String = r
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = if stem.trim_matches('_').is_empty() {
        "design".to_string()
    } else {
        stem
    };
    reply(with_lib(&a, |lib, _| match r.format.as_str() {
        "verilog" => verilog(&r.circuit, lib, &r.name)
            .map(|t| json!({"text": t, "filename": format!("{stem}.v")})),
        "kicad" => kicad_netlist(&r.circuit, lib, &r.name)
            .map(|t| json!({"text": t, "filename": format!("{stem}.net")})),
        f => Err(format!("Unknown export format {f}.")),
    }))
}
