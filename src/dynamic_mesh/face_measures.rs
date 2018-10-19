use dynamic_mesh::*;
use types::*;

impl DynamicMesh
{
    pub fn compute_face_normal(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        let mut dir = v0.cross(&v1);
        dir.normalize_mut();
        dir
    }

    pub fn area(&self, face_id: &FaceID) -> f32
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        v0.cross(&v1).norm()
    }

    pub fn center(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p1 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p2 = *self.position(&walker.vertex_id().unwrap());

        (p0 + p1 + p2)/3.0
    }
}