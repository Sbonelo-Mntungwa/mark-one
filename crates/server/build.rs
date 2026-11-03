//! Embeds every file under `web/` into the binary, so the server is one
//! self-contained executable. Generates `$OUT_DIR/assets.rs`.

use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("read web dir")
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else if !p
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            out.push(p);
        }
    }
}

fn mime(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

fn main() {
    let web = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../web")
        .canonicalize()
        .expect("web dir");
    println!("cargo:rerun-if-changed={}", web.display());
    let mut files = vec![];
    walk(&web, &mut files);
    let mut src = String::from("pub static FILES: &[(&str, &str, &[u8])] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        let rel = f
            .strip_prefix(&web)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        src += &format!(
            "    ({rel:?}, {:?}, include_bytes!({:?})),\n",
            mime(f),
            f.display().to_string()
        );
    }
    src += "];\n";
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("assets.rs");
    std::fs::write(out, src).expect("write assets.rs");
}
