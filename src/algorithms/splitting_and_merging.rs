
use mesh::DynamicMesh;
use ids::*;
use traversal::*;
use connected_components::*;
use std::collections::HashSet;

pub fn split_mesh(mesh: &DynamicMesh, is_at_split: &Fn(&DynamicMesh, &HalfEdgeID) -> bool) -> (DynamicMesh, DynamicMesh)
{
    let mut face_id1 = None;
    let mut face_id2 = None;
    for halfedge_id in mesh.halfedge_iterator() {
        if is_at_split(mesh, &halfedge_id) {
            let mut walker = mesh.walker_from_halfedge(&halfedge_id);
            face_id1 = walker.face_id();
            face_id2 = walker.twin().face_id();
            break;
        }
    }

    let cc1 = if let Some(face_id) = face_id1 {
        connected_component_with_limit(mesh, &face_id, &|halfedge_id| is_at_split(mesh, &halfedge_id))
    } else { HashSet::new() };
    let cc2 = if let Some(face_id) = face_id2 {
        connected_component_with_limit(mesh, &face_id, &|halfedge_id| is_at_split(mesh, &halfedge_id))
    } else { HashSet::new() };

    let sub_mesh1 = mesh.create_sub_mesh(&cc1);
    let sub_mesh2 = mesh.create_sub_mesh(&cc2);
    (sub_mesh1, sub_mesh2)
}