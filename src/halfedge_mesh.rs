use mesh::{self, Mesh};
use attribute::VertexAttributes;
use connectivity_info::ConnectivityInfo;
use traversal::*;
use std::rc::Rc;
use std::collections::{HashSet, HashMap};
use ids::*;
use glm::*;

pub type HalfEdgeIterator = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIterator = Box<Iterator<Item = FaceID>>;

pub struct HalfEdgeMesh {
    indices: Vec<u32>,
    attributes: VertexAttributes,
    connectivity_info: Rc<ConnectivityInfo>
}

impl Mesh for HalfEdgeMesh
{
    fn no_vertices(&self) -> usize
    {
        self.connectivity_info.no_vertices()
    }

    fn no_faces(&self) -> usize
    {
        self.connectivity_info.no_faces()
    }

    fn indices(&self) -> &Vec<u32>
    {
        &self.indices
    }

    fn vertex_iterator(&self) -> mesh::VertexIterator
    {
        self.connectivity_info.vertex_iterator()
    }

    fn position_at(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.attributes.get_vec3_attribute_at("position", vertex_id).unwrap()
    }

    fn set_position_at(&mut self, vertex_id: &VertexID, value: &Vec3)
    {
        self.attributes.set_vec3_attribute_at("position", vertex_id, value).unwrap();
    }

    fn get_vec2_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec2, mesh::Error>
    {
        let val = self.attributes.get_vec2_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn set_vec2_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec2) -> Result<(), mesh::Error>
    {
        self.attributes.set_vec2_attribute_at(name, vertex_id, value)?;
        Ok(())
    }

    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, mesh::Error>
    {
        let val = self.attributes.get_vec3_attribute_at(name, vertex_id)?;
        Ok(val)
    }

    fn set_vec3_attribute_at(&mut self, name: &str, vertex_id: &VertexID, value: &Vec3) -> Result<(), mesh::Error>
    {
        self.attributes.set_vec3_attribute_at(name, vertex_id, value)?;
        Ok(())
    }
}

