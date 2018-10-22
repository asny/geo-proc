use mesh::*;

impl DynamicMesh
{
    pub fn to_static(&self) -> StaticMesh
    {
        let indices = indices(self);
        let positions = get_attribute(self, "position").unwrap().data;

        if let Some(normals) = get_attribute(self, "normal") {
            StaticMesh::create(indices, att!["position" => (positions, 3), "normal" => (normals.data, 3)]).unwrap()
        }
        else {
            StaticMesh::create(indices, att!["position" => (positions, 3)]).unwrap()
        }
    }
}

fn indices(mesh: &DynamicMesh) -> Vec<u32>
{
    let vertices: Vec<VertexID> = mesh.vertex_iterator().collect();
    let mut indices = Vec::with_capacity(mesh.no_faces() * 3);
    for face_id in mesh.face_iterator()
    {
        for walker in mesh.face_halfedge_iterator(&face_id) {
            let vertex_id = walker.vertex_id().unwrap();
            let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
            indices.push(index as u32);
        }
    }
    indices
}

fn get_attribute(mesh: &DynamicMesh, name: &str) -> Option<Attribute>
{
    match name {
        "position" => {
            let mut pos = Vec::with_capacity(mesh.no_vertices() * 3);
            for v3 in mesh.vertex_iterator().map(|ref vertex_id| mesh.position(vertex_id)) {
                pos.push(v3.x); pos.push(v3.y); pos.push(v3.z);
            }
            Some(Attribute::new("position", 3, pos))
        },
        "normal" => {
            let mut nor = Vec::with_capacity(mesh.no_vertices() * 3);
            for vertex_id in mesh.vertex_iterator() {
                if let Some(normal) = mesh.normal(&vertex_id)
                {
                    nor.push(normal.x); nor.push(normal.y); nor.push(normal.z);
                }
                else { return None; }
            }
            Some(Attribute::new("normal", 3, nor))
        },
        _ => None
    }
}

impl StaticMesh
{
    pub fn to_dynamic(&self) -> DynamicMesh
    {
        let indices = self.indices().clone();
        let positions = self.attribute("position").unwrap().data;
        let normals = self.attribute("normal").map(|att| att.data);

        DynamicMesh::create(indices, positions, normals)
    }
}