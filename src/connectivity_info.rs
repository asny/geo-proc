use std::cell::{RefCell};
use ids::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ConnectivityInfo {
    vertices: RefCell<BTreeMap<VertexID, Vertex>>,
    halfedges: RefCell<BTreeMap<HalfEdgeID, HalfEdge>>,
    faces: RefCell<BTreeMap<FaceID, Face>>
}

impl ConnectivityInfo {
    pub fn new() -> ConnectivityInfo
    {
        ConnectivityInfo { vertices: RefCell::new(BTreeMap::new()), halfedges: RefCell::new(BTreeMap::new()), faces: RefCell::new(BTreeMap::new()) }
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

        vertices.insert(id.clone(), Vertex { halfedge: HalfEdgeID::null() });
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
        halfedges.insert(id.clone(), HalfEdge { vertex: VertexID::null(), twin: HalfEdgeID::null(), next: HalfEdgeID::null(), face: FaceID::null() });
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
        faces.insert(id.clone(), Face { halfedge: HalfEdgeID::null() });
        id
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = val.clone();
    }

    pub fn set_halfedge_vertex(&self, id: &HalfEdgeID, val: &VertexID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().vertex = val.clone();
    }

    pub fn set_halfedge_next(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().next = val.clone();
    }

    pub fn set_halfedge_twin(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().twin = val.clone();
    }

    pub fn set_halfedge_face(&self, id: &HalfEdgeID, val: &FaceID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().face = val.clone();
    }

    pub fn set_face_halfedge(&self, id: &FaceID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces).get_mut(id).unwrap().halfedge = val.clone();
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

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> HalfEdgeID
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> VertexID
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().vertex.clone()
    }

    pub fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().twin.clone()
    }

    pub fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().next.clone()
    }

    pub fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> FaceID
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().face.clone()
    }

    pub fn face_halfedge(&self, face_id: &FaceID) -> HalfEdgeID
    {
        RefCell::borrow(&self.faces).get(face_id).unwrap().halfedge.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub halfedge: HalfEdgeID
}

#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub vertex: VertexID,
    pub twin: HalfEdgeID,
    pub next: HalfEdgeID,
    pub face: FaceID
}

#[derive(Clone, Debug)]
pub struct Face {
    pub halfedge: HalfEdgeID
}