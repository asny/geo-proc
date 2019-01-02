use crate::prelude::*;
use std::collections::{HashSet, HashMap};

#[derive(Debug)]
pub enum Error {
    MergeWillCreateNonManifoldMesh {message: String},
    CannotCheckOrientationOfMesh {message: String},
}

impl Mesh
{
    pub fn merge_with(&mut self, other: &Mesh, stitches: &HashMap<VertexID, VertexID>) -> Result<(), Error>
    {
        let mut is_same_orientation = None;
        // Remove halfedges where the meshes should be stitched
        let mut halfedges_to_remove = HashSet::new();
        for (other_vertex_id1, self_vertex_id1) in stitches {
            for (other_vertex_id2, self_vertex_id2) in stitches {
                if let Some(self_halfedge_id) = self.connecting_edge(&self_vertex_id1, &self_vertex_id2)
                {
                    let mut walker = self.walker_from_halfedge(&self_halfedge_id);
                    if walker.face_id().is_some() { walker.as_twin(); }
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
                        if other_walker.face_id().is_some() { other_walker.as_twin(); }
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
        let mut get_or_create_vertex = |mesh: &mut Mesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {return vid.clone();}
            let p = other.position(&vertex_id);
            let n = other.normal(&vertex_id).map(|n| n.clone());
            let vid = mesh.create_vertex(p.clone(), n);
            mapping.insert(vertex_id, vid);
            vid
        };

        for face_id in other.face_iter() {
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
pub fn merge(mesh1: &Mesh, mesh2: &Mesh, stitches: &HashMap<VertexID, VertexID>) -> Result<Mesh, Error>
{
    let mut mesh = mesh1.clone();
    mesh.merge_with(mesh2, stitches)?;
    Ok(mesh)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utility::*;

    #[test]
    fn test_face_face_merging_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mesh1 = Mesh::new_with_connectivity(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = Mesh::new_with_connectivity(indices2, positions2, None);

        let mut mapping = HashMap::new();
        for vertex_id1 in mesh1.vertex_iter() {
            for vertex_id2 in mesh2.vertex_iter() {
                if (*mesh1.position(&vertex_id1) - *mesh2.position(&vertex_id2)).magnitude() < 0.001 {
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
        let mesh1 = Mesh::new_with_connectivity(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
        let mesh2 = Mesh::new_with_connectivity(indices2, positions2, None);

        let mut mapping = HashMap::new();
        for vertex_id1 in mesh1.vertex_iter() {
            for vertex_id2 in mesh2.vertex_iter() {
                if (*mesh1.position(&vertex_id1) - *mesh2.position(&vertex_id2)).magnitude() < 0.001 {
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
}