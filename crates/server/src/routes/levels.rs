//! A Mark's saved draft, and its diagnostics.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::Value;

use mark_one_engine::circuit::Circuit;

use crate::app::{level_idx, ApiErr, St};

pub async fn get_circuit(State(a): St, Path(id): Path<String>) -> Result<Json<Circuit>, ApiErr> {
    level_idx(&id)?;
    let s = a.save.lock().unwrap();
    Ok(Json(s.drafts.get(&id).cloned().unwrap_or_default()))
}

pub async fn put_circuit(
    State(a): St,
    Path(id): Path<String>,
    Json(c): Json<Circuit>,
) -> Result<StatusCode, ApiErr> {
    level_idx(&id)?;
    let mut s = a.save.lock().unwrap();
    s.drafts.insert(id, c);
    s.store(&a.path);
    Ok(StatusCode::NO_CONTENT)
}

/// Run diagnostics. A pass records the best cost; the draft is saved either way.
pub async fn verify(
    State(a): St,
    Path(id): Path<String>,
    Json(c): Json<Circuit>,
) -> Result<Json<Value>, ApiErr> {
    let idx = level_idx(&id)?;
    let mut save = a.save.lock().unwrap();
    let res = {
        let ctx = a.ctx(&save);
        if !ctx.level_open(idx) {
            return Err((StatusCode::FORBIDDEN, "level is locked".into()));
        }
        mark_one_course::verify(idx, &c, &ctx)
    };
    if res.pass {
        let best = save.done.entry(id.clone()).or_insert(res.cost);
        *best = (*best).min(res.cost);
    }
    save.drafts.insert(id, c);
    save.store(&a.path);
    Ok(Json(serde_json::to_value(&res).unwrap_or(Value::Null)))
}
