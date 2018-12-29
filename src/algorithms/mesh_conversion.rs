use crate::*;

impl DynamicMesh
{
    pub fn indices_buffer(&self) -> Vec<u32>
    {
        let vertices: Vec<VertexID> = self.vertex_iter().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iter()
        {
            for walker in self.face_halfedge_iter(&face_id) {
                let vertex_id = walker.vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        indices
    }

    pub fn positions_buffer(&self) -> Vec<f32>
    {
        let mut positions = Vec::with_capacity(self.no_vertices() * 3);
        for v3 in self.vertex_iter().map(|ref vertex_id| self.position(vertex_id)) {
            positions.push(v3.x); positions.push(v3.y); positions.push(v3.z);
        }
        positions
    }

    pub fn normals_buffer(&self) -> Option<Vec<f32>>
    {
        let mut nor = Vec::with_capacity(self.no_vertices() * 3);
        for vertex_id in self.vertex_iter() {
            if let Some(normal) = self.normal(&vertex_id)
            {
                nor.push(normal.x); nor.push(normal.y); nor.push(normal.z);
            }
            else { return None; }
        }
        Some(nor)
    }
}