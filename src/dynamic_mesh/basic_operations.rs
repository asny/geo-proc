
use dynamic_mesh::*;

#[derive(Debug)]
pub enum Error {
    FailedToFlipEdge {message: String},
}

impl DynamicMesh
{
    pub fn flip_edge(&mut self, halfedge_id: &HalfEdgeID) -> Result<(), Error>
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let face_id = walker.face_id().ok_or(Error::FailedToFlipEdge {message: format!("Trying to flip edge on boundary")})?;
        let next_id = walker.next_id().unwrap();
        let previous_id = walker.previous_id().unwrap();
        let v0 = walker.vertex_id().unwrap();
        walker.next();
        let v3 = walker.vertex_id().unwrap();
        walker.previous();

        walker.twin();
        let twin_id = walker.halfedge_id().unwrap();
        let twin_face_id = walker.face_id().ok_or(Error::FailedToFlipEdge {message: format!("Trying to flip edge on boundary")})?;
        let twin_next_id = walker.next_id().unwrap();
        let twin_previous_id = walker.previous_id().unwrap();
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.next().vertex_id().unwrap();

        if self.connecting_edge(&v2, &v3).is_some() { return Err(Error::FailedToFlipEdge {message: format!("Trying to flip edge which will connect two vertices that are already connected by another edge")}) }

        self.connectivity_info.set_face_halfedge(&face_id, previous_id);
        self.connectivity_info.set_face_halfedge(&twin_face_id, twin_previous_id);

        self.connectivity_info.set_vertex_halfedge(&v0, next_id);
        self.connectivity_info.set_vertex_halfedge(&v1, twin_next_id);

        self.connectivity_info.set_halfedge_next(&halfedge_id, previous_id);
        self.connectivity_info.set_halfedge_next(&next_id, twin_id);
        self.connectivity_info.set_halfedge_next(&previous_id, twin_next_id);
        self.connectivity_info.set_halfedge_next(&twin_id, twin_previous_id);
        self.connectivity_info.set_halfedge_next(&twin_next_id, halfedge_id.clone());
        self.connectivity_info.set_halfedge_next(&twin_previous_id, next_id);

        self.connectivity_info.set_halfedge_vertex(&halfedge_id, v3);
        self.connectivity_info.set_halfedge_vertex(&twin_id, v2);

        self.connectivity_info.set_halfedge_face(&next_id, twin_face_id);
        self.connectivity_info.set_halfedge_face(&twin_next_id, face_id);

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_flip_edge()
    {
        let mut no_flips = 0;
        let mut mesh = ::models::create_plane().unwrap().to_dynamic();
        let no_edges = mesh.no_halfedges();
        for halfedge_id in mesh.halfedge_iterator() {
            let (v0, v1) = mesh.edge_vertices(&halfedge_id);

            if mesh.flip_edge(&halfedge_id).is_ok()
            {
                mesh.test_is_valid().unwrap();

                let (v2, v3) = mesh.edge_vertices(&halfedge_id);
                assert_ne!(v0, v2);
                assert_ne!(v1, v2);
                assert_ne!(v0, v3);
                assert_ne!(v1, v3);

                assert!(mesh.connecting_edge(&v0, &v1).is_none());
                assert!(mesh.connecting_edge(&v2, &v3).is_some());

                let edge = mesh.connecting_edge(&v2, &v3).unwrap();
                let twin = mesh.walker_from_halfedge(&edge).twin_id().unwrap();
                assert!(edge == halfedge_id || twin == halfedge_id,
                        format!("Flipped edge {} or flipped edge twin {} should be equal to before flipped edge id {}", edge, twin, halfedge_id));
                no_flips = no_flips + 1;
            }
        }
        assert_eq!(no_edges, mesh.no_halfedges());
        assert_eq!(no_flips, 2);
    }

    #[test]
    fn test_flip_multiple_edges()
    {
        let mut no_flips = 0;
        let mut mesh = ::models::create_icosahedron().unwrap().to_dynamic();
        let no_edges = mesh.no_halfedges();
        for halfedge_id in mesh.halfedge_iterator() {
            let (v0, v1) = mesh.edge_vertices(&halfedge_id);

            if mesh.flip_edge(&halfedge_id).is_ok()
            {
                mesh.test_is_valid().unwrap();

                let (v2, v3) = mesh.edge_vertices(&halfedge_id);
                assert_ne!(v0, v2);
                assert_ne!(v1, v2);
                assert_ne!(v0, v3);
                assert_ne!(v1, v3);

                assert!(mesh.connecting_edge(&v0, &v1).is_none());
                assert!(mesh.connecting_edge(&v2, &v3).is_some());

                let edge = mesh.connecting_edge(&v2, &v3).unwrap();
                let twin = mesh.walker_from_halfedge(&edge).twin_id().unwrap();
                assert!(edge == halfedge_id || twin == halfedge_id,
                        format!("Flipped edge {} or flipped edge twin {} should be equal to before flipped edge id {}", edge, twin, halfedge_id));
                no_flips = no_flips + 1;
            }
        }
        assert_eq!(no_edges, mesh.no_halfedges());
        assert!(no_flips > 0);
    }
}