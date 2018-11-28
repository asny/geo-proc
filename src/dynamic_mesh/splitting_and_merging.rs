
use dynamic_mesh::*;
use connected_components::*;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
    MergeWillCreateNonManifoldMesh {message: String},
    CannotCheckOrientationOfMesh {message: String},
    SplittingEdgesDidNotFormAClosesCurve {message: String}
}

impl DynamicMesh
{
    pub fn create_sub_mesh(&self, faces: &HashSet<FaceID>) -> DynamicMesh
    {
        let info = connectivity_info::ConnectivityInfo::new(faces.len(), faces.len());
        for face_id in faces {
            let face = self.connectivity_info.face(face_id).unwrap();
            for mut walker in self.face_halfedge_iterator(face_id) {
                let halfedge_id = walker.halfedge_id().unwrap();
                let halfedge = self.connectivity_info.halfedge(&halfedge_id).unwrap();
                info.add_halfedge(halfedge_id, halfedge);

                let vertex_id = walker.vertex_id().unwrap();
                let vertex = self.connectivity_info.vertex(&vertex_id).unwrap();
                info.add_vertex(vertex_id, vertex);
                info.set_vertex_halfedge(&vertex_id, walker.next_id().unwrap());

                walker.twin();
                if walker.face_id().is_none()
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let mut twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    info.add_halfedge(twin_id, twin);

                }
                else if !faces.contains(&walker.face_id().unwrap())
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let mut twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    twin.face = None;
                    twin.next = None;
                    info.add_halfedge(twin_id, twin);
                }
            }

            info.add_face(face_id.clone(), face);
        }

        let mut positions = HashMap::with_capacity(info.no_vertices());
        let mut normals = HashMap::with_capacity(info.no_vertices());
        for vertex_id in info.vertex_iterator() {
            let p = self.position(&vertex_id).clone();
            positions.insert(vertex_id.clone(), p);
            if let Some(normal) = self.normal(&vertex_id) {
                normals.insert(vertex_id, normal.clone());
            }
        }

        DynamicMesh::create_internal(positions, normals, Rc::new(info))
    }

    pub fn split(&self, is_at_split: &Fn(&DynamicMesh, &HalfEdgeID) -> bool) -> Result<(DynamicMesh, DynamicMesh), Error>
    {
        let mut face_id1 = None;
        let mut face_id2 = None;
        for halfedge_id in self.halfedge_iterator() {
            if is_at_split(self, &halfedge_id) {
                let mut walker = self.walker_from_halfedge(&halfedge_id);
                face_id1 = walker.face_id();
                face_id2 = walker.twin().face_id();
                break;
            }
        }

        let cc1 = if let Some(face_id) = face_id1 {
            connected_component_with_limit(self, &face_id, &|halfedge_id| is_at_split(self, &halfedge_id))
        } else { HashSet::new() };
        let cc2 = if let Some(face_id) = face_id2 {
            connected_component_with_limit(self, &face_id, &|halfedge_id| is_at_split(self, &halfedge_id))
        } else { HashSet::new() };

        if cc1.len() == cc2.len() {
            return Err(Error::SplittingEdgesDidNotFormAClosesCurve {message: format!("It was not possible to split a mesh in two parts, the splitting edges did not form a closed curve.")})
        }

        let sub_mesh1 = self.create_sub_mesh(&cc1);
        let sub_mesh2 = self.create_sub_mesh(&cc2);
        Ok((sub_mesh1, sub_mesh2))
    }

    pub fn merge_with(&mut self, other: &DynamicMesh, stitches: &HashMap<VertexID, VertexID>) -> Result<(), Error>
    {
        let mut is_same_orientation = None;
        // Remove halfedges where the meshes should be stitched
        let mut halfedges_to_remove = HashSet::new();
        for (other_vertex_id1, self_vertex_id1) in stitches {
            for (other_vertex_id2, self_vertex_id2) in stitches {
                if let Some(mut self_halfedge_id) = self.connecting_edge(&self_vertex_id1, &self_vertex_id2)
                {
                    let mut walker = self.walker_from_halfedge(&self_halfedge_id);
                    if walker.face_id().is_some() { walker.twin(); }
                    if walker.face_id().is_some() {
                        return Err(Error::MergeWillCreateNonManifoldMesh {message: format!("Merge at edge ({}, {}) will create non manifold mesh", self_vertex_id1, self_vertex_id2)});
                    }

                    let halfedge_id_to_remove = walker.halfedge_id().unwrap();
                    halfedges_to_remove.insert(halfedge_id_to_remove);

                    // Check if orientation is correct
                    if is_same_orientation.is_none()
                    {
                        let other_halfedge_id = other.connecting_edge(&other_vertex_id1, &other_vertex_id2).ok_or(
                            Error::CannotCheckOrientationOfMesh {message: format!("No edge connecting ({}, {}) exists", other_vertex_id1, other_vertex_id2)}
                        )?;
                        let mut other_walker = other.walker_from_halfedge(&other_halfedge_id);
                        if other_walker.face_id().is_some() { other_walker.twin(); }
                        if other_walker.face_id().is_some() {
                            return Err(Error::MergeWillCreateNonManifoldMesh {message: format!("Merge at edge ({}, {}) will create non manifold mesh", other_vertex_id1, other_vertex_id2)});
                        }

                        is_same_orientation = Some(*stitches.get(&other_walker.vertex_id().unwrap()).unwrap() != walker.vertex_id().unwrap())
                    }
                }
            }
        }

        if let Some(same_orientation) = is_same_orientation { if !same_orientation { self.flip_orientation() } };

        for halfedge_id in halfedges_to_remove {
            self.connectivity_info.remove_halfedge(&halfedge_id);
        }

        let mut mapping = stitches.clone();
        let mut get_or_create_vertex = |mesh: &mut DynamicMesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {return vid.clone();}
            let p = other.position(&vertex_id);
            let n = other.normal(&vertex_id).map(|n| n.clone());
            let vid = mesh.create_vertex(p.clone(), n);
            mapping.insert(vertex_id, vid);
            vid
        };

        for face_id in other.face_iterator() {
            let vertex_ids = other.face_vertices(&face_id);

            let vertex_id0 = get_or_create_vertex(self, vertex_ids.0);
            let vertex_id1 = get_or_create_vertex(self, vertex_ids.1);
            let vertex_id2 = get_or_create_vertex(self, vertex_ids.2);
            self.connectivity_info.create_face(&vertex_id0, &vertex_id1, &vertex_id2);
        }

        self.create_twin_connectivity();
        Ok(())
    }
}

