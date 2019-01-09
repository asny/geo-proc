//! A triangle mesh data structure including basic operations.
//!
//! Why yet another triangle mesh data structure crate you might ask.
//! Well, if you want a more feature complete crate than [half_edge_mesh](https://crates.io/crates/half_edge_mesh) and a less generic crate than [plexus](https://crates.io/crates/plexus),
//! then `tri-mesh` is probably something for you!
//!
//! ## Features
//! - An implementation of the [halfedge mesh data structure](mesh/struct.Mesh.html)
//! - [Iterators](mesh/struct.Mesh.html#impl) over primitives (vertices, halfedges, faces)
//! - Halfedge [walker](mesh/struct.Mesh.html#impl) to efficiently traverse the mesh
//! - Convenient [connectivity](mesh/struct.Mesh.html#impl-2) functionality (e.g. vertices of a face, edge between two vertices)
//! - Simple measures on primitives (e.g. area of face, length of edge)
//! - Basic manipulation functionality (e.g. split edge, collapse edge, flip edge)
//! - Orientation manipulation functionality (e.g. flip orientation of all faces)
//! - Mesh quality manipulation functionality (e.g. flip edges recursively to improve triangle quality, collapse small faces)
//! - And more...
//!
//! Most functionality is implemented as methods on the [Mesh](mesh/struct.Mesh.html) struct,
//! so take a look at that rather long list of functions for a complete overview.
//! The only exception is the [Mesh builder](mesh_builder/struct.MeshBuilder.html) which is used to construct a new mesh.



pub mod mesh;
pub mod loader;
pub mod exporter;
pub mod algorithms;
pub mod mesh_builder;
pub mod test_utility;
pub mod prelude;

pub use crate::algorithms::*;

pub use crate::mesh_builder::MeshBuilder as MeshBuilder;
