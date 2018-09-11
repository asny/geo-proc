use ids::*;
use halfedge_mesh::HalfEdgeMesh;
use mesh::Mesh;

pub fn connected_components(mesh: &HalfEdgeMesh, face_id: &FaceID) -> Vec<FaceID>
{
    let mut component = vec![FaceID::null(); mesh.no_faces()];
    let mut to_be_tested = Vec::new();
    to_be_tested.push(face_id.clone());

    loop {
        let test_face = match to_be_tested.pop() {
            Some(f) => f,
            None => break
        };

        for mut walker in mesh.face_halfedge_iterator(&test_face) {
            let f = walker.twin().face_id();
            let v = f.val();
            if(!f.is_null() && !component[v].is_null())
            {
                component[v] = f.clone();
                to_be_tested.push(f);
            }
        }
    }


    component

}
