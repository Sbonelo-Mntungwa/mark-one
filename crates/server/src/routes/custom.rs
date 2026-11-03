//! Custom components: Experiment designs saved as reusable chips.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use mark_one_engine::circuit::Circuit;
use mark_one_engine::custom::{CustomDef, PREFIX};
use mark_one_engine::workbench::Library;

use crate::app::{ApiErr, St};

#[derive(Deserialize)]
pub struct SaveReq {
    name: String,
    label: String,
    circuit: Circuit,
}

/// A stable id from a component's name: lowercase letters, digits and dashes.
fn id_for(name: &str) -> String {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("{PREFIX}{slug}")
}

/// Save (or replace) a component built from a design. Replacing keeps the id,
/// so circuits already using it pick up the new version.
pub async fn save(State(a): St, Json(r): Json<SaveReq>) -> Result<Json<Value>, ApiErr> {
    let name = r.name.trim();
    if name.is_empty() || name.chars().count() > 40 || name.chars().any(|c| c.is_control()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "A component name is 1 to 40 printable characters.".into(),
        ));
    }
    let id = id_for(name);
    if id == PREFIX {
        return Err((
            StatusCode::BAD_REQUEST,
            "A component name needs at least one letter or digit.".into(),
        ));
    }
    let mut save = a.save.lock().unwrap();
    let def = {
        let ctx = a.ctx(&save);
        let allow = |p: &_| ctx.experiment_allow(p);
        let lib = Library::new(&save.custom, &allow);
        CustomDef::from_design(&id, name, &r.label, r.circuit, &lib)
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?
    };
    save.custom.insert(id.clone(), def);
    save.store(&a.path);
    // Boards using the old version must be rebuilt.
    a.sims.lock().unwrap().clear();
    Ok(Json(json!({ "id": id })))
}

/// Delete a component, unless another component is built from it.
pub async fn delete(State(a): St, Path(id): Path<String>) -> Result<StatusCode, ApiErr> {
    let mut save = a.save.lock().unwrap();
    let users: Vec<&str> = save
        .custom
        .values()
        .filter(|d| {
            d.circuit
                .parts
                .iter()
                .any(|p| p.chip.as_deref() == Some(id.as_str()))
        })
        .map(|d| d.name.as_str())
        .collect();
    if !users.is_empty() {
        return Err((
            StatusCode::CONFLICT,
            format!("Used inside: {}. Delete those first.", users.join(", ")),
        ));
    }
    save.custom.remove(&id);
    save.store(&a.path);
    a.sims.lock().unwrap().clear();
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::id_for;

    #[test]
    fn ids_are_slugs() {
        assert_eq!(id_for("Half Subtractor v2"), "custom:half-subtractor-v2");
        assert_eq!(id_for("  a/b  "), "custom:a-b");
        assert_eq!(id_for("!!"), "custom:");
    }
}
