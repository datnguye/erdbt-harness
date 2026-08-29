#![allow(dead_code)] // each test target compiles this module and uses only part of it

//! Shared fixtures. Every test builds its tree from these, so a change to the
//! source format is made once here rather than in each test file.

use std::fs;
use std::path::{Path, PathBuf};

use erdbt_core::sources::{Author, Doc, Plugin, Sources};
use serde_yaml::{Mapping, Value};

pub const PLUGIN_YAML: &str = "name: erdbt
version: 0.1.0
description: d
author:
  name: n
  email: e
  url: u
homepage: h
repository: r
license: MIT
keywords:
  - dbt
";

/// A scratch directory that removes itself when the test ends, so a failing
/// assertion never leaves state behind to poison the next run.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("erdbt-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn write(&self, rel: &str, text: &str) {
        let target = self.0.join(rel);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, text).unwrap();
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn meta(pairs: &[(&str, Value)]) -> Mapping {
    pairs
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone()))
        .collect()
}

pub fn doc(slug: &str, pairs: &[(&str, Value)], body: &str) -> Doc {
    Doc {
        slug: slug.into(),
        meta: meta(pairs),
        body: body.into(),
        extras: Vec::new(),
    }
}

pub fn plugin() -> Plugin {
    Plugin {
        name: "erdbt".into(),
        version: "0.1.0".into(),
        description: "d".into(),
        author: Author {
            name: "n".into(),
            email: "e".into(),
            url: "u".into(),
        },
        homepage: "h".into(),
        repository: "r".into(),
        license: "MIT".into(),
        keywords: vec!["dbt".into()],
    }
}

pub fn sources() -> Sources {
    Sources {
        plugin: plugin(),
        skills: vec![doc(
            "alpha",
            &[("description", "a".into()), ("license", "MIT".into())],
            "body\n",
        )],
        commands: vec![doc(
            "beta",
            &[("description", "b".into()), ("argument-hint", "<x>".into())],
            "cmd\n",
        )],
        agents: vec![doc(
            "gamma",
            &[("description", "g".into()), ("model", "opus".into())],
            "agent\n",
        )],
    }
}

/// Lay down a complete, valid source tree under `<dir>/src/erdbt-core`.
pub fn source_tree(dir: &TempDir) {
    dir.write("src/erdbt-core/plugin.yaml", PLUGIN_YAML);
    dir.write("src/erdbt-core/content/shared/gate.md", "GATE TEXT\n");
    dir.write(
        "src/erdbt-core/content/skills/alpha/SKILL.md",
        "---\ndescription: a skill\n---\n\n{{ gate }}\n",
    );
    dir.write(
        "src/erdbt-core/content/commands/beta/COMMAND.md",
        "---\ndescription: a command\n---\n\nrun it\n",
    );
    dir.write(
        "src/erdbt-core/content/agents/gamma/AGENT.md",
        "---\ndescription: an agent\nmodel: opus\n---\n\n{{ gate }}\n",
    );
    dir.write("src/erdbt-core/content/hooks/h.sh", "#!/bin/sh\necho hi\n");
}
