use ids::*;
use mesh::{self, Attribute, Error, Renderable};

pub struct StaticMesh
{
    indices: Vec<u32>,
    attributes: Vec<Attribute>
}

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

    fn no_vertices(&self) -> usize
    {
        self.no_vertices()
    }
}