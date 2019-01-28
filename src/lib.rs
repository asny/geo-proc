//! Geometry processing algorithms working on [tri-mesh](https://github.com/asny/tri-mesh) triangle mesh data structure.
//!

pub mod exporter;
pub mod loader;
pub mod connected_components;
pub mod stitching;
pub mod collision;
pub mod cut;

pub use tri_mesh::prelude as prelude;
pub use tri_mesh;
