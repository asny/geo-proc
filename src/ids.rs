use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for VertexID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {})", self.val)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for HalfEdgeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {})", self.val)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

impl fmt::Display for FaceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {})", self.val)
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

        assert!(v0 != v1);
        assert!(v1 == v1_);
    }
}