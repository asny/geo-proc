use glm::*;
use ids::*;

#[derive(Debug)]
pub enum Error {

}

pub trait Attribute
{
    fn no_components(&self) -> usize;

    fn name(&self) -> &str;

    fn data(&self) -> &Vec<f32>;

    fn data_mut(&mut self) -> &mut Vec<f32>;
}

pub struct IntAttribute {
    name: String,
    data: Vec<f32>
}


impl IntAttribute
{
    pub fn create(name: &str, data: &Vec<u32>) -> Result<IntAttribute, Error>
    {
        let d = data.iter().map(|i| *i as f32).collect();
        Ok(IntAttribute{name: String::from(name), data: d})
    }

    pub fn at(&self, vertex_id: usize) -> u32
    {
        self.data[vertex_id] as u32
    }

    pub fn set(&mut self, vertex_id: usize, value: u32) {
        self.data[vertex_id] = value as f32;
    }
}

impl Attribute for IntAttribute
{
    fn no_components(&self) -> usize
    {
        1
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn data(&self) -> &Vec<f32>
    {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<f32>
    {
        &mut self.data
    }
}

pub struct Vec2Attribute {
    name: String,
    data: Vec<f32>
}


impl Vec2Attribute
{
    pub fn create(name: &str, data: Vec<f32>) -> Result<Vec2Attribute, Error>
    {
        Ok(Vec2Attribute{name: String::from(name), data})
    }

    pub fn at(&self, vertex_id: usize) -> Vec2
    {
        vec2(self.data[vertex_id * 2], self.data[vertex_id * 2 + 1])
    }

    pub fn set(&mut self, vertex_id: usize, value: Vec2) {
        let no_components = self.no_components();
        for i in 0..no_components {
            self.data[vertex_id * no_components] = value[i];
        }
    }
}

impl Attribute for Vec2Attribute
{
    fn no_components(&self) -> usize
    {
        2
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn data(&self) -> &Vec<f32>
    {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<f32>
    {
        &mut self.data
    }
}

pub struct Vec3Attribute {
    name: String,
    data: Vec<f32>
}


impl Vec3Attribute
{
    pub fn create(name: &str, data: Vec<f32>) -> Result<Vec3Attribute, Error>
    {
        Ok(Vec3Attribute{name: String::from(name), data})
    }

    pub fn at(&self, vertex_id: VertexID) -> Vec3
    {
        vec3(self.data[vertex_id.value() * 3], self.data[vertex_id.value() * 3 + 1], self.data[vertex_id.value() * 3 + 2])
    }

    pub fn set(&mut self, vertex_id: VertexID, value: Vec3) {
        let no_components = self.no_components();
        for i in 0..no_components {
            self.data[vertex_id.value() * no_components] = value[i];
        }
    }
}

impl Attribute for Vec3Attribute
{
    fn no_components(&self) -> usize
    {
        3
    }

    fn name(&self) -> &str
    {
        &self.name
    }

    fn data(&self) -> &Vec<f32>
    {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<f32>
    {
        &mut self.data
    }
}