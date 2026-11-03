//! Settings read from the environment.

use std::path::PathBuf;

pub struct Config {
    pub port: u16,
    /// Progress, drafts and designs.
    pub save_path: PathBuf,
    /// Open every Mark and chip (sandbox mode).
    pub unlock_all: bool,
}

impl Config {
    pub fn from_env() -> Config {
        let var = |k: &str| std::env::var(k).ok();
        Config {
            port: var("PORT").and_then(|p| p.parse().ok()).unwrap_or(7878),
            save_path: PathBuf::from(
                var("MARK_ONE_SAVE").unwrap_or_else(|| "mark_one_save.json".into()),
            ),
            unlock_all: var("UNLOCK_ALL").is_some_and(|v| v == "1" || v == "true"),
        }
    }
}
