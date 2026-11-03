//! Shared server state.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;

use mark_one_course::{self as course, Ctx, Save};
use mark_one_engine::components::Model;
use mark_one_engine::sim::Sim;

use crate::config::Config;

pub struct App {
    pub save: Mutex<Save>,
    /// Live simulation per board: (topology key, simulator).
    pub sims: Mutex<HashMap<String, (String, Sim)>>,
    /// Reference models backing the lesson benches.
    pub demos: Mutex<HashMap<String, Box<dyn Model>>>,
    pub path: PathBuf,
    pub unlock_all: bool,
}

impl App {
    pub fn new(cfg: &Config) -> App {
        App {
            save: Mutex::new(Save::load(&cfg.save_path)),
            sims: Mutex::new(HashMap::new()),
            demos: Mutex::new(HashMap::new()),
            path: cfg.save_path.clone(),
            unlock_all: cfg.unlock_all,
        }
    }

    /// Progress context for the course, borrowing the loaded save.
    pub fn ctx<'a>(&self, save: &'a Save) -> Ctx<'a> {
        Ctx {
            done: &save.done,
            unlock_all: self.unlock_all,
        }
    }
}

pub type St = State<Arc<App>>;
pub type ApiErr = (StatusCode, String);

pub fn level_idx(id: &str) -> Result<usize, ApiErr> {
    course::index_of(id).ok_or((StatusCode::NOT_FOUND, format!("unknown level {id}")))
}
