use attribute;
use glm::*;
use std::string::String;
use attribute::Attribute;
use std::rc::Rc;
use ids::*;
use iterators::*;
use traversal::*;
use connectivity_info::ConnectivityInfo;

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
        let mut mesh = Mesh { no_vertices, no_faces, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: None, int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() };
        for _face in 0..no_faces {
            let v0 = mesh.create_vertex();
            let v1 = mesh.create_vertex();
            let v2 = mesh.create_vertex();
            mesh.create_face(&v0, &v1, &v2);
        }
        mesh.add_custom_vec3_attribute( "position", positions)?;
        Ok(mesh)
    }

    pub fn create_indexed(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = Mesh { no_vertices, no_faces, connectivity_info: Rc::new(ConnectivityInfo::new()), indices: Some(indices.clone()), int_attributes: Vec::new(), vec2_attributes: Vec::new(), vec3_attributes: Vec::new() };
        for _vertex in 0..no_vertices {
            mesh.create_vertex();
        }

        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);
            mesh.create_face(&v0, &v1, &v2);
        }
        mesh.add_custom_vec3_attribute( "position", positions)?;

        Ok(mesh)
    }

    fn create_vertex(&mut self) -> VertexID
    {
        self.connectivity_info.create_vertex()
    }

    fn connecting_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        for mut halfedge in self.vertex_halfedge_iterator(vertex_id1) {
            if &halfedge.vertex_id() == vertex_id2 {
                return Some(halfedge.halfedge_id())
            }
        }
        None
    }

    fn find_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        let mut walker = self.walker_from_halfedge(&HalfEdgeID::null());
        for halfedge_id in self.halfedge_iterator() {
            walker.jump_to_edge(&halfedge_id);
            if &walker.vertex_id() == vertex_id2 && &walker.twin().vertex_id() == vertex_id1
            {
                return Some(halfedge_id)
            }
        }
        None
    }

    fn create_face(&mut self, vertex_id1: &VertexID, vertex_id2: &VertexID, vertex_id3: &VertexID) -> FaceID
    {
        let id = self.connectivity_info.create_face();

        // Create inner halfedges
        let halfedge1 = match self.find_edge(vertex_id1, vertex_id2)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id2);
                    halfedge_id
                }
            };
        self.connectivity_info.set_face_halfedge(&id, &halfedge1);
        self.connectivity_info.set_vertex_halfedge(&vertex_id1, &halfedge1);
        self.connectivity_info.set_halfedge_face(&halfedge1, &id);

        let halfedge2 = match self.find_edge(vertex_id2, vertex_id3)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id3);
                    halfedge_id
                }
            };
        self.connectivity_info.set_vertex_halfedge(&vertex_id2, &halfedge2);
        self.connectivity_info.set_halfedge_next(&halfedge1, &halfedge2);
        self.connectivity_info.set_halfedge_face(&halfedge2, &id);

        let halfedge3 = match self.find_edge(vertex_id3, vertex_id1)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id1);
                    halfedge_id
                }
            };
        self.connectivity_info.set_vertex_halfedge(&vertex_id3, &halfedge3);
        self.connectivity_info.set_halfedge_next(&halfedge2, &halfedge3);
        self.connectivity_info.set_halfedge_next(&halfedge3, &halfedge1);
        self.connectivity_info.set_halfedge_face(&halfedge3, &id);

        // Create outer halfedges
        let halfedge4 = match self.find_edge(vertex_id2, vertex_id1)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id1);
                    halfedge_id
                }
            };
        self.connectivity_info.set_halfedge_twin(&halfedge1, &halfedge4);
        self.connectivity_info.set_halfedge_twin(&halfedge4, &halfedge1);

        let halfedge5 = match self.find_edge(vertex_id3, vertex_id2)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id2);
                    halfedge_id
                }
            };
        self.connectivity_info.set_halfedge_twin(&halfedge2, &halfedge5);
        self.connectivity_info.set_halfedge_twin(&halfedge5, &halfedge2);

        let halfedge6 = match self.find_edge(vertex_id1, vertex_id3)
            {
                Some(e) => { e },
                None => {
                    let halfedge_id = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id, &vertex_id3);
                    halfedge_id
                }
            };
        self.connectivity_info.set_halfedge_twin(&halfedge3, &halfedge6);
        self.connectivity_info.set_halfedge_twin(&halfedge6, &halfedge3);

        id
    }

    pub fn walker_from_vertex(&self, vertex_id: &VertexID) -> Walker
    {
        Walker::create_from_vertex(vertex_id, &self.connectivity_info)
    }

    pub fn walker_from_halfedge(&self, halfedge_id: &HalfEdgeID) -> Walker
    {
        Walker::create_from_halfedge(halfedge_id, &self.connectivity_info)
    }

    pub fn walker_from_face(&self, face_id: &FaceID) -> Walker
    {
        Walker::create_from_face(&face_id, &self.connectivity_info)
    }

    pub fn vertex_halfedge_iterator(&self, vertex_id: &VertexID) -> VertexHalfedgeIterator
    {
        VertexHalfedgeIterator::new(vertex_id, &self.connectivity_info)
    }

    pub fn face_halfedge_iterator(&self, face_id: &FaceID) -> FaceHalfedgeIterator
    {
        FaceHalfedgeIterator::new(face_id, &self.connectivity_info)
    }

    pub fn vertex_iterator(&self) -> VertexIterator
    {
        VertexIterator::new(&self.connectivity_info)
    }

    pub fn halfedge_iterator(&self) -> HalfEdgeIterator
    {
        HalfEdgeIterator::new(&self.connectivity_info)
    }

    pub fn face_iterator(&self) -> FaceIterator
    {
        FaceIterator::new(&self.connectivity_info)
    }

    pub fn get_vec2_attribute_names(&self) -> Vec<&str>
    {
        let mut names = Vec::new();
        for attribute in self.vec2_attributes.iter() {
            names.push(attribute.name());
        }
        names
    }

    pub fn get_vec3_attribute_names(&self) -> Vec<&str>
    {
        let mut names = Vec::new();
        names.push("position");
        for attribute in self.vec3_attributes.iter() {
            names.push(attribute.name());
        }
        names
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

    pub fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec2, Error>
    {
        for attribute in self.vec2_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute.at(vertex_id))
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), Error>
    {
        for attribute in self.vec2_attributes.iter_mut() {
            if attribute.name() == name
            {
                attribute.set(&vertex_id, &value);
                return Ok(())
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<Vec3, Error>
    {
        for attribute in self.vec3_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute.at(vertex_id))
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), Error>
    {
        for attribute in self.vec3_attributes.iter_mut() {
            if attribute.name() == name
            {
                attribute.set(&vertex_id, &value);
                return Ok(())
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn position_at(&self, vertex_id: &VertexID) -> Vec3
    {
        self.vec3_attributes.first().unwrap().at(vertex_id)
    }

    pub fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.vec3_attributes.first_mut().unwrap().set(vertex_id, value);
    }

    fn compute_face_normal(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = self.position_at(&walker.vertex_id());
        walker.next();
        let p1 = self.position_at(&walker.vertex_id());
        walker.next();
        let p2 = self.position_at(&walker.vertex_id());

        normalize(cross(p1 - p0, p2 - p0))
    }

    fn compute_vertex_normal(&self, vertex_id: &VertexID) -> Vec3
    {
        let mut normal = vec3(0.0, 0.0, 0.0);
        for walker in self.vertex_halfedge_iterator(&vertex_id) {
            let face_id = walker.face_id();
            if !face_id.is_null() {
                normal = normal + self.compute_face_normal(&face_id);
            }
        }
        normalize(normal)
    }

    pub fn update_normals(&mut self) -> Result<(), Error>
    {
        for vertex_id in self.vertex_iterator() {
            let normal = self.compute_vertex_normal(&vertex_id);
            self.set_vec3_attribute_at("normal", &vertex_id, &normal)?;
        }
        Ok(())
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

        let t1 = mesh.walker_from_vertex(&v1).halfedge_id();
        assert_eq!(t1.val(), 0);

        let t2 = mesh.walker_from_vertex(&v1).twin().halfedge_id();
        assert_eq!(t2.val(), 3);

        let t3 = mesh.walker_from_vertex(&v2).next().next().vertex_id();
        assert_eq!(t3.val(), v2.val());

        let t4 = mesh.walker_from_face(&f1).twin().face_id();
        assert!(t4.is_null());

        let t5 = mesh.walker_from_halfedge(&t1).twin().halfedge_id();
        assert_eq!(t5.val(), 3);
    }

    #[test]
    fn test_vertex_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for vertex_id in mesh.vertex_iterator() {
            assert_eq!(vertex_id.val(), i);
            i = i+1;
        }
        assert_eq!(4, i);
    }

    #[test]
    fn test_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for halfedge_id in mesh.halfedge_iterator() {
            assert_eq!(halfedge_id.val(), i);
            i = i+1;
        }
        assert_eq!(12, i);
    }

    #[test]
    fn test_face_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for face_id in mesh.face_iterator() {
            assert_eq!(face_id.val(), i);
            i = i+1;
        }
        assert_eq!(3, i);
    }

    #[test]
    fn test_connectivity() {
        let mesh = create_three_connected_faces();

        let mut walker = mesh.walker_from_vertex(&VertexID::new(0));
        let start_edge = walker.halfedge_id();
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin().halfedge_id();
        assert_eq!(start_edge.val(), one_round_edge.val());
    }

    #[test]
    fn test_vertex_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        let indices = vec![1, 2, 3];
        for edge in mesh.vertex_halfedge_iterator(&VertexID::new(0)) {
            assert_eq!(edge.vertex_id().val(), indices[i]);
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_vertex_halfedge_iterator_with_holes() {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let indices: Vec<u32> = vec![0, 2, 3,  0, 4, 1,  0, 1, 2];
        let mesh = Mesh::create_indexed(indices, positions).unwrap();

        let mut i = 0;
        let indices = vec![1, 2, 3, 4];
        for edge in mesh.vertex_halfedge_iterator(&VertexID::new(0)) {
            assert_eq!(edge.vertex_id().val(), indices[i]);
            i = i+1;
        }
        assert_eq!(i,4, "All edges of a one-ring are not visited");

    }

    #[test]
    fn test_face_halfedge_iterator() {
        let mesh = create_single_face();
        let mut i = 0;
        for mut edge in mesh.face_halfedge_iterator(&FaceID::new(0)) {
            assert_eq!(edge.halfedge_id().val(), i);
            assert_eq!(edge.face_id().val(), 0);
            i = i+1;
        }
        assert_eq!(i, 3, "All edges of a face are not visited");
    }

    #[test]
    fn test_face_normal() {
        let mesh = create_single_face();
        let computed_normal = mesh.compute_face_normal(&FaceID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }

    #[test]
    fn test_vertex_normal() {
        let mesh = create_three_connected_faces();
        let computed_normal = mesh.compute_vertex_normal(&VertexID::new(0));
        assert_eq!(0.0, computed_normal.x);
        assert_eq!(1.0, computed_normal.y);
        assert_eq!(0.0, computed_normal.z);
    }

    #[test]
    fn test_update_normals() {
        let mut mesh = create_three_connected_faces();
        let normals: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, 0.0,  0.0, 0.0, 0.0];
        mesh.add_custom_vec3_attribute("normal", normals).unwrap();
        mesh.update_normals().unwrap();

        for vertex_id in mesh.vertex_iterator() {
            let normal = mesh.get_vec3_attribute_at("normal", &vertex_id).unwrap();
            assert_eq!(0.0, normal.x);
            assert_eq!(1.0, normal.y);
            assert_eq!(0.0, normal.z);
        }
    }

    fn create_single_face() -> Mesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        let mut mesh = Mesh::create(positions).unwrap();

        let v0 = mesh.create_vertex();
        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        mesh.create_face(&v0, &v1, &v2);
        mesh
    }

    fn create_three_connected_faces() -> Mesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
        Mesh::create_indexed(indices, positions).unwrap()
    }

    fn create_connected_test_object() -> Mesh
    {
        let positions: Vec<f32> = vec![
            1.0, -1.0, -1.0,
            1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0,
            -1.0, -1.0, -1.0,
            1.0, 1.0, -1.0,
            1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0,
            -1.0, 1.0, -1.0
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
            4, 7, 6,
            4, 6, 5,
            0, 4, 5,
            0, 5, 1,
            1, 5, 6,
            1, 6, 2,
            2, 6, 7,
            2, 7, 3,
            4, 0, 3,
            4, 3, 7
        ];

        Mesh::create_indexed(indices, positions).unwrap()
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