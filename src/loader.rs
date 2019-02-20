use std::path::PathBuf;
use tobj;
use tri_mesh::prelude::*;

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
        let mut mesh_builder = tri_mesh::mesh_builder::MeshBuilder::new().with_positions(m.mesh.positions);
        if m.mesh.indices.len() > 0 {
            mesh_builder = mesh_builder.with_indices(m.mesh.indices);
        }
        result.push(mesh_builder.build().unwrap());
    }
    Ok(result)
}

pub fn parse_obj(source: String) -> Result<Vec<Mesh>, Error>
{
    let objs = wavefront_obj::obj::parse(source).unwrap();
    let obj = objs.objects.first().unwrap();

    let mut positions = Vec::new();
    obj.vertices.iter().for_each(|v| {positions.push(v.x as f32); positions.push(v.y as f32); positions.push(v.z as f32);});
    let mut indices = Vec::new();
    for shape in obj.geometry.first().unwrap().shapes.iter() {
        match shape.primitive {
            wavefront_obj::obj::Primitive::Triangle(i0, i1, i2) => {
                indices.push(i0.0 as u32);
                indices.push(i1.0 as u32);
                indices.push(i2.0 as u32);
            },
            _ => {}
        }
    }

    let mut result = Vec::new();
    let mut mesh = tri_mesh::mesh_builder::MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();
    result.push(mesh);
    Ok(result)
}