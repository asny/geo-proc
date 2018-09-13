use ids::*;
use dynamic_mesh::DynamicMesh;
use std::collections::HashSet;

pub fn connected_components(mesh: &DynamicMesh, face_id: &FaceID) -> HashSet<FaceID>
{
    let mut component = HashSet::new();
    component.insert(face_id.clone());
    let mut to_be_tested = vec![face_id.clone()];

    loop {
        let test_face = match to_be_tested.pop() {
            Some(f) => f,
            None => break
        };

        for mut walker in mesh.face_halfedge_iterator(&test_face) {
            if let Some(face_id) = walker.twin().face_id() {
                if !component.contains(&face_id)
                {
                    component.insert(face_id.clone());
                    to_be_tested.push(face_id);
                }
            }
        }
    }
    component

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_connected_component()
    {
        let mesh = create_connected_test_object();
        let cc = connected_components(&mesh, &FaceID::new(0));
        assert_eq!(cc.len(), mesh.no_faces());
    }

    #[test]
    fn test_three_connected_component()
    {
        let mesh = create_unconnected_test_object();

        let mut cc = connected_components(&mesh, &FaceID::new(0));
        assert_eq!(cc.len(), 12);

        cc = connected_components(&mesh, &FaceID::new(13));
        assert_eq!(cc.len(), 2);

        cc = connected_components(&mesh, &FaceID::new(14));
        assert_eq!(cc.len(), 1);
    }

    fn create_connected_test_object() -> DynamicMesh
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
        DynamicMesh::create(indices, positions, None)
    }

    fn create_unconnected_test_object() -> DynamicMesh
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

        DynamicMesh::create(indices, positions, None)
    }
}