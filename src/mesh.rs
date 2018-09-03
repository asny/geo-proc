use attribute;
use glm;
use std::string::String;
use attribute::Attribute;
use traversal::*;
use std::rc::Rc;
use iterators::*;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    WrongSizeOfAttribute {message: String},
    Attribute(attribute::Error)
}

impl From<attribute::Error> for Error {
    fn from(other: attribute::Error) -> Self {
        Error::Attribute(other)
    }
}

pub struct Mesh {
    pub no_vertices: usize,
    pub no_faces: usize,
    pub indices: Option<Vec<u32>>,
    pub positions: attribute::Vec3Attribute,
    int_attributes: Vec<attribute::IntAttribute>,
    vec2_attributes: Vec<attribute::Vec2Attribute>,
    vec3_attributes: Vec<attribute::Vec3Attribute>,
    connectivity_info: Rc<ConnectivityInfo>
}


impl Mesh
{
    pub fn create(positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let no_faces = no_vertices/3;
        let position_attribute = attribute::Vec3Attribute::create("position", positions)?;
        let mut mesh = Mesh { no_vertices, no_faces, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: None, positions: position_attribute, int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() };
        for face in 0..no_faces {
            let v0 = mesh.create_vertex();
            let v1 = mesh.create_vertex();
            let v2 = mesh.create_vertex();
            mesh.create_face(&v0, &v1, &v2);
        }
        Ok(mesh)
    }

