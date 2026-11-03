//! POST /api/demo: drive a Mark's finished component, for the lesson bench.

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use mark_one_engine::components::sigs;

use crate::app::{level_idx, ApiErr, St};

#[derive(Deserialize)]
pub struct DemoReq {
    level: String,
    #[serde(default)]
    inputs: Vec<bool>,
    #[serde(default)]
    reset: bool,
}

pub async fn demo(State(a): St, Json(r): Json<DemoReq>) -> Result<Json<Value>, ApiErr> {
    level_idx(&r.level)?;
    let mut demos = a.demos.lock().unwrap();
    if r.reset || !demos.contains_key(&r.level) {
        match mark_one_course::model(&r.level) {
            Some(m) => {
                demos.insert(r.level.clone(), m);
            }
            None => {
                return Ok(Json(
                    json!({ "error": "This Mark has no component to try." }),
                ))
            }
        }
    }
    let m = demos.get_mut(&r.level).expect("demo present");
    let mut ins = r.inputs.clone();
    ins.resize(m.n_in(), false);
    Ok(Json(json!({ "outputs": m.eval(&sigs(&ins)) })))
}
