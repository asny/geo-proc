
use na::Vector3;

pub type Vec3 = Vector3<f32>;

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3
{
    Vector3::new(x, y, z)
}