    pub fn create_indexed(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let position_attribute = attribute::Vec3Attribute::create("position", positions)?;
        let mut mesh = Mesh { no_vertices, no_faces, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: Some(indices.clone()), positions: position_attribute, int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() };
        for vertex in 0..no_vertices {
            mesh.create_vertex();
        }
        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);
            mesh.create_face(&v0, &v1, &v2);
        }

        Ok(mesh)
    }

    fn create_vertex(&mut self) -> VertexID
    {
        self.connectivity_info.create_vertex()
    }

    fn connecting_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        for halfedge_id in self.one_ring_iterator(vertex_id1) {
            if self.halfedge_walker(&halfedge_id).vertex().deref() == *vertex_id2 {
                return Some(halfedge_id)
            }
        }
        None
    }

    fn create_face(&mut self, vertex_id1: &VertexID, vertex_id2: &VertexID, vertex_id3: &VertexID) -> FaceID
    {
        let id = self.connectivity_info.create_face();

        let halfedge1 = self.connectivity_info.create_halfedge();
        let halfedge2 = self.connectivity_info.create_halfedge();
        let halfedge3 = self.connectivity_info.create_halfedge();

        self.connectivity_info.set_vertex_halfedge(&vertex_id1, &halfedge1);
        self.connectivity_info.set_vertex_halfedge(&vertex_id2, &halfedge2);
        self.connectivity_info.set_vertex_halfedge(&vertex_id3, &halfedge3);

        self.connectivity_info.set_halfedge_vertex(&halfedge1, &vertex_id2);
        self.connectivity_info.set_halfedge_vertex(&halfedge2, &vertex_id3);
        self.connectivity_info.set_halfedge_vertex(&halfedge3, &vertex_id1);

        self.connectivity_info.set_halfedge_next(&halfedge1, &halfedge2);
        self.connectivity_info.set_halfedge_next(&halfedge2, &halfedge3);
        self.connectivity_info.set_halfedge_next(&halfedge3, &halfedge1);

        self.connectivity_info.set_halfedge_face(&halfedge1, &id);
        self.connectivity_info.set_halfedge_face(&halfedge2, &id);
        self.connectivity_info.set_halfedge_face(&halfedge3, &id);

        self.connectivity_info.set_face_halfedge(&id, &halfedge1);

        let halfedge4 = match self.connecting_edge(vertex_id2, vertex_id1)
            { Some(e) => {e}, None => { self.connectivity_info.create_halfedge() } };
        let halfedge5 = match self.connecting_edge(vertex_id3, vertex_id2)
            { Some(e) => {e}, None => { self.connectivity_info.create_halfedge() } };
        let halfedge6 = match self.connecting_edge(vertex_id1, vertex_id3)
            { Some(e) => {e}, None => { self.connectivity_info.create_halfedge() } };

        self.connectivity_info.set_halfedge_vertex(&halfedge4, &vertex_id1);
        self.connectivity_info.set_halfedge_vertex(&halfedge5, &vertex_id2);
        self.connectivity_info.set_halfedge_vertex(&halfedge6, &vertex_id3);

        self.connectivity_info.set_halfedge_twin(&halfedge1, &halfedge4);
        self.connectivity_info.set_halfedge_twin(&halfedge2, &halfedge5);
        self.connectivity_info.set_halfedge_twin(&halfedge3, &halfedge6);

        self.connectivity_info.set_halfedge_twin(&halfedge4, &halfedge1);
        self.connectivity_info.set_halfedge_twin(&halfedge5, &halfedge2);
        self.connectivity_info.set_halfedge_twin(&halfedge6, &halfedge3);

        self.connectivity_info.set_halfedge_next(&halfedge4, &halfedge6);
        self.connectivity_info.set_halfedge_next(&halfedge5, &halfedge4);
        self.connectivity_info.set_halfedge_next(&halfedge6, &halfedge5);

        id
    }

    pub fn vertex_walker(&self, vertex_id: &VertexID) -> VertexWalker
    {
        VertexWalker::new(vertex_id.clone(), self.connectivity_info.clone())
    }

    pub fn halfedge_walker(&self, halfedge_id: &HalfEdgeID) -> HalfEdgeWalker
    {
        HalfEdgeWalker::new(halfedge_id.clone(), self.connectivity_info.clone())
    }

    pub fn face_walker(&self, face_id: &FaceID) -> FaceWalker
    {
        FaceWalker::new(face_id.clone(), self.connectivity_info.clone())
    }

    pub fn one_ring_iterator(&self, vertex_id: &VertexID) -> OneRingIterator
    {
        OneRingIterator::new(vertex_id, self.connectivity_info.clone())
    }

    pub fn face_iterator(&self, face_id: &FaceID) -> FaceIterator
    {
        FaceIterator::new(face_id, self.connectivity_info.clone())
    }

    pub fn get_vec2_attribute(&self, name: &str) -> Result<&attribute::Vec2Attribute, Error>
    {
        for attribute in self.vec2_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_vec3_attribute(&self, name: &str) -> Result<&attribute::Vec3Attribute, Error>
    {
        for attribute in self.vec3_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_vec2_attribute_mut(&mut self, name: &str) -> Result<&mut attribute::Vec2Attribute, Error>
    {
        for attribute in self.vec2_attributes.iter_mut() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_vec3_attribute_mut(&mut self, name: &str) -> Result<&mut attribute::Vec3Attribute, Error>
    {
        for attribute in self.vec3_attributes.iter_mut() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_attributes(&self) -> Vec<&attribute::Attribute>
    {
        let mut att : Vec<&Attribute> = Vec::new();
        att.push(&self.positions);
        for attribute in self.vec2_attributes.iter() {
            att.push(attribute);
        }
        for attribute in self.vec3_attributes.iter() {
            att.push(attribute);
        }
        att
    }

    pub fn add_custom_vec2_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices != data.len()/2 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices)})
        }
        let custom_attribute = attribute::Vec2Attribute::create(name, data)?;
        self.vec2_attributes.push(custom_attribute);
        Ok(())
    }

    pub fn add_custom_vec3_attribute(&mut self, name: &str, data: Vec<f32>) -> Result<(), Error>
    {
        if self.no_vertices != data.len()/3 {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices)})
        }
        let custom_attribute = attribute::Vec3Attribute::create(name, data)?;
        self.vec3_attributes.push(custom_attribute);
        Ok(())
    }

    pub fn add_custom_int_attribute(&mut self, name: &str, data: &Vec<u32>) -> Result<(), Error>
    {
        if self.no_vertices != data.len() {
            return Err(Error::WrongSizeOfAttribute {message: format!("The data for {} does not have the correct size, it should be {}", name, self.no_vertices)})
        }
        let custom_attribute = attribute::IntAttribute::create(name, data)?;
        self.int_attributes.push(custom_attribute);
        Ok(())
    }

    fn indices_of(&self, face_id: usize) -> [usize; 3]
    {
        let index0: usize;
        let index1: usize;
        let index2: usize;
        match self.indices {
            Some(ref indices) => {
                index0 = indices[face_id*3] as usize;
                index1 = indices[face_id*3+1] as usize;
                index2 = indices[face_id*3+2] as usize;
            },
            None => {
                index0 = face_id;
                index1 = face_id+1;
                index2 = face_id+2;
            }
        }
        [index0, index1, index2]
    }

    fn normal_of(&self, face_id: &FaceID) -> glm::Vec3
    {
        let indices = self.indices_of(face_id.val());
        let p0 = self.positions.at(&VertexID::new(indices[0]));
        let p1 = self.positions.at(&VertexID::new(indices[1]));
        let p2 = self.positions.at(&VertexID::new(indices[2]));

        glm::normalize(glm::cross(p1 - p0, p2 - p0))
    }

    pub fn compute_normals(&mut self)
    {
        let mut normals = vec![0.0; 3 * self.no_vertices];
        {
            for face_id in 0..self.no_faces {
                let normal = self.normal_of(&FaceID::new(face_id));
                let indices = self.indices_of(face_id);
                for index in indices.iter() {
                    normals[3 * *index] += normal.x;
                    normals[3 * *index+1] += normal.y;
                    normals[3 * *index+2] += normal.z;
                }
            }
        }
        {
            let normals_dest = self.get_vec3_attribute_mut("normal").unwrap();

            for i in 0..normals.len()/3 {
                let n = glm::normalize(glm::vec3(normals[i*3], normals[i*3+1], normals[i*3+2]));
                normals_dest.set(&VertexID::new(i), n);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_create_vertex() {
        let positions: Vec<f32> = vec![];
        let mut mesh = Mesh::create(positions).unwrap();

        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        let v3 = mesh.create_vertex();
        assert_eq!(v1.val(), 0);
        assert_eq!(v2.val(), 1);
        assert_eq!(v3.val(), 2);
    }

    #[test]
    fn test_create_face() {
        let positions: Vec<f32> = vec![];
        let mut mesh = Mesh::create(positions).unwrap();

        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        let v3 = mesh.create_vertex();
        let f1 = mesh.create_face(&v1, &v2, &v3);
        assert_eq!(f1.val(), 0);

        let t1 = mesh.vertex_walker(&v1).halfedge().deref();
        assert_eq!(t1.val(), 0);

        let t2 = mesh.vertex_walker(&v1).halfedge().twin().deref();
        assert_eq!(t2.val(), 3);

        let t3 = mesh.vertex_walker(&v2).halfedge().next().next().vertex().deref();
        assert_eq!(t3.val(), v2.val());

        let t4 = mesh.face_walker(&f1).halfedge().twin().twin().vertex().halfedge().face().deref();
        assert_eq!(t4.val(), f1.val());

        let t5 = mesh.halfedge_walker(&t1).twin().deref();
        assert_eq!(t5.val(), 3);
    }

    #[test]
    fn test_one_ring_iterator() {
    }

    #[test]
    fn test_face_iterator() {
        let mesh = create_single_face();
        let mut i = 0;
        for edge in mesh.face_iterator(&FaceID::new(0)) {
            assert_eq!(edge.deref().val(), i);
            i = i+1;
        }
    }

    #[test]
    fn test_normal() {
        let mesh = create_test_object().unwrap();
        let normal = mesh.get_vec3_attribute("normal").unwrap().at(&VertexID::new(0));
        let computed_normal = mesh.normal_of(&FaceID::new(0));
        assert_eq!(normal.x, computed_normal.x);
        assert_eq!(normal.y, computed_normal.y);
        assert_eq!(normal.z, computed_normal.z);
    }

    fn create_single_face() -> Mesh
    {
        let positions: Vec<f32> = vec![];
        let mut mesh = Mesh::create(positions).unwrap();

        let v0 = mesh.create_vertex();
        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        mesh.create_face(&v0, &v1, &v2);
        mesh
    }

    fn create_test_object() -> Result<Mesh, Error>
    {
        let positions: Vec<f32> = vec![
            1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0,

            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,

            1.0, -1.0, -1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, -1.0,
            -1.0, -1.0, -1.0,

            -1.0, -1.0, 1.0,
            1.0, -1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0,

            1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            1.0, 1.0, 1.0,
            1.0, -1.0, 1.0,
            1.0, -1.0, -1.0,

            -1.0, 1.0, -1.0,
            -1.0, -1.0, -1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, -1.0, -1.0
        ];
        let normals: Vec<f32> = vec![
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 0.0,

            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, -1.0, 0.0,

            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,
            0.0, 0.0, -1.0,

            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 1.0,

            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,

            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0,
            -1.0, 0.0, 0.0
        ];

        let uvs: Vec<f32> = vec![
            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,

            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,

            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,

            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,

            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0,

            1.0, 0.0,
            0.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
            1.0, 1.0,
            0.0, 0.0
        ];

        let mut mesh = Mesh::create(positions)?;
        mesh.add_custom_vec3_attribute("normal", normals)?;
        mesh.add_custom_vec2_attribute("uv_coordinate", uvs)?;
        Ok(mesh)
    }
}