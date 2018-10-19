use dynamic_mesh::*;
use types::*;
use std::collections::HashSet;

impl DynamicMesh
{
    pub fn flip_edges(&mut self)
    {
        let mut to_be_flipped = HashSet::new();
        for halfedge_id in self.halfedge_iterator()
        {
            if self.should_flip(&halfedge_id)
            {
                to_be_flipped.insert(halfedge_id);
            }
        }

        while to_be_flipped.len() > 0
        {
            let halfedge_id = to_be_flipped.iter().next().unwrap().clone();
            to_be_flipped.remove(&halfedge_id);

            if self.flip_edge(&halfedge_id).is_ok() {
                let mut walker = self.walker_from_halfedge(&halfedge_id);
                let mut id = walker.next().halfedge_id().unwrap();
                if self.should_flip(&id) { to_be_flipped.insert(id.clone()); } else { to_be_flipped.remove(&id); }

                id = walker.next().halfedge_id().unwrap();
                if self.should_flip(&id) { to_be_flipped.insert(id.clone()); } else { to_be_flipped.remove(&id); }

                id = walker.next().twin().next().halfedge_id().unwrap();
                if self.should_flip(&id) { to_be_flipped.insert(id.clone()); } else { to_be_flipped.remove(&id); }

                id = walker.next().halfedge_id().unwrap();
                if self.should_flip(&id) { to_be_flipped.insert(id.clone()); } else { to_be_flipped.remove(&id); }
            }
        }
    }

    fn should_flip(&self, halfedge_id: &HalfEdgeID) -> bool
    {
        let twin_id = self.walker_from_halfedge(halfedge_id).twin_id().unwrap();
        *halfedge_id < twin_id && !self.on_boundary(halfedge_id) && self.flatness(halfedge_id) < 0.1 && self.flip_will_improve_quality(halfedge_id)
    }

    fn flatness(&self, haledge_id: &HalfEdgeID) -> f32
    {
        0.0
    }

    fn flip_will_improve_quality(&self, haledge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(haledge_id);
        let p0 = self.position(&walker.vertex_id().unwrap());
        let p2 = self.position(&walker.next().vertex_id().unwrap());
        let p1 = self.position(&walker.previous().twin().vertex_id().unwrap());
        let p3 = self.position(&walker.next().vertex_id().unwrap());

        triangle_quality(p0, p1, p2) + triangle_quality(p0, p1, p3) <
            triangle_quality(p0, p2, p3) + triangle_quality(p1, p2, p3)
    }
}

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