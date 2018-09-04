

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