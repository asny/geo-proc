
use dynamic_mesh::*;
use connected_components::*;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
    MergeWillCreateNonManifoldMesh {message: String},
    FailedToMergeVertices {message: String},
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
                info.set_vertex_halfedge(&vertex_id, walker.next_id());

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

        DynamicMesh::new_internal(positions, normals, Rc::new(info))
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
            return Err(Error::SplittingEdgesDidNotFormAClosesCurve {message: format!("It was not possible to split a mesh in two parts, the splitting edges and boundary edges did not form a closed curve.")})
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

    pub fn merge_overlapping_primitives(&mut self) -> Result<(), Error>
    {
        let set_of_vertices_to_merge = self.find_overlapping_vertices();
        let set_of_edges_to_merge = self.find_overlapping_edges(&set_of_vertices_to_merge);
        let set_of_faces_to_merge = self.find_overlapping_faces(&set_of_vertices_to_merge);

        for faces_to_merge in set_of_faces_to_merge {
            let mut iter = faces_to_merge.iter();
            iter.next();
            for face_id2 in iter {
                self.remove_face_unsafe(&face_id2);
            }
        }

        for vertices_to_merge in set_of_vertices_to_merge {
            let mut iter = vertices_to_merge.iter();
            let mut vertex_id1 = *iter.next().unwrap();
            for vertex_id2 in iter {
                vertex_id1 = self.merge_vertices(&vertex_id1, vertex_id2)?;
            }
        }

        for edges_to_merge in set_of_edges_to_merge {
            let mut iter = edges_to_merge.iter();
            let mut edge_id1 = *iter.next().unwrap();
            for edge_id2 in iter {
                edge_id1 = self.merge_halfedges(&edge_id1, edge_id2)?;
            }
        }

        Ok(())
    }

    fn merge_halfedges(&mut self, halfedge_id1: &HalfEdgeID, halfedge_id2: &HalfEdgeID) -> Result<HalfEdgeID, Error>
    {
        let mut walker1 = self.walker_from_halfedge(halfedge_id1);
        let mut walker2 = self.walker_from_halfedge(halfedge_id2);

        let edge1_alone =  walker1.face_id().is_none() && walker1.twin().face_id().is_none();
        let edge1_interior =  walker1.face_id().is_some() && walker1.twin().face_id().is_some();
        let edge1_boundary = !edge1_alone && !edge1_interior;

        let edge2_alone =  walker2.face_id().is_none() && walker2.twin().face_id().is_none();
        let edge2_interior =  walker2.face_id().is_some() && walker2.twin().face_id().is_some();
        let edge2_boundary = !edge2_alone && !edge2_interior;

        if edge1_interior && !edge2_alone || edge2_interior && !edge1_alone {
            return Err(Error::FailedToMergeVertices { message: format!("Merging halfedges {} and {} will create a non-manifold mesh", halfedge_id1, halfedge_id2) });
        }

        let mut halfedge_to_remove1 = None;
        let mut halfedge_to_remove2 = None;
        let mut halfedge_to_survive1 = None;
        let mut halfedge_to_survive2 = None;
        let mut vertex_id1 = None;
        let mut vertex_id2 = None;

        if edge1_boundary {
            if walker1.face_id().is_none() { walker1.twin(); };
            halfedge_to_remove1 = walker1.twin_id();
            halfedge_to_survive1 = walker1.halfedge_id();
            vertex_id1 = walker1.vertex_id();
        }
        if edge2_boundary {
            if walker2.face_id().is_none() { walker2.twin(); };
            halfedge_to_remove2 = walker2.twin_id();
            halfedge_to_survive2 = walker2.halfedge_id();
            vertex_id2 = walker2.vertex_id();
        }
        if edge1_alone
        {
            if edge2_interior
            {
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_remove2 = walker1.halfedge_id();

                halfedge_to_survive1 = walker2.halfedge_id();
                vertex_id1 = walker2.vertex_id();
                walker2.twin();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            }
            else {
                if vertex_id2 == walker1.vertex_id() { walker1.twin(); }
                halfedge_to_remove1 = walker1.twin_id();
                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
            }
        }
        if edge2_alone
        {
            if edge1_interior {
                halfedge_to_remove1 = walker2.twin_id();
                halfedge_to_remove2 = walker2.halfedge_id();

                halfedge_to_survive1 = walker1.halfedge_id();
                vertex_id1 = walker1.vertex_id();
                walker1.twin();
                halfedge_to_survive2 = walker1.halfedge_id();
                vertex_id2 = walker1.vertex_id();
            }
            else {
                if vertex_id1 == walker2.vertex_id() { walker2.twin(); }
                halfedge_to_remove2 = walker2.twin_id();
                halfedge_to_survive2 = walker2.halfedge_id();
                vertex_id2 = walker2.vertex_id();
            }
        }

        self.connectivity_info.remove_halfedge(&halfedge_to_remove1.unwrap());
        self.connectivity_info.remove_halfedge(&halfedge_to_remove2.unwrap());
        self.connectivity_info.set_halfedge_twin(halfedge_to_survive1.unwrap(), halfedge_to_survive2.unwrap());
        self.connectivity_info.set_vertex_halfedge(&vertex_id1.unwrap(), halfedge_to_survive2);
        self.connectivity_info.set_vertex_halfedge(&vertex_id2.unwrap(), halfedge_to_survive1);
        Ok(halfedge_to_survive1.unwrap())
    }

    fn merge_vertices(&mut self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Result<VertexID, Error>
    {
        for halfedge_id in self.halfedge_iterator() {
            let mut walker = self.walker_from_halfedge(&halfedge_id);
            if walker.vertex_id().unwrap() == *vertex_id2 {
                self.connectivity_info.set_halfedge_vertex(&walker.halfedge_id().unwrap(), *vertex_id1);
            }
        }
        self.connectivity_info.remove_vertex(vertex_id2);

        Ok(vertex_id1.clone())
    }

    fn find_overlapping_vertices(&self) -> Vec<Vec<VertexID>>
    {
        let mut to_check = HashSet::new();
        self.vertex_iterator().for_each(|v| { to_check.insert(v); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let mut to_merge = Vec::new();
            for id2 in to_check.iter()
            {
                if (self.position(&id1) - self.position(id2)).norm() < 0.00001
                {
                    to_merge.push(*id2);
                }
            }
            if !to_merge.is_empty()
            {
                for id in to_merge.iter()
                {
                    to_check.remove(id);
                }
                to_merge.push(id1);
                set_to_merge.push(to_merge);
            }
        }
        set_to_merge
    }

    fn find_overlapping_faces(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<FaceID>>
    {
        let vertices_to_merge = |vertex_id| {set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))};

        let mut set_of_faces_to_merge = Vec::new();
        for face_id1 in self.face_iterator() {
            let (v0, v1, v2) = self.face_vertices(&face_id1);
            if let Some(vertices_to_merge0) = vertices_to_merge(v0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(v1)
                {
                    if let Some(vertices_to_merge2) = vertices_to_merge(v2)
                    {
                        let mut faces_to_merge = Vec::new();
                        faces_to_merge.push(face_id1);
                        for face_id2 in self.face_iterator() {
                            if face_id1 < face_id2 {
                                let mut is_overlapping = true;
                                for walker in self.face_halfedge_iterator(&face_id2) {
                                    let vid = walker.vertex_id().unwrap();
                                    if !vertices_to_merge0.contains(&vid) && !vertices_to_merge1.contains(&vid) && !vertices_to_merge2.contains(&vid)
                                    {
                                        is_overlapping = false;
                                    }
                                }
                                if is_overlapping { faces_to_merge.push(face_id2) }
                            }
                        }
                        if faces_to_merge.len() > 1 {
                            set_of_faces_to_merge.push(faces_to_merge);
                        }
                    }
                }
            }
        }
        set_of_faces_to_merge
    }

    fn find_overlapping_edges(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<HalfEdgeID>>
    {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.edge_iterator().for_each(|e| { to_check.insert(e); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            if let Some(vertices_to_merge0) = vertices_to_merge(id1.0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(id1.1)
                {
                    let mut to_merge = Vec::new();
                    for id2 in to_check.iter()
                    {
                        if vertices_to_merge0.contains(&id2.0) && vertices_to_merge1.contains(&id2.1)
                            || vertices_to_merge1.contains(&id2.0) && vertices_to_merge0.contains(&id2.1)
                        {
                            to_merge.push(self.connecting_edge(&id2.0, &id2.1).unwrap());
                        }
                    }
                    if !to_merge.is_empty()
                    {
                        for id in to_merge.iter()
                        {
                            to_check.remove(&self.ordered_edge_vertices(id));
                        }
                        to_merge.push(self.connecting_edge(&id1.0, &id1.1).unwrap());
                        set_to_merge.push(to_merge);
                    }
                }
            }
        }
        set_to_merge
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
    use types::*;
    use dynamic_mesh::test_utility::*;

    #[test]
    fn test_face_face_merging_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mesh1 = DynamicMesh::new_with_connectivity(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = DynamicMesh::new_with_connectivity(indices2, positions2, None);

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
        let mesh1 = DynamicMesh::new_with_connectivity(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
        let mesh2 = DynamicMesh::new_with_connectivity(indices2, positions2, None);

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
        let mesh = DynamicMesh::new_with_connectivity(indices, positions, None);

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
        let mesh = DynamicMesh::new_with_connectivity(indices, positions, None);

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

    #[test]
    fn test_merge_overlapping_primitives()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let mut mesh = DynamicMesh::new_with_connectivity((0..9).collect(), positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives_of_cube()
    {
        let mut mesh = ::models::create_unconnected_cube().unwrap().to_dynamic();
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(12, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_individual_faces()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];

        let mut mesh = DynamicMesh::new_with_connectivity((0..9).collect(), positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_faces()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  4, 6, 5,  6, 7, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,
                                       -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,  -1.0, 0.0, 1.5];

        let mut mesh = DynamicMesh::new_with_connectivity(indices, positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_vertices()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
        let mut mesh = DynamicMesh::new_with_connectivity((0..6).collect(), positions, None);

        let mut vertex_id1 = None;
        for vertex_id in mesh.vertex_iterator() {
            if *mesh.position(&vertex_id) == vec3(0.0, 0.0, 0.0)
            {
                if vertex_id1.is_none() { vertex_id1 = Some(vertex_id); }
                else {
                    mesh.merge_vertices(&vertex_id1.unwrap(), &vertex_id).unwrap();
                    break;
                }
            }
        }

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }

    #[test]
    fn test_merge_halfedges()
    {
        let positions: Vec<f32> = vec![1.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, -1.0,
                                       0.0, 0.0, 0.0,  1.0, 0.0, 0.0,  0.0, 0.0, 1.0];
        let mut mesh = DynamicMesh::new_with_connectivity((0..6).collect(), positions, None);

        let mut heid1 = None;
        for (v0, v1) in mesh.edge_iterator() {
            if mesh.position(&v0)[2] == 0.0 && mesh.position(&v1)[2] == 0.0
            {
                let halfedge_id = mesh.connecting_edge(&v0, &v1).unwrap();
                if heid1.is_none() { heid1 = Some((halfedge_id, v0, v1)); }
                else {
                    let (halfedge_id1, v10, v11) = heid1.unwrap();
                    mesh.merge_vertices(&v0, &v11).unwrap();
                    mesh.merge_vertices(&v1, &v10).unwrap();
                    mesh.merge_halfedges(&halfedge_id1, &halfedge_id).unwrap();
                    break;
                }
            }
        }

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
    }
}