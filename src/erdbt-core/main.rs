//! Author erdbt skills and commands once in src/erdbt-core/, render them into the plugin.

use erdbt_core::{bump, emit, sources, tree};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Authored content lives beside the renderer that reads it.
const SRC: &str = "src/erdbt-core";

#[derive(Parser)]
#[command(
    name = "erdbt",
    version,
    about = "Author erdbt skills once in src/erdbt-core/, render them into the Claude Code plugin.",
    arg_required_else_help = true
)]
struct Cli {
    /// Repository root; defaults to the current directory.
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render src/erdbt-core/ into the generated plugin tree.
    Render {
        /// Output format; repeat to select several. Defaults to every format.
        #[arg(long = "format")]
        formats: Vec<String>,
    },
    /// Report generated files that drift from a fresh render.
    Check {
        /// Output format; repeat to select several. Defaults to every format.
        #[arg(long = "format")]
        formats: Vec<String>,
    },
    /// List the output formats this build can render.
    Formats,
    /// Delete the generated output tree.
    Clean,
    /// List the authored skills, commands, and agents.
    Skills,
    /// Raise the version in plugin.yaml, then re-render.
    Bump {
        /// major, minor, patch, or an explicit MAJOR.MINOR.PATCH version.
        level: String,
    },
}

fn selected(names: &[String]) -> Result<Vec<&'static emit::Emitter>, String> {
    if names.is_empty() {
        return Ok(emit::ALL.iter().collect());
    }
    names
        .iter()
        .map(|name| {
            emit::by_name(name).ok_or_else(|| {
                let known: Vec<_> = emit::ALL.iter().map(|e| e.name).collect();
                format!(
                    "unknown format '{name}', expected one of: {}",
                    known.join(", ")
                )
            })
        })
        .collect()
}

fn render_all(root: &Path, formats: &[String]) -> Result<BTreeMap<String, String>, String> {
    let src_root = root.join(SRC);
    let src = sources::load(&src_root)?;
    emit::render(&src, &src_root, &selected(formats)?)
}

fn run(cli: Cli) -> Result<String, String> {
    let root = cli.root.unwrap_or_else(|| PathBuf::from("."));
    match cli.command {
        Command::Render { formats } => {
            let rendered = render_all(&root, &formats)?;
            tree::write(&rendered, &root)?;
            Ok(format!("rendered {} files", rendered.len()))
        }
        Command::Check { formats } => {
            let rendered = render_all(&root, &formats)?;
            let drifted = tree::stale(&rendered, &root)?;
            if drifted.is_empty() {
                Ok("up to date".into())
            } else {
                Err(format!(
                    "{} file(s) out of date — run `erdbt render`:\n  {}",
                    drifted.len(),
                    drifted.join("\n  ")
                ))
            }
        }
        Command::Formats => Ok(emit::ALL
            .iter()
            .map(|e| e.name)
            .collect::<Vec<_>>()
            .join("\n")),
        Command::Clean => {
            let removed = tree::clean(&root)?;
            Ok(format!("removed {removed} file(s)"))
        }
        Command::Bump { level } => {
            let src_root = root.join(SRC);
            let from = bump::current(&src_root)?;
            let to = bump::next(&from, &level)?;
            bump::write(&src_root, &to)?;
            let rendered = render_all(&root, &[])?;
            tree::write(&rendered, &root)?;
            Ok(format!(
                "{from} -> {to} ({} files re-rendered)",
                rendered.len()
            ))
        }
        Command::Skills => {
            let src = sources::load(&root.join(SRC))?;
            let mut out = String::new();
            for doc in &src.skills {
                out.push_str(&format!(
                    "skill    {:<22} {}\n",
                    doc.slug,
                    doc.description()
                ));
            }
            for doc in &src.commands {
                out.push_str(&format!(
                    "command  /{:<21} {}\n",
                    doc.slug,
                    doc.description()
                ));
            }
            for doc in &src.agents {
                out.push_str(&format!(
                    "agent    {:<22} {}\n",
                    doc.slug,
                    doc.description()
                ));
            }
            Ok(out.trim_end().to_string())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("erdbt: {error}");
            ExitCode::FAILURE
        }
    }
}
