use std::path::{Path, PathBuf};

pub(crate) fn root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let beside_exe = dir.join("assets");
        if beside_exe.join("logo").is_dir() || beside_exe.join("themes").is_dir() {
            return beside_exe;
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub(crate) fn icons_dir() -> PathBuf {
    root().join("icons")
}

pub(crate) fn cursors_dir() -> PathBuf {
    root().join("cursors")
}

pub(crate) fn symbols_dir() -> PathBuf {
    root().join("symbols")
}

pub(crate) fn scripts_dir() -> PathBuf {
    root().join("scripts")
}

pub(crate) fn themes_dir() -> PathBuf {
    root().join("themes")
}

pub(crate) fn fonts_dir() -> PathBuf {
    root().join("fonts")
}

pub(crate) fn theme_file(id: &str) -> PathBuf {
    themes_dir().join(format!("{id}.json"))
}

pub(crate) fn user_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let beside_exe = dir.join("assets");
        if beside_exe.join("themes").is_dir() {
            return dir.join("userdata");
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("userdata")
}

pub(crate) fn user_indicators_dir() -> PathBuf {
    user_root().join("indicators")
}
