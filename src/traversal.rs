use std::rc::{Weak, Rc};
use std::cell::{RefCell, Ref};
use std::ops::{Deref,DerefMut};
use std::borrow::{Borrow, BorrowMut};

struct Walker {
    vertices: Rc<RefCell<Vec<Vertex>>>,
    halfedges: Rc<RefCell<Vec<HalfEdge>>>,
    faces: Rc<RefCell<Vec<Face>>>
}

impl Walker {
    fn new(vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> Walker
    {
        Walker { vertices, halfedges, faces }
    }

    fn vertex_at(&self, vertex_id: &VertexID) -> Vertex
    {
        RefCell::borrow(&self.vertices)[vertex_id.val()].clone()
    }

    fn halfedge_at(&self, halfedge_id: &HalfEdgeID) -> HalfEdge
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].clone()
    }

    fn face_at(&self, face_id: usize) -> Face
    {
        RefCell::borrow(&self.faces)[face_id].clone()
    }
}

impl Clone for Walker {
  fn clone(& self) -> Self {
    Walker { vertices: self.vertices.clone(), halfedges: self.halfedges.clone(), faces: self.faces.clone() }
  }
}

pub struct VertexWalker
{
    walker: Walker,
    current: VertexID
}

impl VertexWalker
{
    pub fn new(vertex_id: &VertexID, vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> VertexWalker
    {
        let walker = Walker::new(vertices, halfedges, faces);
        VertexWalker {current: vertex_id.clone(), walker}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        let id = self.walker.vertex_at(&self.current).halfedge.clone();
        HalfEdgeWalker { current: id, walker: self.walker.clone() }
    }

    pub fn deref(&self) -> VertexID
    {
        self.current.clone()
    }
}

pub struct HalfEdgeWalker
{
    walker: Walker,
    current: HalfEdgeID
}

impl HalfEdgeWalker
{
    pub fn new(halfedge_id: &HalfEdgeID, vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> HalfEdgeWalker
    {
        let walker = Walker::new(vertices, halfedges, faces);
        HalfEdgeWalker {current: halfedge_id.clone(), walker}
    }

    pub fn vertex(&mut self) -> VertexWalker
    {
        let id = self.walker.halfedge_at(&self.current).vertex.clone();
        VertexWalker { current: id, walker: self.walker.clone() }
    }

    pub fn twin(&mut self) -> HalfEdgeWalker
    {
        let id = self.walker.halfedge_at(&self.current).twin.clone();
        HalfEdgeWalker { current: id, walker: self.walker.clone() }
    }

    pub fn next(&mut self) -> HalfEdgeWalker
    {
        let id = self.walker.halfedge_at(&self.current).next.clone();
        HalfEdgeWalker { current: id, walker: self.walker.clone() }
    }

    pub fn deref(&self) -> HalfEdgeID
    {
        self.current.clone()
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
    pub vertex: VertexID,
    pub twin: HalfEdgeID,
    pub next: HalfEdgeID,
    pub face: FaceID
}

impl Clone for HalfEdge {
  fn clone(& self) -> Self {
    HalfEdge { vertex: self.vertex.clone(), twin: self.twin.clone(), next: self.next.clone(), face: self.face.clone() }
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

#[derive(Debug)]
pub struct VertexID
{
    val: usize,
    dead: bool
}

impl VertexID {
    pub fn new(val: usize) -> VertexID
    {
        VertexID {val, dead: false}
    }

    pub fn null() -> VertexID
    {
        VertexID {val: 0, dead: true}
    }

    pub fn is_null(&self) -> bool
    {
        self.dead
    }

    pub fn val(&self) -> usize
    {
        if self.is_null() {
            panic!("Vertex is dead");
        }
        self.val
    }
}

impl Clone for VertexID {
  fn clone(& self) -> Self {
    VertexID { val: self.val, dead: self.dead }
  }
}

#[derive(Debug)]
pub struct HalfEdgeID
{
    val: usize,
    dead: bool
}

impl HalfEdgeID {
    pub fn new(val: usize) -> HalfEdgeID
    {
        HalfEdgeID {val, dead: false}
    }

    pub fn null() -> HalfEdgeID
    {
        HalfEdgeID {val: 0, dead: true}
    }

    pub fn is_null(&self) -> bool
    {
        self.dead
    }

    pub fn val(&self) -> usize
    {
        if self.is_null() {
            panic!("Halfedge is dead");
        }
        self.val
    }
}

impl Clone for HalfEdgeID {
  fn clone(& self) -> Self {
    HalfEdgeID { val: self.val, dead: self.dead }
  }
}

#[derive(Debug)]
pub struct FaceID
{
    val: usize,
    dead: bool
}

impl FaceID {
    pub fn new(val: usize) -> FaceID
    {
        FaceID {val, dead: false}
    }

    pub fn null() -> FaceID
    {
        FaceID {val: 0, dead: true}
    }

    pub fn is_null(&self) -> bool
    {
        self.dead
    }

    pub fn val(&self) -> usize
    {
        if self.is_null() {
            panic!("Face is dead");
        }
        self.val
    }
}

impl Clone for FaceID {
  fn clone(& self) -> Self {
    FaceID { val: self.val, dead: self.dead }
  }
}