use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("buildinfo.rs");
    let build_consts = format!(
        r#"
        const BUILDINFO_VERSION: &str = "{}";
        "#,
        env::var("CARGO_PKG_VERSION").unwrap(),
    );
    std::fs::write(&dest_path, build_consts.as_bytes()).unwrap();
}
