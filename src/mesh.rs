use attribute;
use glm;
use std::string::String;
use attribute::Attribute;
use traversal::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

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
        let position_attribute = attribute::Vec3Attribute::create("position", positions)?;
        Ok(Mesh { no_vertices, no_faces: no_vertices/3, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: None, positions: position_attribute, int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() })
    }

    pub fn create_indexed(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let position_attribute = attribute::Vec3Attribute::create("position", positions)?;

        Ok(Mesh { no_vertices, no_faces: indices.len()/3, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: Some(indices), positions: position_attribute, int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() })
    }

    fn create_vertex(&mut self) -> VertexID
    {
        let mut vec = &mut *RefCell::borrow_mut(&self.connectivity_info.vertices);
        let id = VertexID::new(vec.len());
        vec.push(Vertex { halfedge: HalfEdgeID::null() });
        id
    }

    fn create_halfedge(&mut self, vertex_id: &VertexID, face_id: &FaceID) -> HalfEdgeID
    {
        let mut halfedges = &mut *RefCell::borrow_mut(&self.connectivity_info.halfedges);

        let id = HalfEdgeID::new(halfedges.len());
        halfedges.push(HalfEdge { vertex: vertex_id.clone(), twin: HalfEdgeID::null(), next: HalfEdgeID::null(), face: face_id.clone() });

        id
    }

    fn create_face(&mut self, vertex_id1: &VertexID, vertex_id2: &VertexID, vertex_id3: &VertexID) -> FaceID
    {
        let mut id = FaceID::null();
        {
            let mut vec = RefCell::borrow_mut(&self.connectivity_info.faces);
            id = FaceID::new(vec.len());
            let face = Face { halfedge: HalfEdgeID::null() };
            vec.push(face);
        }

        let halfedge1 = self.create_halfedge(vertex_id2, &id);
        let halfedge2 = self.create_halfedge(vertex_id3, &id);
        let halfedge3 = self.create_halfedge(vertex_id1, &id);

        let halfedge4 = self.create_halfedge(vertex_id3, &FaceID::null());
        let halfedge5 = self.create_halfedge(vertex_id2, &FaceID::null());
        let halfedge6 = self.create_halfedge(vertex_id1, &FaceID::null());

        RefCell::borrow_mut(&self.connectivity_info.vertices)[vertex_id1.val()].halfedge = halfedge1.clone();
        RefCell::borrow_mut(&self.connectivity_info.vertices)[vertex_id2.val()].halfedge = halfedge2.clone();
        RefCell::borrow_mut(&self.connectivity_info.vertices)[vertex_id3.val()].halfedge = halfedge3.clone();

        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge1.val()].twin = halfedge6.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge2.val()].twin = halfedge5.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge3.val()].twin = halfedge4.clone();

        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge6.val()].twin = halfedge1.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge5.val()].twin = halfedge2.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge4.val()].twin = halfedge3.clone();

        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge1.val()].next = halfedge2.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge2.val()].next = halfedge3.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge3.val()].next = halfedge1.clone();

        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge6.val()].next = halfedge4.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge4.val()].next = halfedge5.clone();
        RefCell::borrow_mut(&self.connectivity_info.halfedges)[halfedge5.val()].next = halfedge6.clone();

        id
    }

    fn vertex_walker(&self, vertex_id: &VertexID) -> VertexWalker
    {
        VertexWalker::new(vertex_id.clone(), self.connectivity_info.clone())
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

    fn normal_of(&self, face_id: usize) -> glm::Vec3
    {
        let indices = self.indices_of(face_id);
        let p0 = self.positions.at(indices[0]);
        let p1 = self.positions.at(indices[1]);
        let p2 = self.positions.at(indices[2]);

        glm::normalize(glm::cross(p1 - p0, p2 - p0))
    }

    pub fn compute_normals(&mut self)
    {
        let mut normals = vec![0.0; 3 * self.no_vertices];
        {
            for face_id in 0..self.no_faces {
                let normal = self.normal_of(face_id);
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
                normals_dest.set(i, n);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk() {
        let positions: Vec<f32> = vec![
            1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0,
            1.0, 1.0, 1.0];
        let mut mesh = Mesh::create(positions).unwrap();

        let mut v1 = mesh.create_vertex();
        let mut v2 = mesh.create_vertex();
        let mut v3 = mesh.create_vertex();
        let f1 = mesh.create_face(&v1, &v2, &v3);
        assert_eq!(v1.val(), 0);
        assert_eq!(v2.val(), 1);
        assert_eq!(v3.val(), 2);
        assert_eq!(f1.val(), 0);

        // TODO: Test position attribute

        let t1 = mesh.vertex_walker(&v1).halfedge().deref();
        assert_eq!(t1.val(), 0);

        let t2 = mesh.vertex_walker(&v1).halfedge().twin().deref();
        assert_eq!(t2.val(), 5);

        let t2 = mesh.vertex_walker(&v2).halfedge().next().next().vertex().deref();
        assert_eq!(t2.val(), v2.val());

        //let v2 = RefCell::borrow(&v1);
        //let halfedge = v2.halfedge();
        //let test_vertex = halfedge.deref().vertex();
        //let halfedge = face.halfedge();


        //assert!(halfedge.is_valid());
    }

    #[test]
    fn test_normal() {
        let mesh = create_test_object().unwrap();
        let normal = mesh.get_vec3_attribute("normal").unwrap().at(0);
        let computed_normal = mesh.normal_of(0);
        assert_eq!(normal.x, computed_normal.x);
        assert_eq!(normal.y, computed_normal.y);
        assert_eq!(normal.z, computed_normal.z);
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