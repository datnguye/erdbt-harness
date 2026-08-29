//! End-to-end tests: drive the compiled `erdbt` binary against a real tree.
//!
//! These assert the contract a user and CI actually depend on — exit codes and
//! stdout — rather than the internals the unit tests cover.

#[path = "../common/mod.rs"]
mod common;

use std::path::Path;
use std::process::{Command, Output};

use common::{source_tree, TempDir};

fn erdbt(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_erdbt"))
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("failed to run the erdbt binary")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn render_writes_the_plugin_tree() {
    let dir = TempDir::new("cli-render");
    source_tree(&dir);

    let out = erdbt(dir.path(), &["render"]);
    assert!(
        out.status.success(),
        "render failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("rendered"));

    for rel in [
        ".claude-plugin/marketplace.json",
        "plugins/erdbt/.claude-plugin/plugin.json",
        "plugins/erdbt/skills/alpha/SKILL.md",
        "plugins/erdbt/commands/beta.md",
        "plugins/erdbt/hooks/h.sh",
        "plugins/erdbt/README.md",
    ] {
        assert!(dir.path().join(rel).exists(), "render did not write {rel}");
    }
}

/// The whole point of `check` in CI: a fresh render is clean, and any drift
/// from it fails loudly with the offending path.
#[test]
fn check_passes_after_render_and_fails_on_drift() {
    let dir = TempDir::new("cli-check");
    source_tree(&dir);
    erdbt(dir.path(), &["render"]);

    let clean = erdbt(dir.path(), &["check"]);
    assert!(clean.status.success());
    assert!(stdout(&clean).contains("up to date"));

    std::fs::write(dir.path().join("plugins/erdbt/README.md"), "hand edited\n").unwrap();
    let drifted = erdbt(dir.path(), &["check"]);
    assert!(!drifted.status.success(), "check must fail on drift");
    let err = String::from_utf8_lossy(&drifted.stderr);
    assert!(
        err.contains("README.md"),
        "error should name the file: {err}"
    );
    assert!(
        err.contains("erdbt render"),
        "error should say how to fix it: {err}"
    );
}

/// A re-render must be byte-identical, or `check` would report false drift and
/// every CI run would fail at random.
#[test]
fn render_is_deterministic() {
    let dir = TempDir::new("cli-deterministic");
    source_tree(&dir);
    erdbt(dir.path(), &["render"]);
    let first = std::fs::read_to_string(dir.path().join("plugins/erdbt/README.md")).unwrap();
    erdbt(dir.path(), &["render"]);
    let second = std::fs::read_to_string(dir.path().join("plugins/erdbt/README.md")).unwrap();
    assert_eq!(first, second);
    assert!(erdbt(dir.path(), &["check"]).status.success());
}

#[test]
fn clean_removes_output_and_check_then_fails() {
    let dir = TempDir::new("cli-clean");
    source_tree(&dir);
    erdbt(dir.path(), &["render"]);

    let out = erdbt(dir.path(), &["clean"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("removed"));
    assert!(!dir.path().join("plugins/erdbt/README.md").exists());
    assert!(!erdbt(dir.path(), &["check"]).status.success());
}

#[test]
fn skills_lists_authored_content() {
    let dir = TempDir::new("cli-skills");
    source_tree(&dir);
    let out = erdbt(dir.path(), &["skills"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("skill    alpha"), "{text}");
    assert!(text.contains("a skill"), "{text}");
    assert!(text.contains("command  /beta"), "{text}");
}

#[test]
fn a_missing_source_tree_fails_with_a_message_not_a_panic() {
    let dir = TempDir::new("cli-nosrc");
    let out = erdbt(dir.path(), &["render"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.starts_with("erdbt:"),
        "expected a handled error, got: {err}"
    );
    assert!(!err.contains("panicked"), "must not panic: {err}");
}

#[test]
fn an_unknown_shared_block_fails_the_render() {
    let dir = TempDir::new("cli-badblock");
    source_tree(&dir);
    dir.write(
        "src/erdbt-core/content/skills/alpha/SKILL.md",
        "---\ndescription: a\n---\n\n{{ nonexistent }}\n",
    );
    let out = erdbt(dir.path(), &["render"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("nonexistent"));
}

#[test]
fn no_arguments_prints_help_and_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_erdbt")).output().unwrap();
    assert!(!out.status.success());
    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for sub in ["render", "check", "clean", "skills"] {
        assert!(help.contains(sub), "help should list {sub}");
    }
}

/// Build a complete plugin source tree that shares nothing with this repo's
/// own, so the binary is exercised as a user would: an arbitrary project, not
/// the tree it was compiled in.
fn foreign_project(dir: &TempDir) {
    dir.write(
        "src/erdbt-core/plugin.yaml",
        "name: demo\nversion: 3.1.4\ndescription: A plugin built by the binary.\nauthor:\n  name: Tester\n  email: t@example.com\n  url: https://example.com\nhomepage: https://example.com\nrepository: https://example.com\nlicense: MIT\nkeywords:\n  - demo\n",
    );
    dir.write(
        "src/erdbt-core/content/shared/gate.md",
        "SHARED GATE\nsecond line\n",
    );
    dir.write(
        "src/erdbt-core/content/skills/demo/SKILL.md",
        "---\ndescription: demo skill\n---\n\n# Demo\n\n- {{ gate }}\n",
    );
    dir.write(
        "src/erdbt-core/content/commands/go/COMMAND.md",
        "---\ndescription: go command\nargument-hint: \"<x>\"\n---\n\nrun it\n",
    );
    dir.write(
        "src/erdbt-core/content/hooks/h.sh",
        "#!/usr/bin/env bash\necho hook\n",
    );
}

/// The full contract a user depends on, asserted against a project the binary
/// has never seen: the authored version reaches every manifest, a multi-line
/// shared block stays inside its list item, and the render is stable.
#[test]
fn renders_a_foreign_project_end_to_end() {
    let dir = TempDir::new("cli-foreign");
    foreign_project(&dir);

    let out = erdbt(dir.path(), &["render"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let plugin =
        std::fs::read_to_string(dir.path().join("plugins/demo/.claude-plugin/plugin.json"))
            .expect("plugin.json should exist");
    assert!(plugin.contains("\"version\": \"3.1.4\""), "{plugin}");
    assert!(plugin.contains("\"name\": \"demo\""));

    let market = std::fs::read_to_string(dir.path().join(".claude-plugin/marketplace.json"))
        .expect("marketplace.json should exist");
    assert!(market.contains("\"version\": \"3.1.4\""), "{market}");

    let skill = std::fs::read_to_string(dir.path().join("plugins/demo/skills/demo/SKILL.md"))
        .expect("SKILL.md should exist");
    assert!(
        skill.contains("- SHARED GATE\n  second line"),
        "a multi-line shared block must stay inside its list item:\n{skill}"
    );

    assert!(dir.path().join("plugins/demo/commands/go.md").exists());
    assert!(dir.path().join("plugins/demo/hooks/h.sh").exists());
    assert!(erdbt(dir.path(), &["check"]).status.success());
}

/// A rendered hook is executed as a file, so it has to arrive runnable.
#[cfg(unix)]
#[test]
fn rendered_hooks_are_executable() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new("cli-hook-mode");
    foreign_project(&dir);
    erdbt(dir.path(), &["render"]);

    let mode = std::fs::metadata(dir.path().join("plugins/demo/hooks/h.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert!(mode & 0o111 != 0, "hook should be executable, got {mode:o}");
}

/// `bump` raises the authored version and re-renders, so every manifest moves
/// together — that agreement is what `claude plugin tag` validates.
#[test]
fn bump_raises_the_version_everywhere() {
    let dir = TempDir::new("cli-bump");
    foreign_project(&dir);
    erdbt(dir.path(), &["render"]);

    let out = erdbt(dir.path(), &["bump", "minor"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("3.1.4 -> 3.2.0"), "{}", stdout(&out));

    for (rel, needle) in [
        ("src/erdbt-core/plugin.yaml", "version: 3.2.0"),
        (
            "plugins/demo/.claude-plugin/plugin.json",
            "\"version\": \"3.2.0\"",
        ),
        (".claude-plugin/marketplace.json", "\"version\": \"3.2.0\""),
    ] {
        let text = std::fs::read_to_string(dir.path().join(rel)).unwrap();
        assert!(text.contains(needle), "{rel} missing {needle}");
    }
    assert!(erdbt(dir.path(), &["check"]).status.success());
}

#[test]
fn bump_rejects_a_malformed_version() {
    let dir = TempDir::new("cli-bump-bad");
    foreign_project(&dir);
    for bad in ["1.2", "1.2.3-rc.1", "nonsense"] {
        let out = erdbt(dir.path(), &["bump", bad]);
        assert!(!out.status.success(), "'{bad}' should be rejected");
        assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
    }
}

#[test]
fn formats_lists_what_this_build_can_render() {
    let dir = TempDir::new("cli-formats");
    let out = erdbt(dir.path(), &["formats"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("claude"));
}

#[test]
fn render_accepts_an_explicit_format() {
    let dir = TempDir::new("cli-format-explicit");
    foreign_project(&dir);
    assert!(erdbt(dir.path(), &["render", "--format", "claude"])
        .status
        .success());
    assert!(dir
        .path()
        .join("plugins/demo/.claude-plugin/plugin.json")
        .exists());
    assert!(erdbt(dir.path(), &["check", "--format", "claude"])
        .status
        .success());
}

/// A typo must name the formats that exist rather than rendering nothing and
/// reporting success.
#[test]
fn an_unknown_format_fails_with_the_known_ones() {
    let dir = TempDir::new("cli-format-unknown");
    foreign_project(&dir);
    let out = erdbt(dir.path(), &["render", "--format", "cursor"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("cursor"), "{err}");
    assert!(
        err.contains("claude"),
        "should list the known formats: {err}"
    );
}
