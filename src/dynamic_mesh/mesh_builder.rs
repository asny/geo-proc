
use crate::dynamic_mesh::DynamicMesh;

#[derive(Debug)]
pub enum Error {
    NoPositionsSpecified {message: String}
}

#[derive(Debug, Default)]
pub struct MeshBuilder {
    indices: Option<Vec<u32>>,
    positions: Option<Vec<f32>>,
    normals: Option<Vec<f32>>
}

impl MeshBuilder {

    pub fn new() -> Self
    {
        MeshBuilder {indices: None, positions: None, normals: None}
    }

    pub fn with_indices(&mut self, indices: Vec<u32>) -> &mut Self
    {
        self.indices = Some(indices);
        self
    }

    pub fn with_positions(&mut self, positions: Vec<f32>) -> &mut Self
    {
        self.positions = Some(positions);
        self
    }

    pub fn with_normals(&mut self, normals: Vec<f32>) -> &mut Self
    {
        self.positions = Some(normals);
        self
    }

    pub fn build(self) -> Result<DynamicMesh, Error>
    {
        let positions = self.positions.ok_or(
            Error::NoPositionsSpecified {message: format!("Did you forget to specify the vertex positions?")})?;

        if let Some(indices) = self.indices {
            Ok(DynamicMesh::new_with_connectivity(indices, positions, self.normals))
        }
        else {
            Ok(DynamicMesh::new(positions, self.normals))
        }
    }
}
