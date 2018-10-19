
use dynamic_mesh::*;

impl DynamicMesh
{
    pub fn connecting_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        for mut halfedge in self.vertex_halfedge_iterator(vertex_id1) {
            if &halfedge.vertex_id().unwrap() == vertex_id2 {
                return halfedge.halfedge_id()
            }
        }
        None
    }

    pub fn find_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        let mut walker = self.walker();
        for halfedge_id in self.halfedge_iterator() {
            walker.jump_to_edge(&halfedge_id);
            if &walker.vertex_id().unwrap() == vertex_id2 && &walker.twin().vertex_id().unwrap() == vertex_id1
            {
                return Some(halfedge_id)
            }
        }
        None
    }

    pub fn on_boundary(&self, halfedge_id: &HalfEdgeID) -> bool
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        walker.face_id().is_none() || walker.twin().face_id().is_none()
    }
}