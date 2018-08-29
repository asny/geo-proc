use std::rc::{Weak, Rc};
use std::cell::{RefCell, Ref};
use std::ops::{Deref,DerefMut};
use std::borrow::{Borrow, BorrowMut};

#[derive(Debug)]
pub enum Error {
    FailedToDerefPtr {message: String}
}

#[derive(Debug)]
pub struct Ptr<T> {
  pub val: Rc<RefCell<T>>
}

impl<T> Ptr<T> {
    pub fn new(val: T) -> Ptr<T> {
        Ptr { val: Rc::new(RefCell::new(val)) }
    }

    pub fn deref(&self) -> Ref<T>
    {
        RefCell::borrow(self.val.as_ref())
    }
}

impl<T> Deref for Ptr<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &RefCell<T> {
        println!("deref");
        self.val.as_ref()
    }
}

/*impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut T {
        println!("deref mut");
        RefCell::borrow_mut(self.val.as_ref()).deref_mut()
    }
}*/

impl<T> Clone for Ptr<T> {
  fn clone(& self) -> Self {
    Ptr { val: self.val.clone() }
  }
}

#[derive(Debug)]
pub struct VertexPtr {
  pub val: Rc<RefCell<Vertex>>
}

impl VertexPtr {
    pub fn new(val: T) -> Ptr<T> {
        Ptr { val: Rc::new(RefCell::new(val)) }
    }

    pub fn halfedge(&self) -> Ref<T>
    {
        RefCell::borrow(self.val.as_ref())
    }
}

pub struct Vertex {
    id: usize,
    halfedge: Option<Ptr<HalfEdge>>
}

impl Vertex
{
    pub fn create(id: usize) -> Ptr<Vertex>
    {
        Ptr::new(Vertex {id, halfedge: None})
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn halfedge(&self) -> &Ptr<HalfEdge>
    {
        &self.halfedge.as_ref().unwrap()
    }

    pub fn attach_halfedge(&mut self, halfedge: &Ptr<HalfEdge>)
    {
        self.halfedge = Some(halfedge.clone());
    }

    pub fn detach_halfedge(&mut self)
    {
        self.halfedge = None;
    }
}


pub struct HalfEdge {
    id: usize,
    vertex: Ptr<Vertex>
}

impl HalfEdge
{
    pub fn create(id: usize, vertex: &Ptr<Vertex>) -> Ptr<HalfEdge>
    {
        Ptr::new(HalfEdge {id, vertex: vertex.clone()})
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn vertex(&self) -> &Ptr<Vertex>
    {
        &self.vertex
    }
}

pub struct Face {
    id: usize,
    halfedge: Ptr<HalfEdge>
}

impl Face
{
    pub fn create(id: usize, halfedge: &Ptr<HalfEdge>) -> Ptr<Face>
    {
        Ptr::new(Face {id, halfedge: halfedge.clone()})
    }

    pub fn id(&self) -> usize
    {
        self.id
    }

    pub fn halfedge(&self) -> &Ptr<HalfEdge>
    {
        &self.halfedge
    }

    pub fn attach_halfedge(&mut self, halfedge: &Ptr<HalfEdge>)
    {
        self.halfedge = halfedge.clone();
    }
}