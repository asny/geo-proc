use std::path::Path;
use tobj;
use mesh;
use glm;

#[derive(Debug)]
pub enum Error {
    TOBJ(tobj::LoadError),
    Mesh(mesh::Error),
    FileDoesntContainModel{message: String}
}

impl From<tobj::LoadError> for Error {
    fn from(other: tobj::LoadError) -> Self {
        Error::TOBJ(other)
    }
}

impl From<mesh::Error> for Error {
    fn from(other: mesh::Error) -> Self {
        Error::Mesh(other)
    }
}

pub fn load_obj(name: &str) -> Result<mesh::Mesh, Error>
{
    let (models, _materials) = tobj::load_obj(&Path::new(name))?;
    let m = models.first().ok_or(Error::FileDoesntContainModel {message: format!("The file {} doesn't contain a model", name)})?;
    let positions = &m.mesh.positions;
    let mut p = Vec::with_capacity(positions.len()/3);
    for i in 0..positions.len()/3 {
        p.push(glm::vec3(positions[3*i], positions[3*i+1], positions[3*i+2]))
    }

    Ok(mesh::Mesh::create(p)?)
}