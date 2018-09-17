
use na::{Vector2, Vector3, Vector4, Matrix4};

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat4 = Matrix4<f32>;

pub fn vec2(x: f32, y: f32) -> Vec2
{
    Vector2::new(x, y)
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3
{
    Vector3::new(x, y, z)
}

pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4
{
    Vector4::new(x, y, z, w)
}