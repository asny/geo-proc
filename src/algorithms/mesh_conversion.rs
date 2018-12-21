use crate::mesh::*;

impl StaticMesh
{
    pub fn to_dynamic(&self) -> DynamicMesh
    {
        let indices = self.indices().clone();
        let positions = self.attribute("position").unwrap().data.clone();
        let normals = self.attribute("normal").map(|att| att.data.clone());

        DynamicMesh::new_with_connectivity(indices, positions, normals)
    }
}

impl DynamicMesh
{
    pub fn to_static(&self) -> StaticMesh
    {
        let indices = indices(self);

        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for v3 in self.vertex_iter().map(|ref vertex_id| self.position(vertex_id)) {
            positions.push(v3.x); positions.push(v3.y); positions.push(v3.z);
        }

        if let Some(normals) = normals(self) {
            StaticMesh::create(indices, att!["position" => (positions, 3), "normal" => (normals, 3)]).unwrap()
        }
        else {
            StaticMesh::create(indices, att!["position" => (positions, 3)]).unwrap()
        }
    }
}

fn indices(mesh: &DynamicMesh) -> Vec<u32>
{
    let vertices: Vec<VertexID> = mesh.vertex_iter().collect();
    let mut indices = Vec::with_capacity(mesh.no_faces() * 3);
    for face_id in mesh.face_iter()
    {
        for walker in mesh.face_halfedge_iter(&face_id) {
            let vertex_id = walker.vertex_id().unwrap();
            let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
            indices.push(index as u32);
        }
    }
    indices
}

fn normals(mesh: &DynamicMesh) -> Option<Vec<f32>>
{
    let mut nor = Vec::with_capacity(mesh.no_vertices() * 3);
    for vertex_id in mesh.vertex_iter() {
        if let Some(normal) = mesh.normal(&vertex_id)
        {
            nor.push(normal.x); nor.push(normal.y); nor.push(normal.z);
        }
        else { return None; }
    }
    Some(nor)
}