// Stitches is a map of vertex id in the other mesh to vertex id in self where the two meshes should be connected.
pub fn merge(mesh1: &DynamicMesh, mesh2: &DynamicMesh, stitches: &HashMap<VertexID, VertexID>) -> Result<DynamicMesh, Error>
{
    let mut mesh = mesh1.clone();
    mesh.merge_with(mesh2, stitches)?;
    Ok(mesh)
}



#[cfg(test)]
mod tests {
    use super::*;
    use dynamic_mesh::test_utility::*;

    #[test]
    fn test_face_face_merging_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices2, positions2, None);

        let mut mapping = HashMap::new();
        for vertex_id1 in mesh1.vertex_iterator() {
            for vertex_id2 in mesh2.vertex_iterator() {
                if (*mesh1.position(&vertex_id1) - *mesh2.position(&vertex_id2)).norm() < 0.001 {
                    mapping.insert(vertex_id2, vertex_id1);
                }
            }
        }

        let stitched = merge(&mesh1, &mesh2, &mapping).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        assert_eq!(stitched.no_faces(), 2);
        assert_eq!(stitched.no_vertices(), 4);
        test_is_valid(&stitched).unwrap();
    }

    #[test]
    fn test_face_face_merging_at_edge_when_orientation_is_opposite()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
        let mesh2 = DynamicMesh::create(indices2, positions2, None);

        let mut mapping = HashMap::new();
        for vertex_id1 in mesh1.vertex_iterator() {
            for vertex_id2 in mesh2.vertex_iterator() {
                if (*mesh1.position(&vertex_id1) - *mesh2.position(&vertex_id2)).norm() < 0.001 {
                    mapping.insert(vertex_id2, vertex_id1);
                }
            }
        }

        let stitched = merge(&mesh1, &mesh2, &mapping).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        assert_eq!(stitched.no_faces(), 2);
        assert_eq!(stitched.no_vertices(), 4);
        test_is_valid(&stitched).unwrap();
    }

    #[test]
    fn test_create_sub_mesh()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = DynamicMesh::create(indices, positions, None);

        let mut faces = HashSet::new();
        for face_id in mesh.face_iterator() {
            faces.insert(face_id);
            break;
        }

        let sub_mesh = mesh.create_sub_mesh(&faces);

        test_is_valid(&mesh).unwrap();
        test_is_valid(&sub_mesh).unwrap();
    }

    #[test]
    fn test_splitting()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = DynamicMesh::create(indices, positions, None);

        let mut id = None;
        for halfedge_id in mesh.halfedge_iterator() {
            let mut walker = mesh.walker_from_halfedge(&halfedge_id);
            if walker.face_id().is_some() && walker.twin().face_id().is_some()
            {
                id = Some(halfedge_id);
            }
        }

        let (m1, m2) = mesh.split(&|mesh, he_id| {Some(*he_id) == id});
        
        test_is_valid(&mesh).unwrap();
        test_is_valid(&m1).unwrap();
        test_is_valid(&m2).unwrap();
    }
}