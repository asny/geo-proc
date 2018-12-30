use std::path::PathBuf;
use tobj;
use crate::*;

#[derive(Debug)]
pub enum Error {
    ObjLoader(tobj::LoadError),
    FileDoesntContainModel{message: String}
}

impl From<tobj::LoadError> for Error {
    fn from(other: tobj::LoadError) -> Self {
        Error::ObjLoader(other)
    }
}

pub fn load_obj(name: &str) -> Result<Vec<Mesh>, Error>
{
    let mut result = Vec::new();

    let (models, _materials) = tobj::load_obj(&PathBuf::from(name))?;
    if models.is_empty()
    {
        return Err(Error::FileDoesntContainModel {message: format!("The file {} doesn't contain a model", name)})
    }

    for m in models {
        let mut mesh_builder = MeshBuilder::new().with_positions(m.mesh.positions);
        if m.mesh.normals.len() > 0 {
            mesh_builder = mesh_builder.with_normals(m.mesh.normals);
        }
        if m.mesh.indices.len() > 0 {
            mesh_builder = mesh_builder.with_indices(m.mesh.indices);
        }
        result.push(mesh_builder.build().unwrap());
    }
    Ok(result)
}