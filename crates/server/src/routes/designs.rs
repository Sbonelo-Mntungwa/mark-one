//! Free-build designs, saved by name.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use mark_one_engine::circuit::Circuit;

use crate::app::{ApiErr, St};

fn check_name(name: &str) -> Result<(), ApiErr> {
    let ok = !name.trim().is_empty()
        && name.chars().count() <= 60
        && !name.chars().any(|c| c.is_control());
    if ok {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "a design name is 1 to 60 printable characters".into(),
        ))
    }
}

pub async fn list(State(a): St) -> Json<Vec<String>> {
    Json(a.save.lock().unwrap().designs.keys().cloned().collect())
}

pub async fn get(State(a): St, Path(name): Path<String>) -> Result<Json<Circuit>, ApiErr> {
    check_name(&name)?;
    let s = a.save.lock().unwrap();
    Ok(Json(s.designs.get(&name).cloned().unwrap_or_default()))
}

pub async fn put(
    State(a): St,
    Path(name): Path<String>,
    Json(c): Json<Circuit>,
) -> Result<StatusCode, ApiErr> {
    check_name(&name)?;
    let mut s = a.save.lock().unwrap();
    s.designs.insert(name, c);
    s.store(&a.path);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(State(a): St, Path(name): Path<String>) -> StatusCode {
    let mut s = a.save.lock().unwrap();
    s.designs.remove(&name);
    s.benches.remove(&name);
    s.store(&a.path);
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use super::check_name;

    #[test]
    fn names() {
        assert!(check_name("half adder v2").is_ok());
        assert!(check_name("  ").is_err());
        assert!(check_name("a\nb").is_err());
        assert!(check_name(&"x".repeat(61)).is_err());
    }
}
