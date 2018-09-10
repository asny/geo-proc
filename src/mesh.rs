use attribute;
use glm::*;
use std::string::String;
use ids::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    WrongSizeOfAttribute {message: String}
}

pub trait Mesh
{
    fn indices(&self) -> &Vec<u32>;

    fn vertex_iterator(&self) -> VertexIterator;

    fn no_vertices(&self) -> usize;
    fn no_faces(&self) -> usize;
}

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
