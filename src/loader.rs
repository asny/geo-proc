use std::path::Path;
use std::path::PathBuf;
use tobj;
use mesh;
use static_mesh::StaticMesh;
use dynamic_mesh::DynamicMesh;

#[derive(Debug)]
pub enum Error {
    ObjLoader(tobj::LoadError),
    Mesh(mesh::Error),
    FileDoesntContainModel{message: String}
}

impl From<tobj::LoadError> for Error {
    fn from(other: tobj::LoadError) -> Self {
        Error::ObjLoader(other)
    }
}

impl From<mesh::Error> for Error {
    fn from(other: mesh::Error) -> Self {
        Error::Mesh(other)
    }
}

pub fn load_obj_as_simple_mesh(name: &str) -> Result<StaticMesh, Error>
{
    let m = load_obj(name)?;
    // Create mesh
    let indices = match m.indices.len() > 0 { true => m.indices.clone(), false => (0..m.positions.len() as u32/3).collect() };
    let mut mesh = StaticMesh::create(indices, m.positions.clone())?;

    if m.normals.len() > 0
    {
        mesh.add_vec3_attribute("normal", m.normals.clone())?;
    }

    Ok(mesh)
}

pub fn load_obj_as_halfedge_mesh(name: &str) -> Result<DynamicMesh, Error>
{
    let m = load_obj(name)?;

    let indices = match m.indices.len() > 0 { true => m.indices.clone(), false => (0..m.positions.len() as u32/3).collect() };
    let normals = match m.normals.len() > 0 { true => Some(m.normals.clone()), false => None };

    Ok(DynamicMesh::create(indices, m.positions.clone(), normals))
}

fn load_obj(name: &str) -> Result<tobj::Mesh, Error>
{
    let root_path: PathBuf = PathBuf::from("");
    let (models, _materials) = tobj::load_obj(&resource_name_to_path(&root_path,name))?;
    let m = models.first().ok_or(Error::FileDoesntContainModel {message: format!("The file {} doesn't contain a model", name)})?.mesh.clone();
    Ok(m)
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}