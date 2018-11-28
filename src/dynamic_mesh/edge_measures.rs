use dynamic_mesh::*;

impl DynamicMesh
{
    pub fn edge_length(&self, halfedge_id: &HalfEdgeID) -> f32
    {
        let (p0, p1) = self.edge_positions(halfedge_id);
        (p0 - p1).norm()
    }

    pub fn edge_sqr_length(&self, halfedge_id: &HalfEdgeID) -> f32
    {
        let (p0, p1) = self.edge_positions(halfedge_id);
        (p0 - p1).norm_squared()
    }
}