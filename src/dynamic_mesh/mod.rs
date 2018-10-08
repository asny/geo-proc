
pub mod ids;
pub mod traversal;
pub mod dynamic_mesh;
pub mod splitting_and_merging;

mod connectivity_info;

pub use dynamic_mesh::dynamic_mesh::*;
pub use dynamic_mesh::ids::*;
pub use dynamic_mesh::traversal::*;
pub use dynamic_mesh::splitting_and_merging::*;