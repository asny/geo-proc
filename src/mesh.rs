use attribute;
use glm::*;
use std::string::String;
use ids::*;
use iterators::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    WrongSizeOfAttribute {message: String}
}

pub trait Drawable
{
    fn indices(&self) -> &Vec<u32>;

    fn vertex_iterator(&self) -> VertexIterator;

    fn no_vertices(&self) -> usize;
    fn no_faces(&self) -> usize;
}

pub struct Mesh
{
    indices: Vec<u32>,
    vec2_attributes: Vec<attribute::Vec2Attribute>,
    vec3_attributes: Vec<attribute::Vec3Attribute>
}

impl Mesh
{
    pub fn create(positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let indices = (0..positions.len() as u32/3).collect();
        Mesh::create_indexed(indices, positions)
    }

    pub fn create_indexed(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let mut mesh = Mesh { indices, vec2_attributes: Vec::new(), vec3_attributes: Vec::new() };
        mesh.vec3_attributes.push(attribute::Vec3Attribute::create("position", positions));
        Ok(mesh)
    }

    pub fn add_custom_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices() != data.len()/2 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }
        let custom_attribute = attribute::Vec2Attribute::create(name, data);
        self.vec2_attributes.push(custom_attribute);
        Ok(())
    }

    pub fn add_custom_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices() != data.len()/3 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }
        let custom_attribute = attribute::Vec3Attribute::create(name, data);
        self.vec3_attributes.push(custom_attribute);
        Ok(())
    }
}

impl Drawable for Mesh
{
    fn no_vertices(&self) -> usize
    {
        self.vec3_attributes.first().unwrap().len()/3
    }

    fn no_faces(&self) -> usize
    {
        self.indices.len()/3
    }

    fn indices(&self) -> &Vec<u32>
    {
        &self.indices
    }

    fn vertex_iterator(&self) -> VertexIterator
    {
        StaticVertexIterator::new(self.no_vertices())
    }
}

pub struct StaticVertexIterator
{
    current: usize,
    no_vertices: usize
}

impl StaticVertexIterator {
    pub fn new(no_vertices: usize) -> VertexIterator
    {
        Box::new(StaticVertexIterator {current: 0, no_vertices})
    }
}

impl Iterator for StaticVertexIterator {
    type Item = VertexID;

    fn next(&mut self) -> Option<VertexID>
    {
        if self.current == self.no_vertices { return None }
        self.current = self.current + 1;
        Some(VertexID::new(self.current - 1))
    }
}