impl HalfEdgeMesh
{
    pub fn create(indices: Vec<u32>, vec3_attributes: HashMap<&str, Vec<f32>>) -> HalfEdgeMesh
    {
        let no_vertices = vec3_attributes.get("position").unwrap().len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = HalfEdgeMesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)), indices, attributes: VertexAttributes::new()};

        for attribute in vec3_attributes.iter() {
            mesh.attributes.create_vec3_attribute(attribute.0);
        }

        for i in 0..no_vertices {
            let vertex_id = mesh.create_vertex();
            for attribute in vec3_attributes.iter() {
                mesh.set_vec3_attribute_at(attribute.0, &vertex_id, &vec3(attribute.1[i*3], attribute.1[i*3+1], attribute.1[i*3+2]));
            }
        }

        for face in 0..no_faces {
            let v0 = VertexID::new(mesh.indices[face * 3] as usize);
            let v1 = VertexID::new(mesh.indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(mesh.indices[face * 3 + 2] as usize);
            mesh.create_face(&v0, &v1, &v2);
        }
        mesh.create_twin_connectivity();
        mesh
    }

    /*pub fn create_sub_mesh(&self, faces: &Vec<FaceID>) -> HalfEdgeMesh
    {
        let mut vertices = HashSet::new();

        for face_id in faces {
            for walker in self.face_halfedge_iterator(face_id) {
                let vertex_id = walker.vertex_id().unwrap();
                vertices.insert(vertex_id);
            }
        }

        let mut attributes = self.attributes.clone();
        for vertex_id in self.vertex_iterator() {
            if !vertices.contains(&vertex_id) {
                attributes.remove_vertex(&vertex_id);
            }
        }
        // TODO
        let indices = self.indices.clone();
        HalfEdgeMesh::create_from_other(vertices.len(), indices, attributes)
    }*/

    pub fn add_vec2_attribute(&mut self, name: &str)
    {
        self.attributes.create_vec2_attribute(name);
    }

    pub fn add_vec3_attribute(&mut self, name: &str)
    {
        self.attributes.create_vec3_attribute(name);
    }

    fn create_vertex(&mut self) -> VertexID
    {
        self.connectivity_info.create_vertex()
    }

    fn connecting_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        for mut halfedge in self.vertex_halfedge_iterator(vertex_id1) {
            if &halfedge.vertex_id().unwrap() == vertex_id2 {
                return halfedge.halfedge_id()
            }
        }
        None
    }

    fn find_edge(&self, vertex_id1: &VertexID, vertex_id2: &VertexID) -> Option<HalfEdgeID>
    {
        let mut walker = Walker::create(&self.connectivity_info);
        for halfedge_id in self.halfedge_iterator() {
            walker.jump_to_edge(&halfedge_id);
            if &walker.vertex_id().unwrap() == vertex_id2 && &walker.twin().vertex_id().unwrap() == vertex_id1
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
        let halfedge1 = self.connectivity_info.create_halfedge();
        self.connectivity_info.set_halfedge_vertex(&halfedge1, &vertex_id2);
        self.connectivity_info.set_vertex_halfedge(&vertex_id1, &halfedge1);
        self.connectivity_info.set_halfedge_face(&halfedge1, &id);
        self.connectivity_info.set_face_halfedge(&id, &halfedge1);

        let halfedge2 = self.connectivity_info.create_halfedge();
        self.connectivity_info.set_halfedge_vertex(&halfedge2, &vertex_id3);
        self.connectivity_info.set_vertex_halfedge(&vertex_id2, &halfedge2);
        self.connectivity_info.set_halfedge_next(&halfedge1, &halfedge2);
        self.connectivity_info.set_halfedge_face(&halfedge2, &id);

        let halfedge3 = self.connectivity_info.create_halfedge();
        self.connectivity_info.set_halfedge_vertex(&halfedge3, &vertex_id1);
        self.connectivity_info.set_vertex_halfedge(&vertex_id3, &halfedge3);
        self.connectivity_info.set_halfedge_next(&halfedge2, &halfedge3);
        self.connectivity_info.set_halfedge_next(&halfedge3, &halfedge1);
        self.connectivity_info.set_halfedge_face(&halfedge3, &id);

        id
    }

    fn create_twin_connectivity(&mut self)
    {
        let mut walker = Walker::create(&self.connectivity_info);

        for halfedge_id1 in self.halfedge_iterator()
        {
            let twin = walker.jump_to_edge(&halfedge_id1).twin().halfedge_id();

            if twin.is_none()
            {
                walker.jump_to_edge(&halfedge_id1);
                let vertex_id1 = walker.vertex_id().unwrap();
                let vertex_id2 = walker.previous().vertex_id().unwrap();

                let mut halfedge2 = None;
                for halfedge_id2 in self.halfedge_iterator() {
                    let twin = walker.jump_to_edge(&halfedge_id2).twin().halfedge_id();
                    if twin.is_none()
                    {
                        walker.jump_to_edge(&halfedge_id2);
                        if walker.vertex_id().unwrap() == vertex_id2 && walker.previous().vertex_id().unwrap() == vertex_id1
                            {
                                halfedge2 = Some(halfedge_id2);
                                break;
                            }
                    }
                }
                let halfedge_id2 = halfedge2.unwrap_or_else(|| {
                    let halfedge_id2 = self.connectivity_info.create_halfedge();
                    self.connectivity_info.set_halfedge_vertex(&halfedge_id2, &vertex_id2);
                    halfedge_id2}
                );
                self.connectivity_info.set_halfedge_twin(&halfedge_id1, &halfedge_id2);
                self.connectivity_info.set_halfedge_twin(&halfedge_id2, &halfedge_id1);

            }
        }
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

    pub fn halfedge_iterator(&self) -> HalfEdgeIterator
    {
        self.connectivity_info.halfedge_iterator()
    }

    pub fn face_iterator(&self) -> FaceIterator
    {
        self.connectivity_info.face_iterator()
    }

    pub fn compute_face_normal(&self, face_id: &FaceID) -> Vec3
    {
        let mut walker = self.walker_from_face(face_id);
        let p0 = *self.position_at(&walker.vertex_id().unwrap());
        walker.next();
        let p1 = *self.position_at(&walker.vertex_id().unwrap());
        walker.next();
        let p2 = *self.position_at(&walker.vertex_id().unwrap());

        normalize(cross(p1 - p0, p2 - p0))
    }

    pub fn compute_vertex_normal(&self, vertex_id: &VertexID) -> Vec3
    {
        let mut normal = vec3(0.0, 0.0, 0.0);
        for walker in self.vertex_halfedge_iterator(&vertex_id) {
            if let Some(face_id) = walker.face_id() {
                normal = normal + self.compute_face_normal(&face_id)
            }
        }
        normalize(normal)
    }
}

pub struct VertexHalfedgeIterator
{
    current: Walker,
    start: HalfEdgeID,
    is_done: bool
}

impl VertexHalfedgeIterator {
    pub fn new(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> VertexHalfedgeIterator
    {
        let current = Walker::create_from_vertex(vertex_id, connectivity_info);
        let start = current.halfedge_id().unwrap();
        VertexHalfedgeIterator { current, start, is_done: false }
    }
}

impl Iterator for VertexHalfedgeIterator {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();

        match self.current.face_id() {
            Some(_) => {
                self.current.previous().twin();
            },
            None => { // In the case there are holes in the one-ring
                self.current.twin();
                while let Some(_) = self.current.face_id() {
                    self.current.next().twin();
                }
                self.current.twin();
            }
        }
        self.is_done = self.current.halfedge_id().unwrap() == self.start;
        Some(curr)
    }
}

pub struct FaceHalfedgeIterator
{
    current: Walker,
    start: HalfEdgeID,
    is_done: bool
}

impl FaceHalfedgeIterator {
    pub fn new(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> FaceHalfedgeIterator
    {
        let current = Walker::create_from_face(face_id, connectivity_info);
        let start = current.halfedge_id().unwrap().clone();
        FaceHalfedgeIterator { current, start, is_done: false }
    }
}

impl Iterator for FaceHalfedgeIterator {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        self.current.next();
        self.is_done = self.current.halfedge_id().unwrap() == self.start;
        Some(curr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_mesh::SimpleMesh;

    #[test]
    fn test_create_face() {
        let mut attributes = HashMap::new();
        attributes.insert("position", vec![]);
        let mut mesh = HalfEdgeMesh::create(vec![], attributes);

        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        let v3 = mesh.create_vertex();
        let f1 = mesh.create_face(&v1, &v2, &v3);
        mesh.create_twin_connectivity();

        let t1 = mesh.walker_from_vertex(&v1).halfedge_id();
        assert!(t1.is_some());

        let t2 = mesh.walker_from_vertex(&v1).twin().halfedge_id();
        assert!(t2.is_some());

        let t3 = mesh.walker_from_vertex(&v2).next().next().vertex_id();
        assert!(t3.is_some());
        assert_eq!(t3.unwrap(), v2);

        let t4 = mesh.walker_from_face(&f1).twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_halfedge(&t1.unwrap()).twin().halfedge_id();
        assert!(t5.is_some());

        let t6 = mesh.walker_from_vertex(&v3).face_id();
        assert!(t6.is_some());
        assert_eq!(t6.unwrap(), f1);
    }

    #[test]
    fn test_vertex_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.vertex_iterator() {
            i = i+1;
        }
        assert_eq!(4, i);

        // Test that two iterations return the same result
        let vec: Vec<VertexID> = mesh.vertex_iterator().collect();
        i = 0;
        for vertex_id in mesh.vertex_iterator() {
            assert_eq!(vertex_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.halfedge_iterator() {
            i = i+1;
        }
        assert_eq!(12, i);

        // Test that two iterations return the same result
        let vec: Vec<HalfEdgeID> = mesh.halfedge_iterator().collect();
        i = 0;
        for halfedge_id in mesh.halfedge_iterator() {
            assert_eq!(halfedge_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_face_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        for _ in mesh.face_iterator() {
            i = i+1;
        }
        assert_eq!(3, i);

        // Test that two iterations return the same result
        let vec: Vec<FaceID> = mesh.face_iterator().collect();
        i = 0;
        for face_id in mesh.face_iterator() {
            assert_eq!(face_id, vec[i]);
            i = i+1;
        }
    }

    #[test]
    fn test_connectivity() {
        let mesh = create_three_connected_faces();
        let mut id = None;
        for vertex_id in mesh.vertex_iterator() {
            let mut round = true;
            for walker in mesh.vertex_halfedge_iterator(&vertex_id) {
                if walker.face_id().is_none() { round = false; break; }
            }
            if(round) { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(&id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin().halfedge_id().unwrap();
        assert_eq!(start_edge, one_round_edge);
    }

    #[test]
    fn test_vertex_halfedge_iterator() {
        let mesh = create_three_connected_faces();

        let mut i = 0;
        let vertex_id = mesh.vertex_iterator().last().unwrap();
        for edge in mesh.vertex_halfedge_iterator(&vertex_id) {
            assert!(edge.vertex_id().is_some());
            i = i + 1;
        }
        assert_eq!(i, 3, "All edges of a one-ring are not visited");
    }

    #[test]
    fn test_vertex_halfedge_iterator_with_holes() {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut attributes = HashMap::new();
        attributes.insert("position", positions);
        let indices: Vec<u32> = vec![0, 2, 3,  0, 4, 1,  0, 1, 2];
        let mesh = HalfEdgeMesh::create(indices, attributes);

        let mut i = 0;
        for edge in mesh.vertex_halfedge_iterator(&VertexID::new(0)) {
            assert!(edge.vertex_id().is_some());
            i = i+1;
        }
        assert_eq!(i,4, "All edges of a one-ring are not visited");

    }

    #[test]
    fn test_face_halfedge_iterator() {
        let mesh = create_single_face();
        let mut i = 0;
        for mut edge in mesh.face_halfedge_iterator(&FaceID::new(0)) {
            assert!(edge.halfedge_id().is_some());
            assert!(edge.face_id().is_some());
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
        mesh.add_vec3_attribute("normal");
        for vertex_id in mesh.vertex_iterator() {
            mesh.set_vec3_attribute_at("normal", &vertex_id, &vec3(0.0, 0.0, 0.0)).unwrap();
        }

        for vertex_id in mesh.vertex_iterator() {
            let normal = mesh.compute_vertex_normal(&vertex_id);
            mesh.set_vec3_attribute_at("normal", &vertex_id, &normal).unwrap();
        }

        for vertex_id in mesh.vertex_iterator() {
            let normal = mesh.get_vec3_attribute_at("normal", &vertex_id).unwrap();
            assert_eq!(0.0, normal.x);
            assert_eq!(1.0, normal.y);
            assert_eq!(0.0, normal.z);
        }
    }

    fn create_single_face() -> HalfEdgeMesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        let indices = (0..positions.len() as u32/3).collect();
        let mut attributes = HashMap::new();
        attributes.insert("position", positions);
        let mut mesh = HalfEdgeMesh::create(indices, attributes);

        let v0 = mesh.create_vertex();
        let v1 = mesh.create_vertex();
        let v2 = mesh.create_vertex();
        mesh.create_face(&v0, &v1, &v2);
        mesh
    }

    fn create_three_connected_faces() -> HalfEdgeMesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
        let mut attributes = HashMap::new();
        attributes.insert("position", positions);
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
        HalfEdgeMesh::create(indices, attributes)
    }

    /*fn create_connected_test_object() -> HalfEdgeMesh
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

        HalfEdgeMesh::create(indices, positions).unwrap()
    }

    fn create_test_object() -> HalfEdgeMesh
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

        let mut mesh = HalfEdgeMesh::create((0..positions.len() as u32/3).collect(), positions).unwrap();
        mesh.add_vec3_attribute("normal", normals).unwrap();
        mesh.add_vec2_attribute("uv_coordinate", uvs).unwrap();
        mesh
    }*/
}