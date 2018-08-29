use std::rc::{Weak, Rc};
use std::cell::{RefCell, Ref};
use std::ops::{Deref,DerefMut};
use std::borrow::{Borrow, BorrowMut};

pub struct Walker {
    current: usize,
    vertices: Rc<RefCell<Vec<Vertex>>>,
    halfedges: Rc<RefCell<Vec<HalfEdge>>>,
    faces: Rc<RefCell<Vec<Face>>>
}

impl Walker {
    pub fn new(vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> Walker
    {
        Walker { current: 0, vertices, halfedges, faces }
    }

    pub fn new_vertex_walker(id: usize, vertices: Rc<RefCell<Vec<Vertex>>>, halfedges: Rc<RefCell<Vec<HalfEdge>>>, faces: Rc<RefCell<Vec<Face>>>) -> Walker
    {
        let walker = Walker { current: id, vertices, halfedges, faces };

        walker
    }

    pub fn next_vertex(&self) -> &Self
    {

        &self
    }

    pub fn halfedge(&mut self) -> HalfEdgeWalker
    {
        let halfedge = self.current_vertex().halfedge;
        self.current = halfedge;
        HalfEdgeWalker { walker: self.clone() }
    }

    pub fn vertex(&mut self) -> VertexWalker
    {
        let vertex = self.current_halfedge().vertex;
        self.current = vertex;
        VertexWalker { walker: self.clone() }
    }

    pub fn current_vertex(&self) -> Vertex
    {
        RefCell::borrow(&self.vertices)[self.current].clone()
    }

    pub fn current_halfedge(&self) -> HalfEdge
    {
        RefCell::borrow(&self.halfedges)[self.current].clone()
    }

}

impl Clone for Walker {
  fn clone(& self) -> Self {
    Walker { current: self.current, vertices: self.vertices.clone(), halfedges: self.halfedges.clone(), faces: self.faces.clone() }
  }
}

pub struct VertexWalker
{
    walker: Walker
}

impl VertexWalker {
    pub fn halfedge(&mut self) -> HalfEdgeWalker
    {
        self.walker.halfedge()
    }

    pub fn deref(&self) -> Vertex
    {
        self.walker.current_vertex()
    }
}

pub struct HalfEdgeWalker
{
    walker: Walker
}

impl HalfEdgeWalker {
    pub fn vertex(&mut self) -> VertexWalker
    {
        self.walker.vertex()
    }

    pub fn deref(&self) -> HalfEdge
    {
        self.walker.current_halfedge()
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