//! Write, clean, and drift-check the generated tree.
//!
//! Pure file mechanics: no printing, no exit codes. main.rs wraps these.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::emit;

/// Return the generated dirs a render actually writes into.
///
/// A partial render must not wipe a format it was not asked to write, so the
/// dirs to clear come from the render itself rather than from GENERATED_DIRS.
pub fn owned_dirs(rendered: &BTreeMap<String, String>) -> Vec<&'static str> {
    emit::all_dirs()
        .into_iter()
        .filter(|rel| {
            rendered
                .keys()
                .any(|path| path.starts_with(&format!("{rel}/")))
        })
        .collect()
}

/// Replace the rendered part of the generated tree with fresh content.
pub fn write(rendered: &BTreeMap<String, String>, root: &Path) -> Result<(), String> {
    for rel in owned_dirs(rendered) {
        let target = root.join(rel);
        match fs::remove_dir_all(&target) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", target.display())),
        }
        fs::create_dir_all(&target).map_err(|e| format!("{}: {e}", target.display()))?;
    }
    for (rel, text) in rendered {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
        make_executable_if_script(&target)?;
    }
    Ok(())
}

/// A rendered hook is invoked as a file, so it has to stay runnable; writing it
/// fresh each render would otherwise drop the source's executable bit.
#[cfg(unix)]
fn make_executable_if_script(target: &Path) -> Result<(), String> {
    if target.extension().is_none_or(|e| e != "sh") {
        return Ok(());
    }
    let mut perms = fs::metadata(target)
        .map_err(|e| format!("{}: {e}", target.display()))?
        .permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(target, perms).map_err(|e| format!("{}: {e}", target.display()))
}

#[cfg(not(unix))]
fn make_executable_if_script(_target: &Path) -> Result<(), String> {
    Ok(())
}

/// Collect every generated file under `dir`.
///
/// A directory that does not exist yields nothing — that is the normal state
/// before a first render. Any other failure propagates: a drift check that
/// cannot read a directory must say so rather than report the tree as clean.
fn files_under(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("{}: {e}", dir.display())),
    };
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.is_dir() {
            files_under(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// Delete generated output. A render recreates every directory it writes into.
pub fn clean(root: &Path) -> Result<usize, String> {
    let mut removed = 0;
    for rel in emit::all_dirs() {
        let target = root.join(rel);
        let mut found = Vec::new();
        files_under(&target, &mut found)?;
        removed += found.len();
        match fs::remove_dir_all(&target) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", target.display())),
        }
    }
    for rel in emit::all_loose() {
        let target = root.join(rel);
        match fs::remove_file(&target) {
            Ok(()) => removed += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", target.display())),
        }
    }
    Ok(removed)
}

/// Return generated paths whose on-disk content differs from the render.
///
/// Orphans are hunted only in the dirs this render owns, so checking one
/// format never reports another format's files as strays.
pub fn stale(rendered: &BTreeMap<String, String>, root: &Path) -> Result<Vec<String>, String> {
    let mut drifted: Vec<String> = rendered
        .iter()
        .filter(|(rel, text)| fs::read_to_string(root.join(rel)).ok().as_ref() != Some(*text))
        .map(|(rel, _)| rel.clone())
        .collect();
    for rel in owned_dirs(rendered) {
        let base = root.join(rel);
        let mut found = Vec::new();
        files_under(&base, &mut found)?;
        for path in found {
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let key = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if !rendered.contains_key(&key) {
                drifted.push(key);
            }
        }
    }
    drifted.sort();
    drifted.dedup();
    Ok(drifted)
}
