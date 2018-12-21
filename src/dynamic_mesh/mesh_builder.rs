
use crate::dynamic_mesh::DynamicMesh;

#[derive(Debug)]
pub enum Error {
    NoPositionsSpecified {message: String}
}

/// Contains functionality to construct a mesh from either raw data (indices, positions, normals)
/// or from simple geometric shapes (sphere, box, ..)
///
/// # Examples
///
/// ```
/// # use geo_proc::dynamic_mesh::{MeshBuilder, Error};
/// #
/// # fn main() -> Result<(), Box<Error>> {
/// let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
///                                    0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
///                                    0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];
/// let mesh = MeshBuilder::new().with_positions(positions).build()?;
/// assert_eq!(mesh.no_faces(), 3);
/// #
/// #   Ok(())
/// # }
/// ```
///
/// ```
/// # use geo_proc::dynamic_mesh::{MeshBuilder, Error};
/// #
/// # fn main() -> Result<(), Box<Error>> {
/// let mesh = MeshBuilder::new().icosahedron().build()?;
/// #
/// #   Ok(())
/// # }
/// ```
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

    pub fn with_indices(mut self, indices: Vec<u32>) -> Self
    {
        self.indices = Some(indices);
        self
    }

    pub fn with_positions(mut self, positions: Vec<f32>) -> Self
    {
        self.positions = Some(positions);
        self
    }

    pub fn with_normals(mut self, normals: Vec<f32>) -> Self
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

    pub fn icosahedron(mut self) -> Self
    {
        let x = 0.525731112119133606;
        let z = 0.850650808352039932;

        self.positions = Some(vec!(
            -x, 0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z,
            0.0, z, x, 0.0, z, -x, 0.0, -z, x, 0.0, -z, -x,
            z, x, 0.0, -z, x, 0.0, z, -x, 0.0, -z, -x, 0.0
        ));
        self.indices = Some(vec!(
            0, 1, 4, 0, 4, 9, 9, 4, 5, 4, 8, 5, 4, 1, 8,
            8, 1, 10, 8, 10, 3, 5, 8, 3, 5, 3, 2, 2, 3, 7,
            7, 3, 10, 7, 10, 6, 7, 6, 11, 11, 6, 0, 0, 6, 1,
            6, 10, 1, 9, 11, 0, 9, 2, 11, 9, 5, 2, 7, 11, 2
        ));
        self
    }
}
