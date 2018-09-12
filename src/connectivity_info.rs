use std::cell::{RefCell};
use ids::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ConnectivityInfo {
    vertices: RefCell<HashMap<VertexID, Vertex>>,
    halfedges: RefCell<HashMap<HalfEdgeID, HalfEdge>>,
    faces: RefCell<HashMap<FaceID, Face>>
}

impl ConnectivityInfo {
    pub fn new() -> ConnectivityInfo
    {
        ConnectivityInfo { vertices: RefCell::new(HashMap::new()), halfedges: RefCell::new(HashMap::new()), faces: RefCell::new(HashMap::new()) }
    }

    pub fn no_vertices(&self) -> usize
    {
        RefCell::borrow(&self.vertices).len()
    }

    pub fn no_faces(&self) -> usize
    {
        RefCell::borrow(&self.faces).len()
    }

    pub fn create_vertex(&self) -> VertexID
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);

        let mut i = 0;
        let mut id;
        loop {
            if i == usize::max_value() {panic!("Not possible to create a unique id for a new vertex")}
            id = VertexID::new(i);
            if !vertices.contains_key(&id) { break }
            i = i+1;
        }

        vertices.insert(id.clone(), Vertex { halfedge: None });
        id
    }

    pub fn remove_vertex(&mut self, vertex_id: &VertexID)
    {
        let vec = &mut *RefCell::borrow_mut(&self.vertices);
        vec.remove(vertex_id);
        // TODO: Update references!
    }

    pub fn create_halfedge(&self) -> HalfEdgeID
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);

        let mut i = 0;
        let mut id;
        loop {
            if i == usize::max_value() {panic!("Not possible to create a unique id for a new vertex")}
            id = HalfEdgeID::new(i);
            if !halfedges.contains_key(&id) { break }
            i = i+1;
        }
        halfedges.insert(id.clone(), HalfEdge { vertex: None, twin: None, next: None, face: None });
        id
    }

    pub fn create_face(&self) -> FaceID
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);

        let mut i = 0;
        let mut id;
        loop {
            if i == usize::max_value() {panic!("Not possible to create a unique id for a new vertex")}
            id = FaceID::new(i);
            if !faces.contains_key(&id) { break }
            i = i+1;
        }
        faces.insert(id.clone(), Face { halfedge: None });
        id
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = Some(val.clone());
    }

    pub fn set_halfedge_vertex(&self, id: &HalfEdgeID, val: &VertexID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().vertex = Some(val.clone());
    }

    pub fn set_halfedge_next(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().next = Some(val.clone());
    }

    pub fn set_halfedge_twin(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().twin = Some(val.clone());
    }

    pub fn set_halfedge_face(&self, id: &HalfEdgeID, val: &FaceID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().face = Some(val.clone());
    }

    pub fn set_face_halfedge(&self, id: &FaceID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces).get_mut(id).unwrap().halfedge = Some(val.clone());
    }

    pub fn vertex_iterator(&self) -> Box<Iterator<Item = VertexID>>
    {
        let vertices = RefCell::borrow(&self.vertices);
        let t: Vec<VertexID> = vertices.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn halfedge_iterator(&self) -> Box<Iterator<Item = HalfEdgeID>>
    {
        let halfedges = RefCell::borrow(&self.halfedges);
        let t: Vec<HalfEdgeID> = halfedges.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn face_iterator(&self) -> Box<Iterator<Item = FaceID>>
    {
        let faces = RefCell::borrow(&self.faces);
        let t: Vec<FaceID> = faces.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> Option<VertexID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().vertex.clone()
    }

    pub fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().twin.clone()
    }

    pub fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().next.clone()
    }

    pub fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> Option<FaceID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().face.clone()
    }

    pub fn face_halfedge(&self, face_id: &FaceID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.faces).get(face_id).unwrap().halfedge.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub halfedge: Option<HalfEdgeID>
}

#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub vertex: Option<VertexID>,
    pub twin: Option<HalfEdgeID>,
    pub next: Option<HalfEdgeID>,
    pub face: Option<FaceID>
}

#[derive(Clone, Debug)]
pub struct Face {
    pub halfedge: Option<HalfEdgeID>
}