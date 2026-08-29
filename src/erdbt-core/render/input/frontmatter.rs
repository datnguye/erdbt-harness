//! Minimal YAML frontmatter parsing and rendering.

use serde_yaml::{Mapping, Value};

const DELIMITER: &str = "---";

/// Split a markdown document into its frontmatter mapping and body.
pub fn parse(text: &str) -> Result<(Mapping, String), String> {
    let Some(rest) = text.strip_prefix(DELIMITER) else {
        return Ok((Mapping::new(), text.to_string()));
    };
    let Some((raw, rest)) = rest.split_once(&format!("\n{DELIMITER}")) else {
        return Ok((Mapping::new(), text.to_string()));
    };
    let body = match rest.split_once('\n') {
        Some((trailing, body)) if trailing.trim().is_empty() => body,
        None if rest.trim().is_empty() => "",
        _ => return Err("closing frontmatter delimiter must stand alone on its line".into()),
    };
    let value: Value = serde_yaml::from_str(raw).map_err(|e| e.to_string())?;
    let map = match value {
        Value::Null => Mapping::new(),
        Value::Mapping(m) => m,
        _ => return Err("frontmatter must be a mapping".into()),
    };
    Ok((map, body.trim_start_matches('\n').to_string()))
}

/// Recombine a frontmatter mapping and body into a markdown document.
pub fn render(meta: &Mapping, body: &str) -> Result<String, String> {
    let body = if body.ends_with('\n') {
        body.to_string()
    } else {
        format!("{body}\n")
    };
    if meta.is_empty() {
        return Ok(body);
    }
    let raw = serde_yaml::to_string(meta).map_err(|e| e.to_string())?;
    Ok(format!("{DELIMITER}\n{raw}{DELIMITER}\n\n{body}"))
}
