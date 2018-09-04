use std::cell::{RefCell};
use ids::*;

#[derive(Debug)]
pub struct ConnectivityInfo {
    vertices: RefCell<Vec<Vertex>>,
    halfedges: RefCell<Vec<HalfEdge>>,
    faces: RefCell<Vec<Face>>
}

impl ConnectivityInfo {
    pub fn new() -> ConnectivityInfo
    {
        ConnectivityInfo { vertices: RefCell::new(Vec::new()), halfedges: RefCell::new(Vec::new()), faces: RefCell::new(Vec::new()) }
    }

    pub fn create_vertex(&self) -> VertexID
    {
        let vec = &mut *RefCell::borrow_mut(&self.vertices);
        let id = VertexID::new(vec.len());
        vec.push(Vertex { halfedge: HalfEdgeID::null() });
        id
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
        let face = Face { halfedge: HalfEdgeID::null() };
        vec.push(face);
        id
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.vertices)[id.val()].halfedge = val.clone();
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

    pub fn halfedge_first_iter(&self) -> Option<HalfEdgeID>
    {
        let halfedges = RefCell::borrow(&self.halfedges);
        let no_halfedges = halfedges.len();
        let mut i = 0;
        let mut id = HalfEdgeID::null();
        while id.is_null() {
            if i >= no_halfedges { return None; }
            id = halfedges[i].id.clone();
            i = i+1;
        }
        Some(id)
    }

    pub fn halfedge_next_iter(&self, index: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        let halfedges = RefCell::borrow(&self.halfedges);
        let no_halfedges = halfedges.len();
        let mut i = index.val() + 1;
        let mut id = HalfEdgeID::null();
        while id.is_null() {
            if i >= no_halfedges { return None; }
            id = halfedges[i].id.clone();
            i = i+1;
        }
        Some(id)
    }

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> HalfEdgeID
    {
        RefCell::borrow(&self.vertices)[vertex_id.val()].halfedge.clone()
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

#[derive(Debug)]
pub struct Vertex {
    pub halfedge: HalfEdgeID
}

impl Clone for Vertex {
  fn clone(& self) -> Self {
    Vertex { halfedge: self.halfedge.clone() }
  }
}

#[derive(Debug)]
pub struct HalfEdge {
    pub id: HalfEdgeID,
    pub vertex: VertexID,
    pub twin: HalfEdgeID,
    pub next: HalfEdgeID,
    pub face: FaceID
}

impl Clone for HalfEdge {
  fn clone(& self) -> Self {
    HalfEdge { id: self.id.clone(), vertex: self.vertex.clone(), twin: self.twin.clone(), next: self.next.clone(), face: self.face.clone() }
  }
}

#[derive(Debug)]
pub struct Face {
    pub halfedge: HalfEdgeID
}

impl Clone for Face {
  fn clone(& self) -> Self {
    Face { halfedge: self.halfedge.clone() }
  }
}