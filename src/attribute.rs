use glm::*;
use ids::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String}
}

pub struct VertexAttributes
{
    vec2_attributes: HashMap<String, Vec2Attribute>,
    vec3_attributes: HashMap<String, Vec3Attribute>
}

impl VertexAttributes {
    pub fn new() -> VertexAttributes
    {
        VertexAttributes {vec2_attributes: HashMap::new(), vec3_attributes: HashMap::new()}
    }

    // Vec2 attribute
    pub fn create_vec2_attribute(&mut self, name: &str, initial_size: usize)
    {
        let att = Vec2Attribute{ data: vec![vec2(0.0, 0.0); initial_size] };
        self.vec2_attributes.insert(String::from(name), att);
    }

    pub fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, Error>
    {
        match self.vec2_attributes.get(name)
        {
            Some(ref att) => Ok(att.at(vertex_id)),
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    pub fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>
    {
        match self.vec2_attributes.get_mut(name)
        {
            Some(ref mut att) => {att.set(&vertex_id, &value); Ok(())},
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    // Vec3 attribute
    pub fn create_vec3_attribute(&mut self, name: &str, initial_size: usize)
    {
        let att = Vec3Attribute{ data: vec![vec3(0.0, 0.0, 0.0); initial_size] };
        self.vec3_attributes.insert(String::from(name), att);
    }

    pub fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>
    {
        match self.vec3_attributes.get(name)
        {
            Some(ref att) => Ok(att.at(vertex_id)),
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }

    pub fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>
    {
        match self.vec3_attributes.get_mut(name)
        {
            Some(ref mut att) => {att.set(&vertex_id, &value); Ok(())},
            None => Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
        }
    }
}

pub struct Vec2Attribute {
    data: Vec<Vec2>
}

impl Vec2Attribute
{
    pub fn at(&self, vertex_id: &VertexID) -> &Vec2
    {
        &self.data[vertex_id.val()]
    }

    pub fn set(&mut self, vertex_id: &VertexID, value: &Vec2)
    {
        let id = vertex_id.val();
        if id >= self.data.len()
        {
            self.data.append(&mut vec![vec2(0.0, 0.0); 2*id+1])
        }
        self.data[id] = *value;
    }
}

pub struct Vec3Attribute {
    data: Vec<Vec3>
}


impl Vec3Attribute
{
    pub fn at(&self, vertex_id: &VertexID) -> &Vec3
    {
        &self.data[vertex_id.val()]
    }

    pub fn set(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        let id = vertex_id.val();
        if id >= self.data.len()
        {
            self.data.append(&mut vec![vec3(0.0, 0.0, 0.0); 2*id+1])
        }
        self.data[id] = *value;
    }
}