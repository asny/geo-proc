use attribute;
use glm::*;
use std::string::String;
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

    fn add_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>;
    fn add_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>;

    fn position_at(&self, vertex_id: &VertexID) -> Vec3;
    fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3);

    fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec2, Error>;
    fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>;

    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec3, Error>;
    fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>;
}

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
