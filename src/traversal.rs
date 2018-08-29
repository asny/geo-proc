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
    pub fn new(vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> Walker
    {
        Walker { vertices, halfedges, faces }
    }

    pub fn vertex_at(&self, vertex_id: usize) -> Vertex
    {
        RefCell::borrow(&self.vertices)[vertex_id].clone()
    }

    pub fn halfedge_at(&self, halfedge_id: usize) -> HalfEdge
    {
        RefCell::borrow(&self.halfedges)[halfedge_id].clone()
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
    current: Vertex
}

impl VertexWalker
{
    pub fn new(vertex_id: usize, vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> VertexWalker
    {
        let walker = Walker::new(vertices, halfedges, faces);
        let current = walker.vertex_at(vertex_id);
        VertexWalker {current, walker}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        let halfedge = self.current.halfedge;
        HalfEdgeWalker { current: self.walker.halfedge_at(halfedge), walker: self.walker.clone() }
    }

    pub fn deref(&self) -> Vertex
    {
        self.current.clone()
    }
}

pub struct HalfEdgeWalker
{
    walker: Walker,
    current: HalfEdge
}

impl HalfEdgeWalker
{
    pub fn new(halfedge: &HalfEdge, vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> HalfEdgeWalker
    {
        HalfEdgeWalker {current: halfedge.clone(), walker: Walker::new(vertices, halfedges, faces)}
    }

    pub fn vertex(&mut self) -> VertexWalker
    {
        let vertex = self.current.vertex;
        VertexWalker { current: self.walker.vertex_at(vertex), walker: self.walker.clone() }
    }

    pub fn deref(&self) -> HalfEdge
    {
        self.current.clone()
    }
}

/*impl VertexWalker for Walker {
    fn halfedge(&mut self) -> Walker
    {
        let vertex = &RefCell::borrow(&self.vertices)[self.current];
        self.current = vertex.halfedge;
        self.clone()
    }

}

impl HalfEdgeWalker for Walker {
    fn vertex(&mut self) -> Box<VertexWalker>
    {
        let halfedge = &RefCell::borrow(&self.halfedges)[self.current];
        self.current = halfedge.vertex;
        Box::new(self.clone())
    }

    fn deref(&self) -> HalfEdge
    {
        RefCell::borrow(&self.halfedges)[self.current].clone()
    }

}*/

#[derive(Debug)]
pub struct Vertex {
    pub id: usize,
    pub halfedge: usize
}

impl Clone for Vertex {
  fn clone(& self) -> Self {
    Vertex { id: self.id, halfedge: self.halfedge }
  }
}

/*impl Vertex
{
    pub fn create(id: usize) -> Vertex
    {
        Vertex {id, halfedge: 0}
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn halfedge(&self) -> usize
    {
        self.halfedge
    }

    pub fn attach_halfedge(&mut self, halfedge: usize)
    {
        self.halfedge = halfedge;
    }

    pub fn detach_halfedge(&mut self)
    {
        self.halfedge = 0;
    }
}*/

#[derive(Debug)]
pub struct HalfEdge {
    pub id: usize,
    pub vertex: usize
}

impl Clone for HalfEdge {
  fn clone(& self) -> Self {
    HalfEdge { id: self.id, vertex: self.vertex }
  }
}

/*impl HalfEdge
{
    pub fn create(id: usize, vertex: usize) -> HalfEdge
    {
        HalfEdge {id, vertex}
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn vertex(&self) -> usize
    {
        self.vertex
    }
}*/

#[derive(Debug)]
pub struct Face {
    pub id: usize,
    pub halfedge: usize
}

impl Clone for Face {
  fn clone(& self) -> Self {
    Face { id: self.id, halfedge: self.halfedge }
  }
}

/*impl Face
{
    pub fn create(id: usize, halfedge: usize) -> Face
    {
        Face {id, halfedge}
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn halfedge(&self) -> usize
    {
        self.halfedge
    }

    pub fn attach_halfedge(&mut self, halfedge: usize)
    {
        self.halfedge = halfedge;
    }
}*/