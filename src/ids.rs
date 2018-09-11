use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for VertexID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {}, dead: {}))", self.val, self.dead)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for HalfEdgeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {}, dead: {}))", self.val, self.dead)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for FaceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {}, dead: {}))", self.val, self.dead)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_equality()
    {
        let v0 = VertexID::new(0);
        let v1 = VertexID::new(1);
        let v1_ = VertexID::new(1);
        let vn1 = VertexID::null();
        let vn2 = VertexID::null();

        assert!(v0 != v1);
        assert!(v1 == v1_);
        assert!(v0 != vn1);
        assert!(vn1 == vn2);
    }
}