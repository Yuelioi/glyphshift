//! Unpublished native Unity IL2CPP Standard UI observer prototype.
//!
//! This package is intentionally absent from the production Catalog and Runtime
//! Bundle. It uses public IL2CPP exports and never writes managed text.

mod diagnostics;
mod font_substitution;
mod main_thread;
mod metadata;
mod native_adapter;
mod objects;
mod observer_loop;
mod runtime_gate;
mod snapshot;
mod writeback;

pub use runtime_gate::{Il2CppRuntimeGate, RuntimeGateError};

pub use native_adapter::glyphshift_adapter_entry_v1;
