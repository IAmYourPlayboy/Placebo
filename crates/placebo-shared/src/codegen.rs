//! Codegen helper for ts-rs. Runs only under the `export-types` feature.
//!
//! All shared DTOs that need TypeScript bindings must derive `TS` and
//! invoke the `#[ts(export, export_to = "../../bindings/")]` attribute.
//! This module contains a single test that writes all registered types
//! to disk. A post-build step copies `bindings/` into `src/types/api/`.

#![cfg(feature = "export-types")]

// Currently there are no shared DTOs marked for export. When the first
// type with #[derive(TS)] is added (M2: auth), cargo test --features
// export-types will write it to crates/placebo-shared/bindings/.
