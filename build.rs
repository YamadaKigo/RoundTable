use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/fonts");

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let mut arms = String::new();
    for file in ["simsun.ttc", "vgafix.fon"] {
        let path = Path::new("assets/fonts").join(file);
        if path.is_file() {
            let absolute = fs::canonicalize(path).expect("font path");
            let text = absolute.to_string_lossy().replace('\\', "/");
            arms.push_str(&format!(
                "        {:?} => Some(include_bytes!({:?})),\n",
                file, text
            ));
        }
    }
    let source = if arms.is_empty() {
        "pub(crate) fn embedded_font(_file: &str) -> Option<&'static [u8]> {\n    None\n}\n"
            .to_owned()
    } else {
        format!(
            "pub(crate) fn embedded_font(file: &str) -> Option<&'static [u8]> {{\n    match file {{\n{arms}        _ => None,\n    }}\n}}\n"
        )
    };
    fs::write(out.join("embedded_fonts.rs"), source).expect("embedded fonts");
}
