
use tri_mesh::prelude::*;

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

pub fn save(mesh: &Mesh, path: &str) -> Result<(), Error>
{
    let path_split: Vec<&str> = path.split('/').collect();
    if path_split.len() == 0
    {
        return Err(Error::FileNameNotSpecified {message: format!("Filename is not specified!")})
    }
    let filename = path_split.last().unwrap();
    let splitted: Vec<&str> = filename.split('.').collect();
    if splitted.len() == 1
    {
        return Err(Error::ExtensionNotSpecified {message: format!("Extension for file {} is not specified!", splitted[0])})
    }
    let extension = splitted[1];

    let data = if extension == "obj" {
        Ok(mesh.parse_as_obj())
    }
    else if extension == "poly" {
        Ok(parse_as_poly(mesh))
    }
    else { Err(Error::FileTypeNotSupported {message: format!("Extension {} of file {} is not supported!", extension, splitted[0])}) };
    std::fs::write(path, data?)?;
    Ok(())
}

pub fn parse_as_poly(mesh: &Mesh) -> String
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