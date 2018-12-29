
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