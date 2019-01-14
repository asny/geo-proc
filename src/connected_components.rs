
use tri_mesh::prelude::*;
use std::collections::HashSet;

pub fn connected_component(mesh: &Mesh, face_id: FaceID) -> HashSet<FaceID>
{
    connected_component_with_limit(mesh, face_id, &|_| false )
}

pub fn connected_component_with_limit(mesh: &Mesh, face_id: FaceID, limit: &Fn(HalfEdgeID) -> bool) -> HashSet<FaceID>
{
    let mut component = HashSet::new();
    component.insert(face_id);
    let mut to_be_tested = vec![face_id];

    loop {
        let test_face = match to_be_tested.pop() {
            Some(f) => f,
            None => break
        };

        for halfedge_id in mesh.face_halfedge_iter(test_face) {
            if !limit(halfedge_id) {
                if let Some(face_id) = mesh.walker_from_halfedge(halfedge_id).into_twin().face_id() {
                    if !component.contains(&face_id)
                    {
                        component.insert(face_id);
                        to_be_tested.push(face_id);
                    }
                }
            }
        }
    }
    component
}

pub fn connected_components(mesh: &Mesh) -> Vec<HashSet<FaceID>>
{
    let mut result = Vec::new();
    while let Some(face_id) = mesh.face_iter().find(|face_id| result.iter().all(|set: &HashSet<FaceID>| !set.contains(face_id)))
    {
        result.push(connected_component(mesh, face_id));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tri_mesh::mesh_builder::MeshBuilder;

    #[test]
    fn test_one_connected_component()
    {
        let mesh = create_connected_test_object();
        let cc = connected_component(&mesh, mesh.face_iter().next().unwrap());
        assert_eq!(cc.len(), mesh.no_faces());
    }

    #[test]
    fn test_connected_components()
    {
        let mesh = create_unconnected_test_object();
        let cc = connected_components(&mesh);

        assert_eq!(cc.len(), 3);

        assert_eq!(cc[0].len() + cc[1].len() + cc[2].len(), 15);
        assert!(cc.iter().find(|vec| vec.len() == 12).is_some());
        assert!(cc.iter().find(|vec| vec.len() == 2).is_some());
        assert!(cc.iter().find(|vec| vec.len() == 1).is_some());
    }

    fn create_connected_test_object() -> Mesh
    {
        let positions: Vec<f32> = vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7
        ];
        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }

    fn create_unconnected_test_object() -> Mesh
    {
        let positions: Vec<f32> = vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0,

            -1.0, 2.0, -1.0,
            -1.0, 3.0, -1.0,
            -2.0, 4.0, -1.0,
            -2.0, 1.0, -1.0,

            -1.0, 3.0, -2.0,
            -2.0, 4.0, -3.0,
            -2.0, 1.0, -4.0
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7,

            8, 9, 10,
            8, 10, 11,

            12, 13, 14
        ];

        MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap()
    }
}