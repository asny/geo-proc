use glm::*;
use ids::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String}
}

pub struct VertexAttributes
{
    vec2_attributes: Vec<Vec2Attribute>,
    vec3_attributes: Vec<Vec3Attribute>
}

impl VertexAttributes {
    pub fn new() -> VertexAttributes
    {
        VertexAttributes {vec2_attributes: Vec::new(), vec3_attributes: Vec::new()}
    }

    pub fn create_vec2_attribute(&mut self, name: &str)
    {
        self.vec2_attributes.push(Vec2Attribute{ name: String::from(name), data: Vec::new() })
    }

    pub fn create_vec3_attribute(&mut self, name: &str, initial_size: usize)
    {
        self.vec3_attributes.push(Vec3Attribute{ name: String::from(name), data: vec![vec3(0.0, 0.0, 0.0); initial_size] })
    }

    pub fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, Error>
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

    pub fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>
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

    pub fn position_at(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.vec3_attributes.first().unwrap().at(vertex_id)
    }

    pub fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.vec3_attributes.first_mut().unwrap().set(vertex_id, value);
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
    name: String,
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

    pub fn name(&self) -> &str
    {
        self.name.as_ref()
    }
}

pub struct Vec3Attribute {
    name: String,
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

    pub fn name(&self) -> &str
    {
        self.name.as_ref()
    }
}