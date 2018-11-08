use dynamic_mesh::*;
use types::*;
use std::collections::{HashSet, HashMap};

impl DynamicMesh
{
    pub fn smooth_vertices(&mut self, factor: f32)
    {
        let mut map = HashMap::new();
        for vertex_id in self.vertex_iterator() {
            let mut avg_pos = vec3(0.0, 0.0, 0.0);
            let mut i = 0;
            for walker in self.vertex_halfedge_iterator(&vertex_id) {
                avg_pos = avg_pos + *self.position(&walker.vertex_id().unwrap());
                i = i + 1;
            }
            avg_pos = avg_pos / i as f32;
            let p = self.position(&vertex_id);
            map.insert(vertex_id, p + factor * (avg_pos - p));
        }

        for vertex_id in self.vertex_iterator() {
            self.set_position(vertex_id, *map.get(&vertex_id).unwrap());
        }
    }

    pub fn flip_edges(&mut self, flatness_threshold: f32)
    {
        let insert_or_remove = |mesh: &DynamicMesh, to_be_flipped: &mut HashSet<HalfEdgeID>, halfedge_id: HalfEdgeID| {
            let twin_id = mesh.walker_from_halfedge(&halfedge_id).twin_id().unwrap();
            let id = if halfedge_id < twin_id {halfedge_id} else {twin_id};
            if mesh.should_flip(&id, flatness_threshold) { to_be_flipped.insert(id); } else { to_be_flipped.remove(&id); }
        };

        let mut to_be_flipped = HashSet::new();
        for halfedge_id in self.halfedge_iterator()
        {
            insert_or_remove(&self,&mut to_be_flipped, halfedge_id);
        }

        while to_be_flipped.len() > 0
        {
            let halfedge_id = to_be_flipped.iter().next().unwrap().clone();
            to_be_flipped.remove(&halfedge_id);

            if self.flip_edge(&halfedge_id).is_ok() {
                let mut walker = self.walker_from_halfedge(&halfedge_id);
                insert_or_remove(&self,&mut to_be_flipped, walker.next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.next().twin().next().halfedge_id().unwrap());
                insert_or_remove(&self,&mut to_be_flipped, walker.next().halfedge_id().unwrap());
            }
        }
    }

    fn should_flip(&self, halfedge_id: &HalfEdgeID, flatness_threshold: f32) -> bool
    {
        !self.on_boundary(halfedge_id)
            && self.flatness(halfedge_id) > flatness_threshold
            && !self.flip_will_invert_triangle(halfedge_id)
            && self.flip_will_improve_quality(halfedge_id)
    }

    // 1 = Completely flat, 0 = 90 degrees angle between normals
    fn flatness(&self, haledge_id: &HalfEdgeID) -> f32
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let face_id1 = walker.face_id().unwrap();
        let face_id2 = walker.twin().face_id().unwrap();
        self.face_normal(&face_id1).dot(&self.face_normal(&face_id2))
    }

    fn flip_will_invert_triangle(&self, haledge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.position(&walker.vertex_id().unwrap());
        let p2 = self.position(&walker.next().vertex_id().unwrap());
        let p1 = self.position(&walker.previous().twin().vertex_id().unwrap());
        let p3 = self.position(&walker.next().vertex_id().unwrap());

        (p2 - p0).cross(&(p3 - p0)).dot(&(p3 - p1).cross(&(p2 - p1))) < 0.0001
    }

    fn flip_will_improve_quality(&self, haledge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.position(&walker.vertex_id().unwrap());
        let p2 = self.position(&walker.next().vertex_id().unwrap());
        let p1 = self.position(&walker.previous().twin().vertex_id().unwrap());
        let p3 = self.position(&walker.next().vertex_id().unwrap());

        triangle_quality(p0, p2, p1) + triangle_quality(p0, p1, p3) >
            1.1 * (triangle_quality(p0, p2, p3) + triangle_quality(p1, p3, p2))
    }
}

// Quality measure of 1 = good (equilateral) and >> 1 = bad (needle or flattened)
fn triangle_quality(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> f32
{
    let length01 = (p0-p1).norm();
    let length02 = (p0-p2).norm();
    let length12 = (p1-p2).norm();
    let perimiter = length01 + length02 + length12;
    let area = (p1-p0).cross(&(p2-p0)).norm();
    let inscribed_radius = 2.0 * area / perimiter;
    let circumscribed_radius = length01 * length02 * length12 / (4.0 * area);
    circumscribed_radius / inscribed_radius
}