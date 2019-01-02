use crate::prelude::*;
use crate::connected_components::*;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Error {
    SplittingEdgesDidNotFormAClosesCurve {message: String}
}

impl Mesh
{
    pub fn split(&self, is_at_split: &Fn(&Mesh, &HalfEdgeID) -> bool) -> Result<(Mesh, Mesh), Error>
    {
        let mut face_id1 = None;
        let mut face_id2 = None;
        for halfedge_id in self.halfedge_iter() {
            if is_at_split(self, &halfedge_id) {
                let mut walker = self.walker_from_halfedge(&halfedge_id);
                face_id1 = walker.face_id();
                face_id2 = walker.as_twin().face_id();
                if face_id1.is_some() && face_id2.is_some()
                    {
                        break;
                    }
            }
        }

        let cc1 = if let Some(face_id) = face_id1 {
            connected_component_with_limit(self, &face_id, &|halfedge_id| is_at_split(self, &halfedge_id))
        } else { HashSet::new() };
        let cc2 = if let Some(face_id) = face_id2 {
            connected_component_with_limit(self, &face_id, &|halfedge_id| is_at_split(self, &halfedge_id))
        } else { HashSet::new() };

        if self.no_faces() != cc1.len() + cc2.len() {
            return Err(Error::SplittingEdgesDidNotFormAClosesCurve { message: format!("It was not possible to split a mesh in two parts, the splitting edges and boundary edges did not form a closed curve.") })
        }

        let sub_mesh1 = self.create_sub_mesh(&cc1);
        let sub_mesh2 = self.create_sub_mesh(&cc2);
        Ok((sub_mesh1, sub_mesh2))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utility::*;
    use crate::MeshBuilder;

    #[test]
    fn test_splitting()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let (m1, m2) = mesh.split(&|mesh,
            he_id| {
                let (p0, p1) = mesh.edge_positions(he_id);
                p0.z > 0.75 && p0.z < 1.75 && p1.z > 0.75 && p1.z < 1.75
            }).unwrap();

        test_is_valid(&mesh).unwrap();
        test_is_valid(&m1).unwrap();
        test_is_valid(&m2).unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m2.no_faces(), 2);
    }
}