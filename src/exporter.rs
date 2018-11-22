use std::fs::File;
use std::io::prelude::*;
use mesh;
use std;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Mesh(mesh::Error)
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::IO(other)
    }
}

impl From<mesh::Error> for Error {
    fn from(other: mesh::Error) -> Self {
        Error::Mesh(other)
    }
}

pub fn save_as_obj(name: &str) -> Result<(), Error>
{
    unimplemented!();
    let output = "";
    save_model(&output, name)?;
    Ok(())
}

pub fn save_as_poly(mesh: &mesh::StaticMesh, name: &str) -> Result<(), Error>
{
    let mut output = format!("{} 3 0 0 \n", mesh.no_vertices());

    let positions = &mesh.attribute("position").unwrap().data;
    for i in 0..mesh.no_vertices()
    {
        output = format!("{}{} {} {} {} \n", output, i+1, positions[i*3], positions[i*3 + 1], positions[i*3 + 2]);
    }

    output = format!("{}{} 0 \n", output, mesh.no_faces());
    let indices = mesh.indices();
    for i in 0..mesh.no_faces() {
        output = format!("{}1 0 0 \n", output);
        output = format!("{}3 {} {} {} \n", output, indices[i*3] + 1, indices[i*3 + 1] + 1, indices[i*3 + 2] + 1);

    }

    save_model(&output, name)?;
    Ok(())
}

fn save_model(data: &str, name: &str) -> std::io::Result<()>
{
    let mut file = File::create(name)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}