
pub mod ids;
pub mod traversal;
pub mod dynamic_mesh;
pub mod splitting_and_merging;
pub mod connectivity;
pub mod basic_operations;

mod connectivity_info;

pub use dynamic_mesh::dynamic_mesh::*;
pub use dynamic_mesh::ids::*;
pub use dynamic_mesh::traversal::*;