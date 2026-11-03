//! The HTTP API, one module per resource.

mod analysis;
mod custom;
mod demo;
mod designs;
mod levels;
mod sim;
mod state;
mod table;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::app::App;
use crate::assets;

pub fn router(app: Arc<App>) -> Router {
    Router::new()
        .route("/api/state", get(state::get))
        .route(
            "/api/levels/{id}/circuit",
            get(levels::get_circuit).put(levels::put_circuit),
        )
        .route("/api/levels/{id}/verify", post(levels::verify))
        .route("/api/sim", post(sim::simulate))
        .route("/api/demo", post(demo::demo))
        .route("/api/designs", get(designs::list))
        .route(
            "/api/designs/{name}",
            get(designs::get).put(designs::put).delete(designs::delete),
        )
        .route("/api/table", post(table::table))
        .route("/api/bench", post(analysis::run))
        .route("/api/bench/record", post(analysis::record))
        .route(
            "/api/designs/{name}/bench",
            get(analysis::get_bench).put(analysis::put_bench),
        )
        .route("/api/equiv", post(analysis::equiv))
        .route("/api/timing", post(analysis::timing))
        .route("/api/export", post(analysis::export))
        .route("/api/custom", post(custom::save))
        .route("/api/custom/{id}", axum::routing::delete(custom::delete))
        .fallback(assets::serve)
        .with_state(app)
}
