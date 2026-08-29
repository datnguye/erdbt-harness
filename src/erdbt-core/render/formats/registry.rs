//! The emitter registry: one module per output format.
//!
//! An emitter turns loaded sources into `{path: contents}` and declares which
//! paths it owns, so `tree` can clear and drift-check them without knowing
//! which format produced them. Adding a format means adding a module here and
//! listing it in `ALL` — nothing else in the build changes.

use std::collections::BTreeMap;
use std::path::Path;

use crate::formats::{claude, copilot};
use crate::sources::Sources;

/// What an emitter produces: the full set of generated files, keyed by path.
pub type Rendered = BTreeMap<String, String>;

/// One format's renderer.
pub type EmitFn = fn(&Sources, &Path) -> Result<Rendered, String>;

/// What one output format produces and owns.
pub struct Emitter {
    /// The name `--format` accepts.
    pub name: &'static str,
    /// Directories this format owns entirely; cleared before each write so a
    /// renamed source never leaves an orphan behind.
    pub dirs: &'static [&'static str],
    /// Generated files outside any owned directory, which a directory sweep
    /// would otherwise leave behind.
    pub loose: &'static [&'static str],
    pub emit: EmitFn,
}

pub const ALL: &[Emitter] = &[
    Emitter {
        name: "claude",
        dirs: claude::DIRS,
        loose: claude::LOOSE,
        emit: claude::emit,
    },
    Emitter {
        name: "copilot",
        dirs: copilot::DIRS,
        loose: copilot::LOOSE,
        emit: copilot::emit,
    },
];

/// Look up an emitter by the name `--format` accepts.
pub fn by_name(name: &str) -> Option<&'static Emitter> {
    ALL.iter().find(|e| e.name == name)
}

/// Every directory any emitter owns.
pub fn all_dirs() -> Vec<&'static str> {
    ALL.iter().flat_map(|e| e.dirs.iter().copied()).collect()
}

/// Every loose file any emitter owns.
pub fn all_loose() -> Vec<&'static str> {
    ALL.iter().flat_map(|e| e.loose.iter().copied()).collect()
}

/// Run the named emitters and merge their output.
pub fn render(src: &Sources, src_root: &Path, emitters: &[&Emitter]) -> Result<Rendered, String> {
    let mut out = Rendered::new();
    for emitter in emitters {
        for (path, text) in (emitter.emit)(src, src_root)? {
            if out.insert(path.clone(), text).is_some() {
                return Err(format!("two emitters both claim {path}"));
            }
        }
    }
    Ok(out)
}
