//! GET /api/state: the course, progress and component library.

use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::app::St;

pub async fn get(State(a): St) -> Json<Value> {
    let s = a.save.lock().unwrap();
    Json(mark_one_course::state_json(&a.ctx(&s), &s.custom))
}
