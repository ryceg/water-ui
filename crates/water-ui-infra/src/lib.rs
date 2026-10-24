//! Infrastructure for the watercolour engine: the wgpu backend, versioned
//! scene documents, PNG export and the artwork catalogue. Everything here
//! implements or feeds the ports in `water_ui_core::application`.

pub mod authoring;
pub mod document;
pub mod export;
pub mod gpu;

pub use water_ui_core as core;
