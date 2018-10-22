pub use static_mesh::*;
pub use dynamic_mesh::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    FailedToFindEntryForVertexID {message: String},
    WrongSizeOfAttribute {message: String},
    NeedPositionAttributeToCreateMesh {message: String}
}