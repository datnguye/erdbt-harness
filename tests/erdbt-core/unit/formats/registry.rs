//! Unit tests for the emitter registry.

#[path = "../../common/mod.rs"]
mod common;

use std::path::Path;

use common::sources;
use erdbt_core::emit::{all_dirs, all_loose, by_name, render, ALL};

#[test]
fn every_emitter_is_reachable_by_name() {
    assert!(!ALL.is_empty());
    for emitter in ALL {
        assert!(
            by_name(emitter.name).is_some(),
            "{} unreachable",
            emitter.name
        );
    }
    assert!(by_name("nonexistent").is_none());
}

/// `tree` clears and drift-checks these paths without knowing which format
/// produced them, so every emitter has to declare what it owns.
#[test]
fn every_emitter_declares_the_paths_it_owns() {
    for emitter in ALL {
        assert!(
            !emitter.dirs.is_empty(),
            "{} owns no directory",
            emitter.name
        );
        let out = (emitter.emit)(&sources(), Path::new("/nonexistent")).unwrap();
        for path in out.keys() {
            let owned = emitter
                .dirs
                .iter()
                .any(|d| path.starts_with(&format!("{d}/")))
                || emitter.loose.contains(&path.as_str());
            assert!(owned, "{} emits unowned path {path}", emitter.name);
        }
    }
}

#[test]
fn rendering_no_emitters_produces_nothing() {
    assert!(render(&sources(), Path::new("/nonexistent"), &[])
        .unwrap()
        .is_empty());
}

#[test]
fn rendering_one_emitter_produces_its_files() {
    let claude = by_name("claude").unwrap();
    let out = render(&sources(), Path::new("/nonexistent"), &[claude]).unwrap();
    assert!(out.contains_key("plugins/erdbt/.claude-plugin/plugin.json"));
    assert!(out.contains_key(".claude-plugin/marketplace.json"));
}

/// A second format must not silently overwrite the first's output; the clash is
/// a build error, not a last-writer-wins race.
#[test]
fn two_emitters_claiming_one_path_is_an_error() {
    let claude = by_name("claude").unwrap();
    let err = render(&sources(), Path::new("/nonexistent"), &[claude, claude]).unwrap_err();
    assert!(err.contains("both claim"), "{err}");
}

#[test]
fn owned_paths_aggregate_across_every_emitter() {
    assert!(all_dirs().contains(&"plugins"));
    assert!(all_loose().contains(&".claude-plugin/marketplace.json"));
}
