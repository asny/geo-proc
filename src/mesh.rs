#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    FailedToFindEntryForVertexID {message: String},
    WrongSizeOfAttribute {message: String},
    NeedPositionAttributeToCreateMesh {message: String}
}

#[macro_export]
macro_rules! att {
    ($( $name: expr => ($data: expr, $no_components: expr)),*) => {{
         let mut vec = Vec::new();
         $( vec.push(mesh::Attribute::new($name, $no_components, $data)); )*
         vec
    }}
}

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub no_components: usize,
    pub data: Vec<f32>
}

impl Attribute {
    pub fn new(name: &str, no_components: usize, data: Vec<f32>) -> Attribute
    {
        Attribute {name: name.to_string(), no_components, data}
    }
}

pub trait Renderable
{
    fn indices(&self) -> Vec<u32>;

    fn get_attribute(&self, name: &str) -> Option<Attribute>;

    fn no_vertices(&self) -> usize;
}