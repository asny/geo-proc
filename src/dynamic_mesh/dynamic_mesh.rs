use mesh::{self, Renderable};
use dynamic_mesh::connectivity_info::ConnectivityInfo;
use dynamic_mesh::*;
use types::*;
use std::rc::Rc;
use std::collections::HashMap;

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
pub type HalfEdgeIterator = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIterator = Box<Iterator<Item = FaceID>>;
pub type EdgeIterator = Box<Iterator<Item = (VertexID, VertexID)>>;

#[derive(Clone, Debug)]
pub struct DynamicMesh {
    positions: HashMap<VertexID, Vec3>,
    normals: HashMap<VertexID, Vec3>,
    pub(super) connectivity_info: Rc<ConnectivityInfo>
}

impl Renderable for DynamicMesh
{
    fn indices(&self) -> Vec<u32>
    {
        let vertices: Vec<VertexID> = self.vertex_iterator().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iterator()
        {
            for walker in self.face_halfedge_iterator(&face_id) {
                let vertex_id = walker.vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        indices
    }

    fn get_attribute(&self, name: &str) -> Option<mesh::Attribute>
    {
        match name {
            "position" => {
                let mut pos = Vec::with_capacity(self.no_vertices() * 3);
                for v3 in self.vertex_iterator().map(|ref vertex_id| self.positions.get(vertex_id).unwrap()) {
                    pos.push(v3.x); pos.push(v3.y); pos.push(v3.z);
                }
                Some(mesh::Attribute::new("position", 3, pos))
            },
            "normal" => {
                if self.normals.len() != self.no_vertices() {
                    return None;
                }
                let mut nor = Vec::with_capacity(self.no_vertices() * 3);
                for vertex_id in self.vertex_iterator() {
                    if let Some(normal) = self.normals.get(&vertex_id)
                    {
                        nor.push(normal.x); nor.push(normal.y); nor.push(normal.z);
                    }
                    else { return None; }
                }
                Some(mesh::Attribute::new("normal", 3, nor))
            },
            _ => None
        }
    }

    fn no_vertices(&self) -> usize
    {
        self.no_vertices()
    }
}

impl DynamicMesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>, normals: Option<Vec<f32>>) -> DynamicMesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = DynamicMesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)),
            positions: HashMap::new(), normals: HashMap::new()};

        for i in 0..no_vertices {
            let nor = match normals { Some(ref data) => Some(vec3(data[i*3], data[i*3+1], data[i*3+2])), None => None };
            mesh.create_vertex(vec3(positions[i*3], positions[i*3+1], positions[i*3+2]), nor);
        }

        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);
            mesh.connectivity_info.create_face(&v0, &v1, &v2);
        }
        mesh.create_twin_connectivity();
        mesh
    }

    pub(super) fn create_internal(positions: HashMap<VertexID, Vec3>, normals: HashMap<VertexID, Vec3>, connectivity_info: Rc<ConnectivityInfo>) -> DynamicMesh
    {
        DynamicMesh {positions, normals, connectivity_info}
    }

    pub fn no_vertices(&self) -> usize
    {
        self.connectivity_info.no_vertices()
    }

    pub fn no_halfedges(&self) -> usize
    {
        self.connectivity_info.no_halfedges()
    }

    pub fn no_faces(&self) -> usize
    {
        self.connectivity_info.no_faces()
    }

    pub fn test_is_valid(&self) -> Result<(), mesh::Error>
    {
        for vertex_id in self.vertex_iterator() {
            if let Some(halfedge_id) = self.walker_from_vertex(&vertex_id).halfedge_id()
            {
                if !self.halfedge_iterator().any(|he_id| he_id == halfedge_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Vertex {} points to an invalid halfedge {}", vertex_id, halfedge_id)});
                }
            }
            else {
                return Err(mesh::Error::IsNotValid {message: format!("Vertex {} does not point to a halfedge", vertex_id)});
            }
        }
        for halfedge_id in self.halfedge_iterator() {
            let walker = self.walker_from_halfedge(&halfedge_id);

            if let Some(twin_id) = walker.twin_id()
            {
                if !self.halfedge_iterator().any(|he_id| he_id == twin_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to an invalid twin halfedge {}", halfedge_id, twin_id)});
                }
            }
            else {
                return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} does not point to a twin halfedge", halfedge_id)});
            }

            if let Some(vertex_id) = walker.vertex_id()
            {
                if !self.vertex_iterator().any(|vid| vid == vertex_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to an invalid vertex {}", halfedge_id, vertex_id)});
                }
            }
            else {
                return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} does not point to a vertex", halfedge_id)});
            }

            if let Some(face_id) = walker.face_id()
            {
                if !self.face_iterator().any(|fid| fid == face_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to an invalid face {}", halfedge_id, face_id)});
                }
                if walker.next_id().is_none() {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to a face but not a next halfedge", halfedge_id)});
                }
            }

            if let Some(next_id) = walker.next_id()
            {
                if !self.halfedge_iterator().any(|he_id| he_id == next_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to an invalid next halfedge {}", halfedge_id, next_id)});
                }
                if walker.face_id().is_none() {
                    return Err(mesh::Error::IsNotValid {message: format!("Halfedge {} points to a next halfedge but not a face", halfedge_id)});
                }
            }
        }
        for face_id in self.face_iterator() {
            if let Some(halfedge_id) = self.walker_from_face(&face_id).halfedge_id()
            {
                if !self.halfedge_iterator().any(|he_id| he_id == halfedge_id) {
                    return Err(mesh::Error::IsNotValid {message: format!("Face {} points to an invalid halfedge {}", face_id, halfedge_id)});
                }
            }
            else {
                return Err(mesh::Error::IsNotValid {message: format!("Face {} does not point to a halfedge", face_id)});
            }
        }

        for vertex_id1 in self.vertex_iterator()
        {
            for vertex_id2 in self.vertex_iterator()
            {
                if self.connecting_edge(&vertex_id1, &vertex_id2).is_some() != self.connecting_edge(&vertex_id2, &vertex_id1).is_some()
                {
                    return Err(mesh::Error::IsNotValid {message: format!("Vertex {} and Vertex {} is connected one way, but not the other way", vertex_id1, vertex_id2)});
                }
            }
        }
        Ok(())
    }

    pub fn edge_vertices(&self, halfedge_id: &HalfEdgeID) -> (VertexID, VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.twin().vertex_id().unwrap();
        (v1, v2)
    }

    pub fn ordered_edge_vertices(&self, halfedge_id: &HalfEdgeID) -> (VertexID, VertexID)
    {
        let mut walker = self.walker_from_halfedge(halfedge_id);
        let v1 = walker.vertex_id().unwrap();
        let v2 = walker.twin().vertex_id().unwrap();
        if v1 < v2 { (v1, v2) } else { (v2, v1) }
    }

    pub fn face_vertices(&self, face_id: &FaceID) -> (VertexID, VertexID, VertexID)
    {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.next();
        let v2 = walker.vertex_id().unwrap();
        walker.next();
        let v3 = walker.vertex_id().unwrap();
        (v1, v2, v3)
    }

    pub fn ordered_face_vertices(&self, face_id: &FaceID) -> (VertexID, VertexID, VertexID)
    {
        let mut walker = self.walker_from_face(face_id);
        let v1 = walker.vertex_id().unwrap();
        walker.next();
        let v2 = walker.vertex_id().unwrap();
        walker.next();
        let v3 = walker.vertex_id().unwrap();
        if v1 < v2 {
            if v2 < v3 { (v1, v2, v3) }
            else { if v1 < v3 { (v1, v3, v2) } else { (v3, v1, v2) } }
        }
        else {
            if v1 < v3 { (v2, v1, v3) }
            else { if v2 < v3 { (v2, v3, v1) } else { (v3, v2, v1) } }
        }
    }

    //////////////////////////////////////////
    // *** Connectivity changing functions ***
    //////////////////////////////////////////

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

    ////////////////////////////////////////////
    // *** Functions related to the position ***
    ////////////////////////////////////////////

    pub fn position(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.positions.get(vertex_id).unwrap()
    }

    pub fn set_position(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.positions.insert(vertex_id, value);
    }

    pub fn move_vertex(&mut self, vertex_id: VertexID, value: Vec3)
    {
        let mut p = value;
        {
            p = p + *self.positions.get(&vertex_id).unwrap();
        }
        self.positions.insert(vertex_id, p);
    }

    pub fn edge_positions(&self, halfedge_id: &HalfEdgeID) -> (&Vec3, &Vec3)
    {
        let vertices = self.ordered_edge_vertices(halfedge_id);
        (self.position(&vertices.0), self.position(&vertices.1))
    }

    pub fn face_positions(&self, face_id: &FaceID) -> (&Vec3, &Vec3, &Vec3)
    {
        let vertices = self.ordered_face_vertices(face_id);
        (self.position(&vertices.0), self.position(&vertices.1), self.position(&vertices.2))
    }

    //////////////////////////////////////////
    // *** Functions related to the normal ***
    //////////////////////////////////////////

    pub fn normal(&self, vertex_id: &VertexID) ->  Option<&Vec3>
    {
        self.normals.get(vertex_id)
    }

    pub fn set_normal(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.normals.insert(vertex_id, value);
    }

    pub fn compute_face_normal(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        let mut dir = v0.cross(&v1);
        dir.normalize_mut();
        dir
    }

    pub fn compute_vertex_normal(&self, vertex_id: &VertexID) -> Vec3
    {
        let mut normal = vec3(0.0, 0.0, 0.0);
        for walker in self.vertex_halfedge_iterator(&vertex_id) {
            if let Some(face_id) = walker.face_id() {
                normal = normal + self.compute_face_normal(&face_id)
            }
        }
        normal.normalize_mut();
        normal
    }

    pub fn update_vertex_normals(&mut self)
    {
        for vertex_id in self.vertex_iterator() {
            let normal = self.compute_vertex_normal(&vertex_id);
            self.set_normal(vertex_id, normal);
        }
    }

    pub fn area(&self, face_id: &FaceID) -> f32
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let v0 = *self.position(&walker.vertex_id().unwrap()) - p0;
        walker.next();
        let v1 = *self.position(&walker.vertex_id().unwrap()) - p0;

        v0.cross(&v1).norm()
    }

    pub fn center(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p1 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p2 = *self.position(&walker.vertex_id().unwrap());

        (p0 + p1 + p2)/3.0
    }

    ///////////////////////////////////////////////////
    // *** Internal connectivity changing functions ***
    ///////////////////////////////////////////////////

    pub(super) fn create_vertex(&mut self, position: Vec3, normal: Option<Vec3>) -> VertexID
    {
        let id = self.connectivity_info.new_vertex();
        self.positions.insert(id.clone(), position);
        if let Some(nor) = normal {self.normals.insert(id.clone(), nor);}
        id
    }

    pub(super) fn create_twin_connectivity(&mut self)
    {
        let mut walker = Walker::create(&self.connectivity_info);
        let edges: Vec<HalfEdgeID> = self.halfedge_iterator().collect();

        for i1 in 0..edges.len()
        {
            let halfedge_id1 = edges[i1];
            if walker.jump_to_edge(&halfedge_id1).twin_id().is_none()
            {
                let vertex_id1 = walker.vertex_id().unwrap();
                let vertex_id2 = walker.previous().vertex_id().unwrap();

                let mut halfedge2 = None;
                for i2 in i1+1..edges.len()
                {
                    let halfedge_id2 = &edges[i2];
                    if walker.jump_to_edge(halfedge_id2).twin_id().is_none()
                    {
                        if walker.vertex_id().unwrap() == vertex_id2 && walker.previous().vertex_id().unwrap() == vertex_id1
                        {
                            halfedge2 = Some(halfedge_id2.clone());
                            break;
                        }
                    }
                }
                let halfedge_id2 = halfedge2.unwrap_or_else(|| {
                        self.connectivity_info.new_halfedge(Some(vertex_id2), None, None)
                    });
                self.connectivity_info.set_halfedge_twin(halfedge_id1, halfedge_id2);

            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_face_connectivity() {
        let mut mesh = DynamicMesh::create(vec![], vec![], None);

        let v1 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v2 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v3 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let f1 = mesh.connectivity_info.create_face(&v1, &v2, &v3);
        mesh.create_twin_connectivity();

        let t1 = mesh.walker_from_vertex(&v1).vertex_id();
        assert_eq!(t1, Some(v2.clone()));

        let t2 = mesh.walker_from_vertex(&v1).twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(&v2.clone()).next().next().vertex_id();
        assert_eq!(t3, Some(v2.clone()));

        let t4 = mesh.walker_from_face(&f1.clone()).twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(&f1.clone()).twin().next_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(&f1.clone()).previous().previous().twin().twin().face_id();
        assert_eq!(t6, Some(f1.clone()));

        let t7 = mesh.walker_from_vertex(&v2.clone()).next().next().next_id();
        assert_eq!(t7, mesh.walker_from_vertex(&v2).halfedge_id());

        let t8 = mesh.walker_from_vertex(&v3).face_id();
        assert_eq!(t8, Some(f1));
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = create_three_connected_faces();
        let mut id = None;
        for vertex_id in mesh.vertex_iterator() {
            let mut round = true;
            for walker in mesh.vertex_halfedge_iterator(&vertex_id) {
                if walker.face_id().is_none() { round = false; break; }
            }
            if round { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(&id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin_id().unwrap();
        assert_eq!(start_edge, one_round_edge);
    }

    #[test]
    fn test_vertex_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.vertex_iterator() {
            i = i+1;
        }
        assert_eq!(4, i);

        // Test that two iterations return the same result
        let vec: Vec<VertexID> = mesh.vertex_iterator().collect();
        i = 0;
        for vertex_id in mesh.vertex_iterator() {
            assert_eq!(vertex_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.halfedge_iterator() {
            i = i+1;
        }
        assert_eq!(12, i);

        // Test that two iterations return the same result
        let vec: Vec<HalfEdgeID> = mesh.halfedge_iterator().collect();
        i = 0;
        for halfedge_id in mesh.halfedge_iterator() {
            assert_eq!(halfedge_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_face_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.face_iterator() {
            i = i+1;
        }
        assert_eq!(3, i);

        // Test that two iterations return the same result
        let vec: Vec<FaceID> = mesh.face_iterator().collect();
        i = 0;
        for face_id in mesh.face_iterator() {
            assert_eq!(face_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_vertex_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        let vertex_id = mesh.vertex_iterator().last().unwrap();
        for edge in mesh.vertex_halfedge_iterator(&vertex_id) {
            assert!(edge.vertex_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_vertex_halfedge_iterator_with_holes() {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 4, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0; 5 * 3];
        let mesh = DynamicMesh::create(indices, positions, None);

        let mut i = 0;
        for edge in mesh.vertex_halfedge_iterator(&VertexID::new(0)) {
            assert!(edge.vertex_id().is_some());
            i = i+1;
        }
        assert_eq!(i,4, "All edges of a one-ring are not visited");

    }

    #[test]
    fn test_face_halfedge_iterator() {
        let mesh = create_single_face();
        let mut i = 0;
        for mut edge in mesh.face_halfedge_iterator(&FaceID::new(0)) {
            assert!(edge.halfedge_id().is_some());
            assert!(edge.face_id().is_some());
            i = i+1;
        }
        assert_eq!(i, 3, "All edges of a face are not visited");
    }

    #[test]
    fn test_face_normal() {
        let mesh = create_single_face();
        let computed_normal = mesh.compute_face_normal(&FaceID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }

    #[test]
    fn test_vertex_normal() {
        let mesh = create_three_connected_faces();
        let computed_normal = mesh.compute_vertex_normal(&VertexID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }

    #[test]
    fn test_update_normals() {
        let mut mesh = create_three_connected_faces();
        mesh.update_vertex_normals();

        for vertex_id in mesh.vertex_iterator() {
            let normal = mesh.normal(&vertex_id).unwrap();
            assert_eq!(0.0, normal.x);
            assert_eq!(1.0, normal.y);
            assert_eq!(0.0, normal.z);
        }
    }

    #[test]
    fn test_get_attribute() {
        let mut mesh = create_three_connected_faces();
        mesh.update_vertex_normals();

        let data = mesh.get_attribute("normal").unwrap().data;
        assert_eq!(data.len(), mesh.no_vertices() * 3);
        for i in 0..mesh.no_vertices() {
            assert_eq!(0.0, data[i * 3]);
            assert_eq!(1.0, data[i * 3+1]);
            assert_eq!(0.0, data[i * 3+2]);
        }
    }

    /*#[test]
    fn test_remove_face()
    {
        let mut mesh = ::models::create_cube_as_dynamic_mesh().unwrap();
        let face_id = mesh.face_iterator().next().unwrap();
        mesh.connectivity_info.remove_face(&face_id);

        //mesh.test_is_valid().unwrap(); Is not valid!

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(11, mesh.no_faces());

        let mut i = 0;
        for face_id in mesh.face_iterator()
        {
            mesh.connectivity_info.remove_face(&face_id);
            i = i+1;
        }
        assert_eq!(i, 11);
        assert_eq!(0, mesh.no_vertices());
        assert_eq!(0, mesh.no_halfedges());
        assert_eq!(0, mesh.no_faces());

        mesh.test_is_valid().unwrap();
    }*/

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

                mesh.test_is_valid().unwrap();

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

                mesh.test_is_valid().unwrap();
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

        mesh.test_is_valid().unwrap();
    }

    fn create_single_face() -> DynamicMesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        DynamicMesh::create((0..3).collect(), positions, None)
    }

    fn create_two_connected_faces() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
        DynamicMesh::create(indices, positions, None)
    }

    fn create_three_connected_faces() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
        let normals: Vec<f32> = vec![0.0; 4 * 3];
        DynamicMesh::create(indices, positions, Some(normals))
    }
}