use std::fs::File;
use std::io::prelude::*;
use crate::*;
use std;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    FileTypeNotSupported {message: String},
    ExtensionNotSpecified {message: String},
    FileNameNotSpecified {message: String}
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::IO(other)
    }
}

pub fn save(mesh: &Mesh, filename: &str) -> Result<(), Error>
{
    let splitted: Vec<&str> = filename.split('.').collect();
    if splitted.len() == 0
    {
        return Err(Error::FileNameNotSpecified {message: format!("Filename is not specified!")})
    }
    if splitted.len() == 1
    {
        return Err(Error::ExtensionNotSpecified {message: format!("Extension for file {} is not specified!", splitted[0])})
    }
    let extension = splitted[1];

    let data = if extension == "obj" {
        Ok(parse_as_obj(mesh))
    }
    else if extension == "poly" {
        Ok(parse_as_poly(mesh))
    }
    else { Err(Error::FileTypeNotSupported {message: format!("Extension {} of file {} is not supported!", extension, splitted[0])}) };
    save_model(&data?, filename)?;
    Ok(())
}

fn save_as_obj(mesh: &Mesh, name: &str) -> Result<(), Error>
{
    let data = parse_as_obj(mesh);
    save_model(&data, name)?;
    Ok(())
}

fn parse_as_obj(mesh: &Mesh) -> String
{
    let mut output = String::from("o object\n");

    let positions = mesh.positions_buffer();
    for i in 0..mesh.no_vertices()
    {
        output = format!("{}v {} {} {}\n", output, positions[i*3], positions[i*3 + 1], positions[i*3 + 2]);
    }

    if let Some(ref normals) = mesh.normals_buffer()
    {
        for i in 0..mesh.no_vertices()
        {
            output = format!("{}vn {} {} {}\n", output, normals[i*3], normals[i*3 + 1], normals[i*3 + 2]);
        }
    }

    let indices = mesh.indices_buffer();
    for i in 0..mesh.no_faces() {
        let mut face = String::new();
        for j in 0..3 {
            let index = indices[i*3 + j] + 1;
            face = format!("{} {}/{}/{}", face, index, index, index);
        }
        output = format!("{}f{}\n", output, face);
    }
    output
}

fn save_as_poly(mesh: &Mesh, name: &str) -> Result<(), Error>
{
    let data = parse_as_poly(mesh);
    save_model(&data, name)?;
    Ok(())
}

fn parse_as_poly(mesh: &Mesh) -> String
{
    let mut output = format!("{} 3 0 0\n", mesh.no_vertices());

    let positions = &mesh.positions_buffer();
    for i in 0..mesh.no_vertices()
    {
        output = format!("{}{} {} {} {}\n", output, i+1, positions[i*3], positions[i*3 + 1], positions[i*3 + 2]);
    }

    output = format!("{}{} 0\n", output, mesh.no_faces());
    let indices = mesh.indices_buffer();
    for i in 0..mesh.no_faces() {
        output = format!("{}1 0 0\n", output);
        output = format!("{}3 {} {} {}\n", output, indices[i*3] + 1, indices[i*3 + 1] + 1, indices[i*3 + 2] + 1);
    }
    output
}


fn save_model(data: &str, name: &str) -> std::io::Result<()>
{
    let mut file = File::create(name)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}