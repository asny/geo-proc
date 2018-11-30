use dynamic_mesh::*;
use types::*;

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

    pub fn split_edge(&mut self, halfedge_id: &HalfEdgeID, position: Vec3) -> VertexID
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        if walker.face_id().is_none()
        {
            walker.twin();
        }
        let split_halfedge_id = walker.halfedge_id().unwrap();

        walker.twin();
        let twin_halfedge_id = walker.halfedge_id().unwrap();
        let twin_vertex_id = walker.vertex_id();
        let is_boundary = walker.face_id().is_none();

        let new_vertex_id = self.create_vertex(position, None);
        self.split_one_face(&split_halfedge_id, twin_halfedge_id.clone(), new_vertex_id.clone());

        if !is_boundary {
            self.split_one_face(&twin_halfedge_id, split_halfedge_id, new_vertex_id.clone());
        }
        else {
            let new_halfedge_id = self.connectivity_info.new_halfedge(twin_vertex_id, None, None);
            self.connectivity_info.set_halfedge_twin(split_halfedge_id, new_halfedge_id);
            self.connectivity_info.set_halfedge_vertex(&twin_halfedge_id, new_vertex_id.clone());
        };

        new_vertex_id
    }

    pub fn split_face(&mut self, face_id: &FaceID, position: Vec3) -> VertexID
    {
        let new_vertex_id = self.create_vertex(position, None);

        let mut walker = self.walker_from_face(face_id);
        let vertex_id1 = walker.vertex_id().unwrap();

        walker.next();
        let halfedge_id2 = walker.halfedge_id().unwrap();
        let twin_id2 = walker.twin_id().unwrap();
        let vertex_id2 = walker.vertex_id().unwrap();

        walker.next();
        let halfedge_id3 = walker.halfedge_id().unwrap();
        let twin_id3 = walker.twin_id().unwrap();
        let vertex_id3 = walker.vertex_id().unwrap();

        let face_id1 = self.connectivity_info.create_face(&vertex_id1, &vertex_id2, &new_vertex_id);
        let face_id2 = self.connectivity_info.create_face(&vertex_id2, &vertex_id3, &new_vertex_id);

        self.connectivity_info.set_halfedge_vertex(&halfedge_id2, new_vertex_id.clone());

        // Update twin information
        let mut new_halfedge_id = HalfEdgeID::new(0);
        for walker in self.face_halfedge_iterator(&face_id1) {
            let vid = walker.vertex_id().unwrap();
            let hid = walker.halfedge_id().unwrap();
            if vid == vertex_id1 {
                self.connectivity_info.set_halfedge_twin(halfedge_id2, hid);
            }
            else if vid == vertex_id2 {
                self.connectivity_info.set_halfedge_twin(twin_id2, hid);
            }
            else if vid == new_vertex_id {
                new_halfedge_id = walker.halfedge_id().unwrap();
            }
            else {
                panic!("Split face failed")
            }
        }
        for walker in self.face_halfedge_iterator(&face_id2) {
            let vid = walker.vertex_id().unwrap();
            let hid = walker.halfedge_id().unwrap();
            if vid == vertex_id2 {
                self.connectivity_info.set_halfedge_twin(new_halfedge_id, hid);
            }
            else if vid == vertex_id3 {
                self.connectivity_info.set_halfedge_twin(twin_id3, hid);
            }
            else if vid == new_vertex_id {
                self.connectivity_info.set_halfedge_twin(halfedge_id3, hid);
            }
            else {
                panic!("Split face failed")
            }
        }
        new_vertex_id
    }

    fn split_one_face(&mut self, halfedge_id: &HalfEdgeID, twin_halfedge_id: HalfEdgeID, new_vertex_id: VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let vertex_id1 = walker.vertex_id().unwrap();

        walker.next();
        let vertex_id2 = walker.vertex_id().unwrap();
        let halfedge_to_update1 = walker.twin_id().unwrap();
        let halfedge_to_update2 = walker.halfedge_id().unwrap();

        self.connectivity_info.set_halfedge_vertex(halfedge_id, new_vertex_id);
        let new_face_id = self.connectivity_info.create_face(&vertex_id1, &vertex_id2, &new_vertex_id);

        // Update twin information
        for walker in self.face_halfedge_iterator(&new_face_id) {
            let vid = walker.vertex_id().unwrap();
            let hid = walker.halfedge_id().unwrap();
            if vid == vertex_id1 {
                self.connectivity_info.set_halfedge_twin(twin_halfedge_id, hid);
            }
            else if vid == vertex_id2 {
                self.connectivity_info.set_halfedge_twin(halfedge_to_update1, hid);
            }
            else if vid == new_vertex_id {
                self.connectivity_info.set_halfedge_twin(halfedge_to_update2, hid);
            }
            else {
                panic!("Split one face failed")
            }
        }
    }

    pub fn collapse_edge(&mut self, halfedge_id: &HalfEdgeID) -> VertexID
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let surviving_vertex_id = walker.vertex_id().unwrap();
        walker.twin();
        let dying_vertex_id = walker.vertex_id().unwrap();

        // Update halfedges pointing to dying vertex
        for walker1 in self.vertex_halfedge_iterator(&dying_vertex_id) {
            self.connectivity_info.set_halfedge_vertex(&walker1.twin_id().unwrap(), surviving_vertex_id);
        }
        for walker2 in self.vertex_halfedge_iterator(&surviving_vertex_id) {
            if walker2.halfedge_id().unwrap() != walker.halfedge_id().unwrap()
            {
                self.connectivity_info.set_vertex_halfedge( &surviving_vertex_id, walker2.halfedge_id().unwrap());

            }
        }

        // Remove first face + halfedges
        let he_id1 = walker.halfedge_id().unwrap();
        if walker.face_id().is_some() {
            self.remove_one_face(&he_id1)
        }
        else {
            self.connectivity_info.remove_halfedge(&he_id1);
        }

        // Remove second face + halfedges
        walker.twin();
        let he_id2 = walker.halfedge_id().unwrap();
        if walker.face_id().is_some() {
            self.remove_one_face(&he_id2)
        }
        else {
            self.connectivity_info.remove_halfedge(&he_id2);
        }

        // Remove dying vertex
        self.connectivity_info.remove_vertex(&dying_vertex_id);

        surviving_vertex_id
    }

    fn remove_one_face(&mut self, halfedge_id: &HalfEdgeID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let face_id = walker.face_id().unwrap();

        walker.next();
        let halfedge_id1 = walker.halfedge_id().unwrap();
        let twin_id1 = walker.twin_id().unwrap();
        walker.next();
        let halfedge_id2 = walker.halfedge_id().unwrap();
        let twin_id2 = walker.twin_id().unwrap();

        self.connectivity_info.set_halfedge_twin(twin_id1, twin_id2);
        self.connectivity_info.remove_face(&face_id);
        self.connectivity_info.remove_halfedge(halfedge_id);
        self.connectivity_info.remove_halfedge(&halfedge_id1);
        self.connectivity_info.remove_halfedge(&halfedge_id2);

        walker.twin();

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dynamic_mesh::test_utility::*;

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
                test_is_valid(&mesh).unwrap();

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
                test_is_valid(&mesh).unwrap();

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

    #[test]
    fn test_split_edge_on_boundary()
    {
        let mut mesh = create_single_face();
        for halfedge_id in mesh.halfedge_iterator()
        {
            if mesh.walker_from_halfedge(&halfedge_id).face_id().is_some()
            {
                mesh.split_edge(&halfedge_id, vec3(-1.0, -1.0, -1.0));

                assert_eq!(mesh.no_vertices(), 4);
                assert_eq!(mesh.no_halfedges(), 2 * 3 + 4);
                assert_eq!(mesh.no_faces(), 2);

                let mut walker = mesh.walker_from_halfedge(&halfedge_id);
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_some());
                assert!(walker.vertex_id().is_some());

                walker.twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_none());
                assert!(walker.vertex_id().is_some());

                walker.twin().next().twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_some());
                assert!(walker.vertex_id().is_some());

                walker.next().next().twin();
                assert!(walker.halfedge_id().is_some());
                assert!(walker.face_id().is_none());
                assert!(walker.vertex_id().is_some());

                test_is_valid(&mesh).unwrap();

                break;
            }
        }
    }

    #[test]
    fn test_split_edge()
    {
        let mut mesh = create_two_connected_faces();
        for halfedge_id in mesh.halfedge_iterator() {
            let mut walker = mesh.walker_from_halfedge(&halfedge_id);
            if walker.face_id().is_some() && walker.twin().face_id().is_some()
            {
                let vertex_id = mesh.split_edge(&halfedge_id, vec3(-1.0, -1.0, -1.0));
                assert_eq!(mesh.no_vertices(), 5);
                assert_eq!(mesh.no_halfedges(), 4 * 3 + 4);
                assert_eq!(mesh.no_faces(), 4);

                let mut w = mesh.walker_from_vertex(&vertex_id);
                let start_halfedge_id = w.halfedge_id();
                let mut end_halfedge_id = w.twin_id();
                for _ in 0..4 {
                    assert!(w.halfedge_id().is_some());
                    assert!(w.twin_id().is_some());
                    assert!(w.vertex_id().is_some());
                    assert!(w.face_id().is_some());
                    w.previous().twin();
                    end_halfedge_id = w.halfedge_id();
                }
                assert_eq!(start_halfedge_id, end_halfedge_id, "Did not go the full round");

                test_is_valid(&mesh).unwrap();
                break;
            }
        }
    }

    #[test]
    fn test_split_face()
    {
        let mut mesh = create_single_face();
        let face_id = mesh.face_iterator().next().unwrap();

        let vertex_id = mesh.split_face(&face_id, vec3(-1.0, -1.0, -1.0));

        assert_eq!(mesh.no_vertices(), 4);
        assert_eq!(mesh.no_halfedges(), 3 * 3 + 3);
        assert_eq!(mesh.no_faces(), 3);

        let mut walker = mesh.walker_from_vertex(&vertex_id);
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin().halfedge_id().unwrap();
        assert_eq!(start_edge, one_round_edge);

        assert!(walker.face_id().is_some());
        walker.next().twin();
        assert!(walker.face_id().is_none());

        walker.twin().next().twin().next().twin();
        assert!(walker.face_id().is_none());

        walker.twin().next().twin().next().twin();
        assert!(walker.face_id().is_none());

        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_collapse_edge_on_boundary()
    {
        let mut mesh = create_two_connected_faces();
        for halfedge_id in mesh.halfedge_iterator()
        {
            if mesh.on_boundary(&halfedge_id)
            {
                let surviving_vertex_id = mesh.collapse_edge(&halfedge_id);

                assert_eq!(mesh.no_vertices(), 3);
                assert_eq!(mesh.no_halfedges(), 6);
                assert_eq!(mesh.no_faces(), 1);

                test_is_valid(&mesh).unwrap();

                break;
            }
        }
    }

    #[test]
    fn test_collapse_edge()
    {
        let mut mesh = create_three_connected_faces();
        for halfedge_id in mesh.halfedge_iterator() {
            let mut walker = mesh.walker_from_halfedge(&halfedge_id);
            if !mesh.on_boundary(&halfedge_id)
            {
                let surviving_vertex_id = mesh.collapse_edge(&halfedge_id);
                assert_eq!(mesh.no_vertices(), 3);
                assert_eq!(mesh.no_halfedges(), 6);
                assert_eq!(mesh.no_faces(), 1);

                test_is_valid(&mesh).unwrap();
                break;
            }
        }
    }
}