use attribute;
use std::string::String;
use glm;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    Attribute(attribute::Error)
}

impl From<attribute::Error> for Error {
    fn from(other: attribute::Error) -> Self {
        Error::Attribute(other)
    }
}

pub struct Mesh {
    no_vertices: usize,
    indices: Vec<u16>,
    positions: attribute::Attribute,
    custom_attributes: Vec<attribute::Attribute>
}


impl Mesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let mut positions_vec3 = Vec::with_capacity(no_vertices);
        for vid in 0..no_vertices {
            positions_vec3.push(glm::vec3(positions[vid * 3], positions[vid * 3 + 1], positions[vid * 3 + 2]));
        }
        let mut indices_u16 = Vec::with_capacity(indices.len());
        for i in 0..indices.len() {
            indices_u16.push(indices[i] as u16);
        }

        let position_attribute = attribute::Attribute::create_vec3_attribute("Position", positions_vec3)?;
        let mesh = Mesh { no_vertices, indices: indices_u16, positions: position_attribute, custom_attributes: Vec::new() };
        Ok(mesh)
    }

    pub fn positions(&self) -> &attribute::Attribute
    {
        &self.positions
    }

    pub fn indices(&self) -> &Vec<u16>
    {
        &self.indices
    }

    pub fn no_vertices(&self) -> usize
    {
        self.no_vertices
    }

    pub fn get(&self, name: &str) -> Result<&attribute::Attribute, Error>
    {
        for attribute in self.custom_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn add_custom_attribute(&mut self, name: &str, data: Vec<glm::Vec3>) -> Result<(), Error>
    {
        let custom_attribute = attribute::Attribute::create_vec3_attribute(name, data)?;
        self.custom_attributes.push(custom_attribute);
        Ok(())
    }
}
