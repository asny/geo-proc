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
        let halfedge = self.walker.vertex_at(&self.current).halfedge.clone();
        HalfEdgeWalker { current: halfedge, walker: self.walker.clone() }
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
        let vertex = self.walker.halfedge_at(&self.current).vertex.clone();
        VertexWalker { current: vertex, walker: self.walker.clone() }
    }

    pub fn deref(&self) -> HalfEdgeID
    {
        self.current.clone()
    }
}

#[derive(Debug)]
pub struct Vertex {
    pub id: VertexID,
    pub halfedge: HalfEdgeID
}

impl Clone for Vertex {
  fn clone(& self) -> Self {
    Vertex { id: self.id.clone(), halfedge: self.halfedge.clone() }
  }
}

#[derive(Debug)]
pub struct HalfEdge {
    pub id: HalfEdgeID,
    pub vertex: VertexID
}

impl Clone for HalfEdge {
  fn clone(& self) -> Self {
    HalfEdge { id: self.id.clone(), vertex: self.vertex.clone() }
  }
}

#[derive(Debug)]
pub struct Face {
    pub id: FaceID,
    pub halfedge: HalfEdgeID
}

impl Clone for Face {
  fn clone(& self) -> Self {
    Face { id: self.id.clone(), halfedge: self.halfedge.clone() }
  }
}

#[derive(Debug)]
pub struct VertexID
{
    val: usize
}

impl VertexID {
    pub fn new(val: usize) -> VertexID
    {
        VertexID {val}
    }

    pub fn val(&self) -> usize
    {
        self.val
    }
}

impl Clone for VertexID {
  fn clone(& self) -> Self {
    VertexID { val: self.val }
  }
}

#[derive(Debug)]
pub struct HalfEdgeID
{
    val: usize
}

impl HalfEdgeID {
    pub fn new(val: usize) -> HalfEdgeID
    {
        HalfEdgeID {val}
    }

    pub fn val(&self) -> usize
    {
        self.val
    }
}

impl Clone for HalfEdgeID {
  fn clone(& self) -> Self {
    HalfEdgeID { val: self.val }
  }
}

#[derive(Debug)]
pub struct FaceID
{
    val: usize
}

impl FaceID {
    pub fn new(val: usize) -> FaceID
    {
        FaceID {val}
    }

    pub fn val(&self) -> usize
    {
        self.val
    }
}

impl Clone for FaceID {
  fn clone(& self) -> Self {
    FaceID { val: self.val }
  }
}