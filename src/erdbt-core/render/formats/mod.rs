//! One module per output format.
//!
//! Each exposes `emit(sources, src_root)` plus the `DIRS` and `LOOSE` paths it
//! owns. `emit::ALL` is the registry that binds them; nothing outside this
//! directory names a format.

pub mod claude;
pub mod copilot;
pub mod registry;
