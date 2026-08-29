//! Unit tests for raising the plugin version.

#[path = "../../common/mod.rs"]
mod common;

use common::{TempDir, PLUGIN_YAML};
use erdbt_core::bump::{current, next, parse, write};

#[test]
fn each_level_resets_the_lesser_components() {
    assert_eq!(next("1.2.3", "patch").unwrap(), "1.2.4");
    assert_eq!(next("1.2.3", "minor").unwrap(), "1.3.0");
    assert_eq!(next("1.2.3", "major").unwrap(), "2.0.0");
}

/// An explicit version is a valid step, so a release can jump straight to it.
#[test]
fn an_explicit_version_is_taken_as_given() {
    assert_eq!(next("1.2.3", "9.0.1").unwrap(), "9.0.1");
    assert!(next("1.2.3", "nightly").is_err());
}

/// A pre-release qualifier cannot survive a bump, so it is refused outright
/// rather than silently dropped.
#[test]
fn only_plain_semver_is_accepted() {
    assert!(parse("1.2.3").is_ok());
    for bad in ["1.2", "1.2.3.4", "1.2.3-rc.1", "v1.2.3", "", "a.b.c"] {
        assert!(parse(bad).is_err(), "'{bad}' should be rejected");
    }
}

#[test]
fn current_reads_the_authored_version() {
    let dir = TempDir::new("bump-current");
    dir.write("plugin.yaml", PLUGIN_YAML);
    assert_eq!(current(dir.path()).unwrap(), "0.1.0");
}

#[test]
fn a_versionless_plugin_yaml_is_an_error() {
    let dir = TempDir::new("bump-noversion");
    dir.write("plugin.yaml", "name: erdbt\ndescription: d\n");
    assert!(current(dir.path()).is_err());
}

#[test]
fn write_replaces_only_the_version_line() {
    let dir = TempDir::new("bump-write");
    dir.write("plugin.yaml", PLUGIN_YAML);
    write(dir.path(), "2.0.0").unwrap();
    let text = std::fs::read_to_string(dir.path().join("plugin.yaml")).unwrap();
    assert!(text.contains("version: 2.0.0"));
    assert!(text.contains("name: erdbt"), "other fields must survive");
    assert_eq!(text.matches("version:").count(), 1);
}

#[test]
fn write_refuses_a_malformed_version() {
    let dir = TempDir::new("bump-bad");
    dir.write("plugin.yaml", PLUGIN_YAML);
    assert!(write(dir.path(), "1.2").is_err());
}

/// `parse` delegates to `u64::from_str`, which accepts a leading `+` and
/// leading zeros — either would be written into plugin.yaml verbatim and then
/// disagree with the tag `claude plugin tag` derives from it.
#[test]
fn a_non_canonical_version_is_refused() {
    for bad in ["+1.2.3", "1.+2.3", "01.02.03", "1.2.03"] {
        assert!(parse(bad).is_err(), "{bad} should not parse");
        assert!(next("1.2.3", bad).is_err(), "{bad} should not bump");
    }
}
