use ids::*;
use mesh::{self, Attribute, Error, Renderable};

pub struct StaticMesh
{
    indices: Vec<u32>,
    attributes: Vec<Attribute>
}

// TODO: Remove setters and initialize from constructor
// TODO: Contain all attributes in vec<f32>
impl StaticMesh
{
    pub fn create(indices: Vec<u32>, attributes: Vec<Attribute>) -> Result<StaticMesh, Error>
    {
        if attributes.len() == 0 {
            return Err(Error::NeedPositionAttributeToCreateMesh {message: format!("Need at least the position attribute to create a mesh.")})
        }
        Ok(StaticMesh { indices, attributes })
    }

    fn no_vertices(&self) -> usize
    {
        let att = self.attributes.first().unwrap();
        att.data.len()/att.no_components
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

    fn get_attribute(&self, name: &str) -> Option<&Attribute>
    {
        self.attributes.iter().find(|att| att.name == name)
    }

    fn vertex_iterator(&self) -> mesh::VertexIterator
    {
        VertexIterator::new(self.no_vertices())
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