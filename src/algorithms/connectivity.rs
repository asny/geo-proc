
use ids::*;
use dynamic_mesh::DynamicMesh;

pub fn connecting_edge(mesh: &DynamicMesh, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
{
    for mut halfedge in mesh.vertex_halfedge_iterator(vertex_id1) {
        if &halfedge.vertex_id().unwrap() == vertex_id2 {
            return halfedge.halfedge_id()
        }
    }
    None
}

pub fn find_edge(mesh: &DynamicMesh, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
{
    let mut walker = mesh.walker();
    for halfedge_id in mesh.halfedge_iterator() {
        walker.jump_to_edge(&halfedge_id);
        if &walker.vertex_id().unwrap() == vertex_id2 && &walker.twin().vertex_id().unwrap() == vertex_id1
        {
            return Some(halfedge_id)
        }
    }
    None
}