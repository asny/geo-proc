use std::rc::{Rc};
use std::cell::{RefCell};

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
        halfedges.push(HalfEdge { vertex: VertexID::null(), twin: HalfEdgeID::null(), next: HalfEdgeID::null(), face: FaceID::null() });
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

    fn vertex_halfedge(&self, vertex_id: &VertexID) -> HalfEdgeID
    {
        RefCell::borrow(&self.vertices)[vertex_id.val()].halfedge.clone()
    }

    fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> VertexID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].vertex.clone()
    }

    fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].twin.clone()
    }

    fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].next.clone()
    }

    fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> FaceID
    {
        RefCell::borrow(&self.halfedges)[halfedge_id.val()].face.clone()
    }

    fn face_halfedge(&self, face_id: &FaceID) -> HalfEdgeID
    {
        RefCell::borrow(&self.faces)[face_id.val()].halfedge.clone()
    }
}

pub struct VertexWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: VertexID
}

impl VertexWalker
{
    pub fn new(current: VertexID, connectivity_info: Rc<ConnectivityInfo>) -> VertexWalker
    {
        VertexWalker {current, connectivity_info}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.vertex_halfedge(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn deref(&self) -> VertexID
    {
        self.current.clone()
    }

    pub fn deref_option(&self) -> Option<VertexID>
    {
        if self.current.is_null()
        {
            return None
        }
        Some(self.current.clone())
    }
}

pub struct HalfEdgeWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID
}

impl HalfEdgeWalker
{
    pub fn new(current: HalfEdgeID, connectivity_info: Rc<ConnectivityInfo>) -> HalfEdgeWalker
    {
        HalfEdgeWalker {current, connectivity_info}
    }

    pub fn vertex(&mut self) -> VertexWalker
    {
        if self.current.is_null()
        {
            return VertexWalker { current: VertexID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_vertex(&self.current);
        VertexWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn twin(&mut self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_twin(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn next(&mut self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_next(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn previous(&mut self) -> HalfEdgeWalker
    {
        self.next().next()
    }

    pub fn face(&mut self) -> FaceWalker
    {
        if self.current.is_null()
        {
            return FaceWalker { current: FaceID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_face(&self.current);
        FaceWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn deref(&self) -> HalfEdgeID
    {
        self.current.clone()
    }

    pub fn deref_option(&self) -> Option<HalfEdgeID>
    {
        if self.current.is_null()
        {
            return None
        }
        Some(self.current.clone())
    }
}

pub struct FaceWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: FaceID
}

impl FaceWalker
{
    pub fn new(current: FaceID, connectivity_info: Rc<ConnectivityInfo>) -> FaceWalker
    {
        FaceWalker {current, connectivity_info}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.face_halfedge(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn deref(&self) -> FaceID
    {
        self.current.clone()
    }

    pub fn deref_option(&self) -> Option<FaceID>
    {
        if self.current.is_null()
        {
            return None
        }
        Some(self.current.clone())
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

impl PartialEq for VertexID {
    fn eq(&self, other: &VertexID) -> bool {
        !self.is_null() && !other.is_null() && self.val == other.val
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

impl PartialEq for HalfEdgeID {
    fn eq(&self, other: &HalfEdgeID) -> bool {
        !self.is_null() && !other.is_null() && self.val == other.val
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

impl PartialEq for FaceID {
    fn eq(&self, other: &FaceID) -> bool {
        !self.is_null() && !other.is_null() && self.val == other.val
    }
}