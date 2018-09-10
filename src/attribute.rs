use glm::*;
use ids::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    WrongSizeOfAttribute {message: String}
}

pub struct VertexAttributes
{
    vec2_attributes: Vec<Vec2Attribute>,
    vec3_attributes: Vec<Vec3Attribute>
}

impl VertexAttributes {
    pub fn new(positions: Vec<f32>) -> VertexAttributes
    {
        let mut vec3_attributes = Vec::new();
        vec3_attributes.push(Vec3Attribute::create("position", positions));
        VertexAttributes {vec2_attributes: Vec::new(), vec3_attributes}
    }

    pub fn add_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices() != data.len()/2 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }
        let custom_attribute = Vec2Attribute::create(name, data);
        self.vec2_attributes.push(custom_attribute);
        Ok(())
    }

    pub fn add_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices() != data.len()/3 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices())})
        }
        let custom_attribute = Vec3Attribute::create(name, data);
        self.vec3_attributes.push(custom_attribute);
        Ok(())
    }

    pub fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec2, Error>
    {
        for attribute in self.vec2_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute.at(vertex_id))
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>
    {
        for attribute in self.vec2_attributes.iter_mut() {
            if attribute.name() == name
            {
                attribute.set(&vertex_id, &value);
                return Ok(())
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec3, Error>
    {
        for attribute in self.vec3_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute.at(vertex_id))
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>
    {
        for attribute in self.vec3_attributes.iter_mut() {
            if attribute.name() == name
            {
                attribute.set(&vertex_id, &value);
                return Ok(())
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn position_at(&self, vertex_id: &VertexID) -> Vec3
    {
        self.vec3_attributes.first().unwrap().at(vertex_id)
    }

    pub fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.vec3_attributes.first_mut().unwrap().set(vertex_id, value);
    }

    fn no_vertices(&self) -> usize
    {
        self.vec3_attributes.first().unwrap().len()/3
    }
}

pub struct IntAttribute {
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

    pub fn len(&self) -> usize { self.data.len() }
}

pub struct Vec2Attribute {
    name: String,
    data: Vec<f32>
}


impl Vec2Attribute
{
    pub fn create(name: &str, data: Vec<f32>) -> Vec2Attribute
    {
        Vec2Attribute{name: String::from(name), data}
    }

    pub fn at(&self, vertex_id: &VertexID) -> Vec2
    {
        vec2(self.data[vertex_id.val() * 2], self.data[vertex_id.val() * 2 + 1])
    }

    pub fn set(&mut self, vertex_id: &VertexID, value: &Vec2)
    {
        self.data[vertex_id.val() * 2] = value[0];
        self.data[vertex_id.val() * 2 + 1] = value[1];
    }

    pub fn name(&self) -> &str
    {
        self.name.as_ref()
    }

    pub fn len(&self) -> usize { self.data.len() }
}

pub struct Vec3Attribute {
    name: String,
    data: Vec<f32>
}


impl Vec3Attribute
{
    pub fn create(name: &str, data: Vec<f32>) -> Vec3Attribute
    {
        Vec3Attribute{name: String::from(name), data}
    }

    pub fn at(&self, vertex_id: &VertexID) -> Vec3
    {
        vec3(self.data[vertex_id.val() * 3], self.data[vertex_id.val() * 3 + 1], self.data[vertex_id.val() * 3 + 2])
    }

    pub fn set(&mut self, vertex_id: &VertexID, value: &Vec3) {
        self.data[vertex_id.val() * 3] = value[0];
        self.data[vertex_id.val() * 3 + 1] = value[1];
        self.data[vertex_id.val() * 3 + 2] = value[2];
    }

    pub fn name(&self) -> &str
    {
        self.name.as_ref()
    }

    pub fn len(&self) -> usize { self.data.len() }
}