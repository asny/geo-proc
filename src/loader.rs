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
        let indices = match m.mesh.indices.len() > 0 { true => m.mesh.indices.clone(), false => (0..m.mesh.positions.len() as u32/3).collect() };
        let normals = if m.mesh.normals.len() > 0 { Some(m.mesh.normals) } else { None };
        let mesh = Mesh::new_with_connectivity(indices, m.mesh.positions, normals);
        result.push(mesh);
    }
    Ok(result)
}