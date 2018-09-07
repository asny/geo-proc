use glm::*;
use ids::*;

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
}