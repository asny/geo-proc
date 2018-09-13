
use attribute::VertexAttributes;
use glm::*;
use ids::*;
use mesh::{self, Error, Renderable};

pub struct StaticMesh
{
    indices: Vec<u32>,
    no_vertices: usize,
    attributes: VertexAttributes
}

// TODO: Remove setters and initialize from constructor
// TODO: Contain all attributes in vec<f32>
impl StaticMesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>) -> Result<StaticMesh, Error>
    {
        let mut mesh = StaticMesh { indices, no_vertices: positions.len()/3, attributes: VertexAttributes::new() };
        mesh.add_vec3_attribute("position", positions)?;
        Ok(mesh)
    }

    pub fn add_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        let no_vertices = self.no_vertices();
        if no_vertices != data.len()/2 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }

        let mut i = 0;
        for vertex_id in self.vertex_iterator() {
            self.attributes.set_vec2_attribute_at(name, &vertex_id, &vec2(data[i], data[i + 1]));
            i = i+2;
        }
        
        Ok(())
    }

    pub fn add_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        let no_vertices = self.no_vertices();
        if no_vertices != data.len()/3 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }

        let mut i = 0;
        for vertex_id in self.vertex_iterator() {
            let value = vec3(data[i], data[i + 1], data[i + 2]);
            self.attributes.set_vec3_attribute_at(name, &vertex_id, &value);
            i = i+3;
        }

        Ok(())
    }

    pub fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>
    {
        self.attributes.set_vec2_attribute_at(name, vertex_id, value);
        Ok(())
    }

    pub fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>
    {
        self.attributes.set_vec3_attribute_at(name, vertex_id, value);
        Ok(())
    }

    fn no_vertices(&self) -> usize
    {
        self.no_vertices
    }

    fn no_faces(&self) -> usize
    {
        self.indices.len()/3
    }
}

impl Renderable for StaticMesh
{
    fn indices(&self) -> Vec<u32>
    {
        self.indices.clone()
    }

    fn vertex_iterator(&self) -> mesh::VertexIterator
    {
        VertexIterator::new(self.no_vertices())
    }

    fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, Error>
    {
        let val = self.attributes.get_vec2_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>
    {
        let val = self.attributes.get_vec3_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn no_vertices(&self) -> usize
    {
        self.no_vertices()
    }
}

struct VertexIterator
{
    current: usize,
    no_vertices: usize
}

impl VertexIterator {
    pub fn new(no_vertices: usize) -> mesh::VertexIterator
    {
        Box::new(VertexIterator {current: 0, no_vertices})
    }
}

impl Iterator for VertexIterator {
    type Item = VertexID;

    fn next(&mut self) -> Option<VertexID>
    {
        if self.current == self.no_vertices { return None }
        self.current = self.current + 1;
        Some(VertexID::new(self.current - 1))
    }
}