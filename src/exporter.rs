use std::fs::File;
use std::io::prelude::*;
use mesh;
use std;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Mesh(mesh::Error),
    FileTypeNotSupported {message: String},
    ExtensionNotSpecified {message: String},
    FileNameNotSpecified {message: String}
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

pub fn save(mesh: &mesh::StaticMesh, filename: &str) -> Result<(), Error>
{
    if filename == ""
    {
        return Err(Error::FileNameNotSpecified {message: format!("Filename is not specified!")})
    }
    let splitted: Vec<&str> = filename.split('.').collect();
    if splitted.len() == 1
    {
        return Err(Error::ExtensionNotSpecified {message: format!("Extension for file {} is not specified!", splitted[1])})
    }

    let extension = splitted[2];
    let mut data = String::new();
    if extension == "obj" {
        data = parse_as_obj(mesh);
    }
    else if extension == "poly" {
        data = parse_as_poly(mesh);
    }
    else { return Err(Error::FileTypeNotSupported {message: format!("Extension {} of file {} is not supported!", extension, splitted[1])}) }
    save_model(&data, filename)?;
    Ok(())
}

fn save_as_obj(mesh: &mesh::StaticMesh, name: &str) -> Result<(), Error>
{
    let data = parse_as_obj(mesh);
    save_model(&data, name)?;
    Ok(())
}

fn parse_as_obj(mesh: &mesh::StaticMesh) -> String
{
    let mut output = String::from("o object\n");

    let positions = &mesh.attribute("position").unwrap().data;
    for i in 0..mesh.no_vertices()
    {
        output = format!("{}v {} {} {}\n", output, positions[i*3], positions[i*3 + 1], positions[i*3 + 2]);
    }

    if let Some(ref normals) = mesh.attribute("normal")
    {
        for i in 0..mesh.no_vertices()
        {
            output = format!("{}vn {} {} {}\n", output, normals.data[i*3], normals.data[i*3 + 1], normals.data[i*3 + 2]);
        }
    }

    let indices = mesh.indices();
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

fn save_as_poly(mesh: &mesh::StaticMesh, name: &str) -> Result<(), Error>
{
    let data = parse_as_poly(mesh);
    save_model(&data, name)?;
    Ok(())
}

fn parse_as_poly(mesh: &mesh::StaticMesh) -> String
{
    let mut output = format!("{} 3 0 0\n", mesh.no_vertices());

    let positions = &mesh.attribute("position").unwrap().data;
    for i in 0..mesh.no_vertices()
    {
        output = format!("{}{} {} {} {}\n", output, i+1, positions[i*3], positions[i*3 + 1], positions[i*3 + 2]);
    }

    output = format!("{}{} 0\n", output, mesh.no_faces());
    let indices = mesh.indices();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_as_obj()
    {
        let model = ::models::create_cube().unwrap();
        let data = parse_as_obj(&model);

        let truth = "o object
v 1 -1 -1
v 1 -1 1
v -1 -1 1
v -1 -1 -1
v 1 1 -1
v 1 1 1
v -1 1 1
v -1 1 -1
f 1/1/1 2/2/2 3/3/3
f 1/1/1 3/3/3 4/4/4
f 5/5/5 8/8/8 7/7/7
f 5/5/5 7/7/7 6/6/6
f 1/1/1 5/5/5 6/6/6
f 1/1/1 6/6/6 2/2/2
f 2/2/2 6/6/6 7/7/7
f 2/2/2 7/7/7 3/3/3
f 3/3/3 7/7/7 8/8/8
f 3/3/3 8/8/8 4/4/4
f 5/5/5 1/1/1 4/4/4
f 5/5/5 4/4/4 8/8/8
";

        assert_eq!(data, truth);
    }

    #[test]
    fn test_parse_as_poly()
    {
        let model = ::models::create_cube().unwrap();
        let data = parse_as_poly(&model);

        let truth = "8 3 0 0
1 1 -1 -1
2 1 -1 1
3 -1 -1 1
4 -1 -1 -1
5 1 1 -1
6 1 1 1
7 -1 1 1
8 -1 1 -1
12 0
1 0 0
3 1 2 3
1 0 0
3 1 3 4
1 0 0
3 5 8 7
1 0 0
3 5 7 6
1 0 0
3 1 5 6
1 0 0
3 1 6 2
1 0 0
3 2 6 7
1 0 0
3 2 7 3
1 0 0
3 3 7 8
1 0 0
3 3 8 4
1 0 0
3 5 1 4
1 0 0
3 5 4 8
";

        assert_eq!(data, truth);
    }
}