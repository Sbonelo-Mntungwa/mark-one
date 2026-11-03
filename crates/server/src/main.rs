//! Mark One server: the web view plus a JSON API over the engine and course.

mod app;
mod assets;
mod config;
mod routes;

use std::sync::Arc;

use app::App;
use config::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::from_env();
    let app = Arc::new(App::new(&cfg));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", cfg.port))
        .await
        .expect("bind port");
    println!(
        "Mark One running at http://127.0.0.1:{}  (save file: {})",
        cfg.port,
        cfg.save_path.display()
    );
    axum::serve(listener, routes::router(app))
        .await
        .expect("server");
}
