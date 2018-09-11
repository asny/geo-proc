use std::cell::{RefCell};
use ids::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ConnectivityInfo {
    vertices: RefCell<HashMap<VertexID, Vertex>>,
    halfedges: RefCell<Vec<HalfEdge>>,
    faces: RefCell<Vec<Face>>,
    no_vertices: RefCell<usize>,
    no_faces: RefCell<usize>
}

impl ConnectivityInfo {
    pub fn new() -> ConnectivityInfo
    {
        ConnectivityInfo { vertices: RefCell::new(HashMap::new()), halfedges: RefCell::new(Vec::new()), faces: RefCell::new(Vec::new()), no_vertices: RefCell::new(0),  no_faces: RefCell::new(0) }
    }

    pub fn no_vertices(&self) -> usize
    {
        RefCell::borrow(&self.no_vertices).clone()
    }

    pub fn no_faces(&self) -> usize
    {
        RefCell::borrow(&self.no_faces).clone()
    }

    pub fn create_vertex(&self) -> VertexID
    {
        let vec = &mut *RefCell::borrow_mut(&self.vertices);
        let id = VertexID::new(vec.len());
        vec.insert(id.clone(), Vertex { id: id.clone(), halfedge: HalfEdgeID::null() });
        *RefCell::borrow_mut(&self.no_vertices) = self.no_vertices() + 1;
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
        let id = HalfEdgeID::new(halfedges.len());
        halfedges.push(HalfEdge { id: id.clone(), vertex: VertexID::null(), twin: HalfEdgeID::null(), next: HalfEdgeID::null(), face: FaceID::null() });
        id
    }

    pub fn create_face(&self) -> FaceID
    {
        let mut vec = RefCell::borrow_mut(&self.faces);
        let id = FaceID::new(vec.len());
        let face = Face { id: id.clone(), halfedge: HalfEdgeID::null() };
        vec.push(face);
        *RefCell::borrow_mut(&self.no_faces) = self.no_faces() + 1;
        id
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = val.clone();
    }

    pub fn set_halfedge_vertex(&self, id: &HalfEdgeID, val: &VertexID)
    {
        RefCell::borrow_mut(&self.halfedges)[id.val()].vertex = val.clone();
    }

    pub fn set_halfedge_next(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges)[id.val()].next = val.clone();
    }

    pub fn set_halfedge_twin(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges)[id.val()].twin = val.clone();
    }

    pub fn set_halfedge_face(&self, id: &HalfEdgeID, val: &FaceID)
    {
        RefCell::borrow_mut(&self.halfedges)[id.val()].face = val.clone();
    }

    pub fn set_face_halfedge(&self, id: &FaceID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces)[id.val()].halfedge = val.clone();
    }

    pub fn vertex_iterator(&self) -> Box<Iterator<Item = VertexID>>
    {
        let vec = &mut *RefCell::borrow_mut(&self.vertices);
        let t: Vec<_> = vec.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn halfedge_first_iter(&self) -> Option<HalfEdgeID>
    {
        self.next_halfedge(-1)
    }

    pub fn halfedge_next_iter(&self, index: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        self.next_halfedge(index.val() as i32)
    }

    fn next_halfedge(&self, index: i32) -> Option<HalfEdgeID>
    {
        let vec = RefCell::borrow(&self.halfedges);
        let mut i = (index + 1) as usize;
        loop {
            if i >= vec.len() { return None; }
            if !vec[i].id().is_null() { return Some(vec[i].id().clone()) }
            i = i+1;
        }
    }

    pub fn face_first_iter(&self) -> Option<FaceID>
    {
        self.next_face(-1)
    }

    pub fn face_next_iter(&self, index: &FaceID) -> Option<FaceID>
    {
        self.next_face(index.val() as i32)
    }

    fn next_face(&self, index: i32) -> Option<FaceID>
    {
        let vec = RefCell::borrow(&self.faces);
        let mut i = (index + 1) as usize;
        loop {
            if i >= vec.len() { return None; }
            if !vec[i].id().is_null() { return Some(vec[i].id().clone()) }
            i = i+1;
        }
    }

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> HalfEdgeID
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> VertexID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].vertex.clone()
    }

    pub fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].twin.clone()
    }

    pub fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].next.clone()
    }

    pub fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> FaceID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].face.clone()
    }

    pub fn face_halfedge(&self, face_id: &FaceID) -> HalfEdgeID
    {
        RefCell::borrow(&self.faces)[face_id.val()].halfedge.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub id: VertexID,
    pub halfedge: HalfEdgeID
}

impl Vertex {
    pub fn id(&self) -> &VertexID
    {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub id: HalfEdgeID,
    pub vertex: VertexID,
    pub twin: HalfEdgeID,
    pub next: HalfEdgeID,
    pub face: FaceID
}

impl HalfEdge {
    pub fn id(&self) -> &HalfEdgeID
    {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub struct Face {
    pub id: FaceID,
    pub halfedge: HalfEdgeID
}

impl Face {
    pub fn id(&self) -> &FaceID
    {
        &self.id
    }
}