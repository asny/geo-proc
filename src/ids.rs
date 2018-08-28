

pub struct VertexID {
    id: usize
}

impl VertexID
{
    pub fn next()
    {

    }

    pub fn value(&self) -> usize
    {
        self.id
    }
}