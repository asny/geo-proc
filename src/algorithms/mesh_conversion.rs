
use dynamic_mesh::*;
use static_mesh::*;
use mesh::Renderable;

pub fn static_to_dynamic(mesh: &StaticMesh) -> DynamicMesh
{
    let indices = mesh.indices();
    let positions = mesh.get_attribute("position").unwrap().data;
    let normals = mesh.get_attribute("normal").map(|att| att.data);

    DynamicMesh::create(indices, positions, normals)
}

pub fn dynamic_to_static(mesh: &DynamicMesh) -> StaticMesh
{
    let indices = mesh.indices();
    let positions = mesh.get_attribute("position").unwrap().data;

    if let Some(normals) = mesh.get_attribute("normal") {
        StaticMesh::create(indices, att!["position" => (positions, 3), "normal" => (normals.data, 3)]).unwrap()
    }
    else {
        StaticMesh::create(indices, att!["position" => (positions, 3)]).unwrap()
    }
}