//! POST /api/table: the truth table of an Experiment circuit, with the parts
//! the player has unlocked and their custom components.

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use mark_one_engine::analysis::truth_table_with;
use mark_one_engine::circuit::Circuit;
use mark_one_engine::workbench::Library;

use crate::app::St;

pub async fn table(State(a): St, Json(c): Json<Circuit>) -> Json<Value> {
    let save = a.save.lock().unwrap();
    let ctx = a.ctx(&save);
    let allow = |p: &_| ctx.experiment_allow(p);
    let lib = Library::new(&save.custom, &allow);
    match truth_table_with(&c, &lib) {
        Ok(t) => Json(serde_json::to_value(&t).unwrap_or(Value::Null)),
        Err(e) => Json(json!({ "error": e })),
    }
}
