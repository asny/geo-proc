use glm::*;
use ids::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    FailedToFindEntryForVertexID {message: String},
    WrongSizeOfAttribute {message: String}
}

pub trait Renderable
{
    fn indices(&self) -> &Vec<u32>;

    fn vertex_iterator(&self) -> VertexIterator;

    fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, Error>;
    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>;

    fn no_vertices(&self) -> usize;
}

pub type VertexIterator = Box<Iterator<Item = VertexID>>;