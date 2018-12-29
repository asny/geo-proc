
pub mod math {
    use cgmath::{Vector2, Vector3};
    pub use cgmath::prelude::*;

    pub type Vec2 = Vector2<f32>;
    pub type Vec3 = Vector3<f32>;

    pub fn vec2(x: f32, y: f32) -> Vec2
    {
        Vector2::new(x, y)
    }
    pub fn vec3(x: f32, y: f32, z: f32) -> Vec3
    {
        Vector3::new(x, y, z)
    }
}

pub mod mesh_builder;
pub mod ids;
pub mod traversal;
pub mod mesh;
pub mod splitting_and_merging;
pub mod connectivity;
pub mod basic_operations;
pub mod quality;
pub mod edge_measures;
pub mod face_measures;

pub mod test_utility;
mod connectivity_info;

pub use crate::dynamic_mesh::mesh::*;
pub use crate::dynamic_mesh::mesh_builder::*;
pub use crate::dynamic_mesh::traversal::*;