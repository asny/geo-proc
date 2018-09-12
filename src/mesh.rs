use attribute;
use glm::*;
use ids::*;

#[derive(Debug)]
pub enum Error {
    Attribute(attribute::Error)
}

impl From<attribute::Error> for Error {
    fn from(other: attribute::Error) -> Self {
        Error::Attribute(other)
    }
}

// Todo: Split in different traits
pub trait Mesh
{
    fn indices(&self) -> &Vec<u32>;

    fn vertex_iterator(&self) -> VertexIterator;

    fn no_vertices(&self) -> usize;
    fn no_faces(&self) -> usize;

    fn position_at(&self, vertex_id: &VertexID) -> &Vec3;
    fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3);
}

pub type VertexIterator = Box<Iterator<Item = VertexID>>;