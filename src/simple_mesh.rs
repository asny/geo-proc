
use attribute::VertexAttributes;
use glm::*;
use std::string::String;
use ids::*;
use mesh;

pub struct SimpleMesh
{
    indices: Vec<u32>,
    no_vertices: usize,
    attributes: VertexAttributes
}

impl SimpleMesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>) -> Result<SimpleMesh, mesh::Error>
    {
        Ok(SimpleMesh { indices, no_vertices: positions.len()/3, attributes: VertexAttributes::new(positions) })
    }
}

impl mesh::Mesh for SimpleMesh
{
    fn no_vertices(&self) -> usize
    {
        self.no_vertices
    }

    fn no_faces(&self) -> usize
    {
        self.indices.len()/3
    }

    fn indices(&self) -> &Vec<u32>
    {
        &self.indices
    }

    fn vertex_iterator(&self) -> mesh::VertexIterator
    {
        VertexIterator::new(self.no_vertices())
    }

    fn add_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), mesh::Error>
    {
        self.attributes.add_vec2_attribute(name, data)?;
        Ok(())
    }

    fn add_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), mesh::Error>
    {
        self.attributes.add_vec3_attribute(name, data)?;
        Ok(())
    }

    fn position_at(&self, vertex_id: &VertexID) -> Vec3
    {
        self.attributes.position_at(vertex_id)
    }

    fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.attributes.set_position_at(vertex_id, value);
    }

    fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec2, mesh::Error>
    {
        let val = self.attributes.get_vec2_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), mesh::Error>
    {
        self.attributes.set_vec2_attribute_at(name, vertex_id, value)?;
        Ok(())
    }

    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec3, mesh::Error>
    {
        let val = self.attributes.get_vec3_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), mesh::Error>
    {
        self.attributes.set_vec3_attribute_at(name, vertex_id, value)?;
        Ok(())
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