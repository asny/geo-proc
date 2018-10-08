
use dynamic_mesh::*;
use connected_components::*;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

impl DynamicMesh
{
    pub fn create_sub_mesh(&self, faces: &HashSet<FaceID>) -> DynamicMesh
    {
        let info = connectivity_info::ConnectivityInfo::new(faces.len(), faces.len());
        for face_id in faces {
            let face = self.connectivity_info.face(face_id).unwrap();
            for walker in self.face_halfedge_iterator(face_id) {
                let halfedge_id = walker.halfedge_id().unwrap();
                let halfedge = self.connectivity_info.halfedge(&halfedge_id).unwrap();
                info.add_halfedge(halfedge_id, halfedge);

                let twin_id = walker.twin_id().unwrap();
                let twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                info.add_halfedge(twin_id, twin);

                let vertex_id = walker.vertex_id().unwrap();
                let vertex = self.connectivity_info.vertex(&vertex_id).unwrap();
                info.add_vertex(vertex_id, vertex);
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

    pub fn split(&self, is_at_split: &Fn(&DynamicMesh, &HalfEdgeID) -> bool) -> (DynamicMesh, DynamicMesh)
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

        let sub_mesh1 = self.create_sub_mesh(&cc1);
        let sub_mesh2 = self.create_sub_mesh(&cc2);
        (sub_mesh1, sub_mesh2)
    }

    pub fn merge_with(&mut self, other: &DynamicMesh, stitches: &HashMap<VertexID, VertexID>)
    {
        let mut mapping = stitches.clone();
        let mut get_or_create_vertex = |mesh: &mut DynamicMesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {return vid.clone();}
            let p = other.position(&vertex_id);
            let n = other.normal(&vertex_id).map(|n| n.clone());
            let vid = mesh.create_vertex(p.clone(), n);
            mapping.insert(vertex_id, vid);
            vid
        };

        let stitch_edge = |mesh: &mut DynamicMesh, halfedge_id|
        {
            let mut walker = mesh.walker_from_halfedge(&halfedge_id);
            if walker.face_id().is_some() { walker.twin(); }
            if walker.face_id().is_some() { panic!("Merge will create non manifold mesh") }

            mesh.connectivity_info.remove_halfedge(&walker.halfedge_id().unwrap());
        };

        for face_id in other.face_iterator() {

            let vertex_ids = other.face_vertices(&face_id);
            let vertex_id0 = get_or_create_vertex(self, vertex_ids.0);
            let vertex_id1 = get_or_create_vertex(self, vertex_ids.1);
            let vertex_id2 = get_or_create_vertex(self, vertex_ids.2);

            if stitches.contains_key(&vertex_ids.0) && stitches.contains_key(&vertex_ids.1)
                && ::connectivity::connecting_edge(other, &vertex_ids.0, &vertex_ids.1).is_some()
            {
                let halfedge_id = ::connectivity::connecting_edge(self, &vertex_id0, &vertex_id1).unwrap();
                stitch_edge(self, halfedge_id);
            }
            if stitches.contains_key(&vertex_ids.1) && stitches.contains_key(&vertex_ids.2)
                && ::connectivity::connecting_edge(other, &vertex_ids.1, &vertex_ids.2).is_some()
            {
                let halfedge_id = ::connectivity::connecting_edge(self, &vertex_id1, &vertex_id2).unwrap();
                stitch_edge(self, halfedge_id);
            }
            if stitches.contains_key(&vertex_ids.2) && stitches.contains_key(&vertex_ids.0)
                && ::connectivity::connecting_edge(other, &vertex_ids.2, &vertex_ids.0).is_some()
            {
                let halfedge_id = ::connectivity::connecting_edge(self, &vertex_id2, &vertex_id0).unwrap();
                stitch_edge(self, halfedge_id);
            }

            self.connectivity_info.create_face(&vertex_id0, &vertex_id1, &vertex_id2);
        }

        self.create_twin_connectivity();
    }
}

// Stitches is a map of vertex id in the other mesh to vertex id in self where the two meshes should be connected.
pub fn merge(mesh1: &DynamicMesh, mesh2: &DynamicMesh, stitches: &HashMap<VertexID, VertexID>) -> DynamicMesh
{
    let mut mesh = mesh1.clone();
    mesh.merge_with(mesh2, stitches);
    mesh
}