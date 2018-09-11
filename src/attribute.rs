use glm::*;
use ids::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    FailedToFindEntryForVertexID {message: String}
}

#[derive(Clone, Debug)]
pub struct VertexAttributes
{
    vec2_attributes: HashMap<String, HashMap<VertexID, Vec2>>,
    vec3_attributes: HashMap<String, HashMap<VertexID, Vec3>>
}

impl VertexAttributes {
    pub fn new() -> VertexAttributes
    {
        VertexAttributes {vec2_attributes: HashMap::new(), vec3_attributes: HashMap::new()}
    }

    // Vec2 attribute
    pub fn create_vec2_attribute(&mut self, name: &str)
    {
        self.vec2_attributes.insert(String::from(name), HashMap::new());
    }

    pub fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, Error>
    {
        match self.vec2_attributes.get(name)
        {
            Some(ref att) => att.get(vertex_id)
                .ok_or(Error::FailedToFindEntryForVertexID{message: format!("Failed to find entry for {} attribute and {} vertex id", name, vertex_id)}),
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    pub fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>
    {
        match self.vec2_attributes.get_mut(name)
        {
            Some(ref mut att) => {att.insert(vertex_id.clone(), *value); Ok(())},
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    // Vec3 attribute
    pub fn create_vec3_attribute(&mut self, name: &str)
    {
        self.vec3_attributes.insert(String::from(name), HashMap::new());
    }

    pub fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>
    {
        match self.vec3_attributes.get(name)
        {
            Some(ref att) => att.get(vertex_id)
                .ok_or(Error::FailedToFindEntryForVertexID{message: format!("Failed to find entry for {} attribute and {} vertex id", name, vertex_id)}),
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    pub fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>
    {
        match self.vec3_attributes.get_mut(name)
        {
            Some(ref mut att) => {att.insert(vertex_id.clone(), *value); Ok(())},
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }
}