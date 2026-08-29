//! Unit tests for YAML frontmatter parsing and rendering.

#[path = "../../common/mod.rs"]
mod common;

use common::meta;
use erdbt_core::frontmatter::{parse, render};
use serde_yaml::Mapping;

#[test]
fn parses_frontmatter_and_body() {
    let (m, body) = parse("---\nname: x\n---\n\nhello\n").unwrap();
    assert_eq!(m.get("name").unwrap().as_str(), Some("x"));
    assert_eq!(body, "hello\n");
}

#[test]
fn body_without_frontmatter_is_returned_whole() {
    let (m, body) = parse("# just markdown\n").unwrap();
    assert!(m.is_empty());
    assert_eq!(body, "# just markdown\n");
}

#[test]
fn unterminated_frontmatter_is_not_treated_as_meta() {
    let (m, body) = parse("---\nname: x\nno closing delimiter\n").unwrap();
    assert!(m.is_empty());
    assert!(body.starts_with("---"));
}

#[test]
fn scalar_frontmatter_is_rejected() {
    assert!(parse("---\njust a string\n---\nbody\n").is_err());
}

#[test]
fn empty_frontmatter_parses_as_empty_mapping() {
    let (m, body) = parse("---\n\n---\n\nbody\n").unwrap();
    assert!(m.is_empty());
    assert_eq!(body, "body\n");
}

#[test]
fn invalid_yaml_is_an_error() {
    assert!(parse("---\na: [unclosed\n---\nbody\n").is_err());
}

#[test]
fn render_round_trips_parse() {
    let text = "---\nname: x\n---\n\nhello\n";
    let (m, body) = parse(text).unwrap();
    let (m2, body2) = parse(&render(&m, &body).unwrap()).unwrap();
    assert_eq!(m, m2);
    assert_eq!(body, body2);
}

#[test]
fn render_without_meta_emits_bare_body() {
    assert_eq!(render(&Mapping::new(), "hi").unwrap(), "hi\n");
    assert_eq!(render(&Mapping::new(), "hi\n").unwrap(), "hi\n");
}

#[test]
fn render_appends_trailing_newline() {
    let out = render(&meta(&[("a", "b".into())]), "body no newline").unwrap();
    assert!(out.ends_with("body no newline\n"));
}

/// Frontmatter key order is what makes a re-render byte-identical, so it must
/// survive rendering rather than being sorted alphabetically.
#[test]
fn render_preserves_key_order() {
    let out = render(
        &meta(&[("name", "x".into()), ("description", "y".into())]),
        "b",
    )
    .unwrap();
    assert!(out.find("name").unwrap() < out.find("description").unwrap());
}

/// A closing delimiter with trailing content is malformed; accepting it would
/// silently leak that content into the body.
#[test]
fn closing_delimiter_must_stand_alone() {
    assert!(parse("---\nname: x\n---extra\nbody\n").is_err());
    assert!(parse("---\nname: x\n--- \n\nbody\n").is_ok());
}
