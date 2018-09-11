use ids::*;
use halfedge_mesh::HalfEdgeMesh;
use mesh::Mesh;
use std::collections::HashSet;

pub fn connected_components(mesh: &HalfEdgeMesh, face_id: &FaceID) -> HashSet<FaceID>
{
    let mut component = HashSet::new();
    let mut to_be_tested = vec![face_id.clone()];

    loop {
        let test_face = match to_be_tested.pop() {
            Some(f) => f,
            None => break
        };

        for mut walker in mesh.face_halfedge_iterator(&test_face) {
            let f = walker.twin().face_id();
            if(!f.is_null() && !component.contains(&f))
            {
                component.insert(f.clone());
                to_be_tested.push(f);
            }
        }
    }
    component

}