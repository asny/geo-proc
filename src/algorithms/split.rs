use crate::mesh::*;
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