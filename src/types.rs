
use na::{Vector2, Vector3, Vector4, Matrix4};
pub use na::Unit;

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


pub fn rotation_matrix_from_dir_to_dir(source_dir: &Vec3, target_dir: &Vec3) -> Mat4
{
    let c = source_dir.dot(target_dir);
    if c > 0.99999 {
        return Mat4::identity();
    }
    if c < -0.99999 {
        return -Mat4::identity();
    }
    let axis = source_dir.cross(target_dir).normalize();

    let s = (1.0 - c*c).sqrt();
    let oc = 1.0 - c;
    return Mat4::new(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0,                                0.0,                                0.0,                                1.0);
}