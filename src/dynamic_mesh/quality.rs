use dynamic_mesh::*;

impl DynamicMesh
{
    pub fn flip_edges(&mut self)
    {
        let edges = self.halfedge_iterator();
        for halfedge_id in edges
        {
            if !self.on_boundary(&halfedge_id) && self.flatness(&halfedge_id) < 0.1 && self.flip_will_improve_quality(&halfedge_id)
            {
                self.flip_edge(&halfedge_id).unwrap();
            }
        }
    }

    fn flatness(&self, haledge_id: &HalfEdgeID) -> f32
    {
        0.0
    }

    fn flip_will_improve_quality(&self, haledge_id: &HalfEdgeID) -> bool
    {
        true
    }
}