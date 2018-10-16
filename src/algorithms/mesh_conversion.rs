
use dynamic_mesh::*;
use static_mesh::*;
use mesh::Renderable;

impl DynamicMesh
{
    pub fn to_static(&self) -> StaticMesh
    {
        let indices = self.indices();
        let positions = self.get_attribute("position").unwrap().data;

        if let Some(normals) = self.get_attribute("normal") {
            StaticMesh::create(indices, att!["position" => (positions, 3), "normal" => (normals.data, 3)]).unwrap()
        }
        else {
            StaticMesh::create(indices, att!["position" => (positions, 3)]).unwrap()
        }
    }
}

impl StaticMesh
{
    pub fn to_dynamic(&self) -> DynamicMesh
    {
        let indices = self.indices();
        let positions = self.get_attribute("position").unwrap().data;
        let normals = self.get_attribute("normal").map(|att| att.data);

        DynamicMesh::create(indices, positions, normals)
    }
}