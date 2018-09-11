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

    pub fn create_vec2_attribute(&mut self, name: &str, initial_size: usize)
    {
        let att = Vec2Attribute{ data: vec![vec2(0.0, 0.0); initial_size] };
        self.vec2_attributes.insert(String::from(name), att);
    }

    pub fn create_vec3_attribute(&mut self, name: &str, initial_size: usize)
    {
        let att = Vec3Attribute{ data: vec![vec3(0.0, 0.0, 0.0); initial_size] };
        self.vec3_attributes.insert(String::from(name), att);
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

    pub fn position_at(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.get_vec3_attribute_at("position", vertex_id).unwrap()
    }

    pub fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.set_vec3_attribute_at("position", vertex_id, value).unwrap()
    }
}

/*pub struct IntAttribute {
    name: String,
    data: Vec<f32>
}

impl IntAttribute
{
    pub fn create(name: &str, data: &Vec<u32>) -> IntAttribute
    {
        let d = data.iter().map(|i| *i as f32).collect();
        IntAttribute{name: String::from(name), data: d}
    }

    pub fn at(&self, vertex_id: &VertexID) -> u32
    {
        self.data[vertex_id.val()] as u32
    }

    pub fn set(&mut self, vertex_id: &VertexID, value: u32) {
        self.data[vertex_id.val()] = value as f32;
    }

    pub fn name(&self) -> &str
    {
        self.name.as_ref()
    }
}*/

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