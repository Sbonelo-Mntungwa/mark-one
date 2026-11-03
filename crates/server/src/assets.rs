//! The web view, embedded at build time from `web/` (see `build.rs`).

use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Look up an embedded file by its path under `web/`.
pub fn find(path: &str) -> Option<(&'static str, &'static [u8])> {
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    FILES
        .iter()
        .find(|(p, _, _)| *p == path)
        .map(|(_, m, b)| (*m, *b))
}

pub async fn serve(uri: Uri) -> Response {
    match find(uri.path()) {
        Some((mime, body)) => (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            body,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_is_embedded() {
        let (mime, body) = find("/").expect("index.html");
        assert!(mime.starts_with("text/html"));
        assert!(!body.is_empty());
    }

    #[test]
    fn every_reference_in_the_page_resolves() {
        // Stylesheets and module scripts linked from index.html must exist.
        let (_, body) = find("index.html").unwrap();
        let html = std::str::from_utf8(body).unwrap();
        for attr in ["href=\"", "src=\""] {
            for part in html.split(attr).skip(1) {
                let target = part.split('"').next().unwrap();
                if target.starts_with("http")
                    || target.starts_with("data:")
                    || target.starts_with('#')
                {
                    continue;
                }
                assert!(
                    find(target).is_some(),
                    "index.html references missing file {target}"
                );
            }
        }
    }

    #[test]
    fn unknown_paths_are_not_found() {
        assert!(find("/nope.js").is_none());
    }
}
