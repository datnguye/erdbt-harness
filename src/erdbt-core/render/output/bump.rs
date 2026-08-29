//! Raise the plugin version.
//!
//! `plugin.yaml` is the source of truth for what the plugin publishes, and
//! everything carrying that version downstream is generated — so a bump is one
//! authored line plus a re-render. The crate version is independent: the
//! renderer is a build tool, not the artifact being versioned.

use std::fs;
use std::path::Path;

/// Parse a `MAJOR.MINOR.PATCH` version, rejecting anything else.
///
/// Pre-release and build metadata are refused rather than silently dropped: a
/// bump that turned `1.2.3-rc.1` into `1.3.0` would discard the qualifier
/// without saying so.
pub fn parse(version: &str) -> Result<[u64; 3], String> {
    let bad = || format!("'{version}' is not MAJOR.MINOR.PATCH");
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(bad());
    }
    let mut out = [0u64; 3];
    for (slot, part) in out.iter_mut().zip(parts) {
        if !part.bytes().all(|b| b.is_ascii_digit()) || (part.len() > 1 && part.starts_with('0')) {
            return Err(bad());
        }
        *slot = part.parse().map_err(|_| bad())?;
    }
    Ok(out)
}

/// Resolve `step` — `major`, `minor`, `patch`, or an explicit version — against
/// the version currently authored.
pub fn next(version: &str, step: &str) -> Result<String, String> {
    let [major, minor, patch] = parse(version)?;
    Ok(match step {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{major}.{}.0", minor + 1),
        "patch" => format!("{major}.{minor}.{}", patch + 1),
        explicit => {
            parse(explicit)?;
            explicit.to_string()
        }
    })
}

/// Read the authored version from plugin.yaml.
pub fn current(src_root: &Path) -> Result<String, String> {
    let path = src_root.join("plugin.yaml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.lines()
        .find_map(|line| line.strip_prefix("version:"))
        .map(|rest| rest.trim().trim_matches(['"', '\'']).to_string())
        .ok_or_else(|| format!("{}: no version field", path.display()))
}

/// Write `version` into plugin.yaml, leaving every other line untouched.
pub fn write(src_root: &Path, version: &str) -> Result<(), String> {
    parse(version)?;
    let path = src_root.join("plugin.yaml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut done = false;
    let out: String = text
        .lines()
        .map(|line| {
            if !done && line.starts_with("version:") {
                done = true;
                format!("version: {version}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    if !done {
        return Err(format!("{}: no version field", path.display()));
    }
    fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))
}
