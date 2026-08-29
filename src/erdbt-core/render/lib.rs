//! Render authored erdbt sources into agent-tool formats.
//!
//! `input` loads what was authored, `formats` turns it into files, `output`
//! writes them. The re-exports keep call sites reading `sources::load` rather
//! than `input::sources::load`.

pub mod formats;
pub mod input;
pub mod output;

pub use formats::registry as emit;
pub use input::{frontmatter, sources};
pub use output::{bump, tree};
