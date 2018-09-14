use mesh::{self, Error, Renderable};
use connectivity_info::ConnectivityInfo;
use traversal::*;
use std::rc::Rc;
use std::collections::{HashSet, HashMap};
use ids::*;
use glm::*;

pub type HalfEdgeIterator = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIterator = Box<Iterator<Item = FaceID>>;

pub struct DynamicMesh {
    positions: HashMap<VertexID, Vec3>,
    normals: HashMap<VertexID, Vec3>,
    connectivity_info: Rc<ConnectivityInfo>
}

impl Renderable for DynamicMesh
{
    fn indices(&self) -> Vec<u32>
    {
        let vertices: Vec<VertexID> = self.vertex_iterator().collect();
        let mut indices = Vec::with_capacity(self.no_faces() * 3);
        for face_id in self.face_iterator()
        {
            for walker in self.face_halfedge_iterator(&face_id) {
                let vertex_id = walker.vertex_id().unwrap();
                let index = vertices.iter().position(|v| v == &vertex_id).unwrap();
                indices.push(index as u32);
            }
        }
        indices
    }

    fn vertex_iterator(&self) -> mesh::VertexIterator
    {
        self.connectivity_info.vertex_iterator()
    }

    fn get_vec2_attribute_at(&self, name: &str, _vertex_id: &VertexID) -> Result<&Vec2, Error>
    {
        panic!("Half edge meshes does not contain {}, only positions and normals", name);
    }

    fn get_vec3_attribute_at(&self, name: &str, vertex_id: &VertexID) -> Result<&Vec3, Error>
    {
        Ok(match name {
            "position" => self.position(vertex_id),
            "normal" => self.normal(vertex_id),
            _ => panic!("Half edge meshes does not contain {}, only positions and normals", name)
        })
    }

    fn no_vertices(&self) -> usize
    {
        self.no_vertices()
    }
}

impl DynamicMesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>, normals: Option<Vec<f32>>) -> DynamicMesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = DynamicMesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)),
            positions: HashMap::new(), normals: HashMap::new()};

        for i in 0..no_vertices {
            let vertex_id = mesh.create_vertex(vec3(positions[i*3], positions[i*3+1], positions[i*3+2]));
            if let Some(ref data) = normals {
                mesh.normals.insert(vertex_id, vec3(data[i*3], data[i*3+1], data[i*3+2]));
            }
        }

        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);
            mesh.create_face(&v0, &v1, &v2);
        }
        mesh.create_twin_connectivity();
        mesh
    }

    pub fn no_vertices(&self) -> usize
    {
        self.connectivity_info.no_vertices()
    }

    pub fn no_halfedges(&self) -> usize
    {
        self.connectivity_info.no_halfedges()
    }

    pub fn no_faces(&self) -> usize
    {
        self.connectivity_info.no_faces()
    }

    pub fn create_sub_mesh(&self, faces: &HashSet<FaceID>) -> DynamicMesh
    {
        let mut submesh = DynamicMesh {positions: self.positions.clone(), normals: self.normals.clone(),
            connectivity_info: Rc::new((*self.connectivity_info).clone())};

        let current_faces: Vec<FaceID> = self.face_iterator().collect();
        for face_id in current_faces.iter() {
            if !faces.contains(face_id) {
                submesh.remove_face(face_id);
            }
        }

        submesh
    }

    pub fn position(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.positions.get(vertex_id).unwrap()
    }

    pub fn set_position(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.positions.insert(vertex_id, value);
    }

    pub fn normal(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.normals.get(vertex_id).unwrap()
    }

    pub fn set_normal(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.normals.insert(vertex_id, value);
    }

    fn create_vertex(&mut self, position: Vec3) -> VertexID
    {
        let id = self.connectivity_info.create_vertex();
        self.positions.insert(id.clone(), position);
        id
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
        let halfedge1 = self.connectivity_info.create_halfedge(Some(vertex_id2.clone()), None, Some(id.clone()));
        let halfedge3 = self.connectivity_info.create_halfedge(Some(vertex_id1.clone()), Some(halfedge1.clone()),Some(id.clone()));
        let halfedge2 = self.connectivity_info.create_halfedge(Some(vertex_id3.clone()), Some(halfedge3.clone()),Some(id.clone()));

        self.connectivity_info.set_halfedge_next(&halfedge1, &halfedge2);

        self.connectivity_info.set_vertex_halfedge(&vertex_id1, &halfedge1);
        self.connectivity_info.set_vertex_halfedge(&vertex_id2, &halfedge2);
        self.connectivity_info.set_vertex_halfedge(&vertex_id3, &halfedge3);

        self.connectivity_info.set_face_halfedge(&id, &halfedge1);

        id
    }

    fn create_twin_connectivity(&mut self)
    {
        let mut walker = Walker::create(&self.connectivity_info);
        let edges: Vec<HalfEdgeID> = self.halfedge_iterator().collect();

        for halfedge_id1 in self.halfedge_iterator()
        {
            let twin = walker.jump_to_edge(&halfedge_id1).twin().halfedge_id();

            if twin.is_none()
            {
                walker.jump_to_edge(&halfedge_id1);
                let vertex_id1 = walker.vertex_id().unwrap();
                let vertex_id2 = walker.previous().vertex_id().unwrap();

                let mut halfedge2 = None;
                for halfedge_id2 in edges.iter() {
                    let twin = walker.jump_to_edge(halfedge_id2).twin().halfedge_id();
                    if twin.is_none()
                    {
                        walker.jump_to_edge(halfedge_id2);
                        if walker.vertex_id().unwrap() == vertex_id2 && walker.previous().vertex_id().unwrap() == vertex_id1
                            {
                                halfedge2 = Some(halfedge_id2.clone());
                                break;
                            }
                    }
                }
                let halfedge_id2 = halfedge2.unwrap_or_else(|| {
                    self.connectivity_info.create_halfedge(Some(vertex_id2), None,None)
                }
                );
                self.connectivity_info.set_halfedge_twin(&halfedge_id1, &halfedge_id2);
                self.connectivity_info.set_halfedge_twin(&halfedge_id2, &halfedge_id1);

            }
        }
    }

    pub fn remove_face(&mut self, face_id: &FaceID)
    {
        self.connectivity_info.remove_face(face_id);
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
        let p0 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p1 = *self.position(&walker.vertex_id().unwrap());
        walker.next();
        let p2 = *self.position(&walker.vertex_id().unwrap());

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

    pub fn update_vertex_normals(&mut self)
    {
        for vertex_id in self.vertex_iterator() {
            let normal = self.compute_vertex_normal(&vertex_id);
            self.set_normal(vertex_id, normal);
        }
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

    #[test]
    fn test_one_face_connectivity() {
        let mut mesh = DynamicMesh::create(vec![], vec![], None);

        let v1 = mesh.create_vertex(vec3(0.0, 0.0, 0.0));
        let v2 = mesh.create_vertex(vec3(0.0, 0.0, 0.0));
        let v3 = mesh.create_vertex(vec3(0.0, 0.0, 0.0));
        let f1 = mesh.create_face(&v1, &v2, &v3);
        mesh.create_twin_connectivity();

        let t1 = mesh.walker_from_vertex(&v1).vertex_id();
        assert_eq!(t1, Some(v2.clone()));

        let t2 = mesh.walker_from_vertex(&v1).twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(&v2.clone()).next().next().vertex_id();
        assert_eq!(t3, Some(v2.clone()));

        let t4 = mesh.walker_from_face(&f1.clone()).twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(&f1.clone()).twin().next().halfedge_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(&f1.clone()).previous().previous().twin().twin().face_id();
        assert_eq!(t6, Some(f1.clone()));

        let t7 = mesh.walker_from_vertex(&v2.clone()).next().next().next().halfedge_id();
        assert_eq!(t7, mesh.walker_from_vertex(&v2).halfedge_id());

        let t8 = mesh.walker_from_vertex(&v3).face_id();
        assert_eq!(t8, Some(f1));
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = create_three_connected_faces();
        let mut id = None;
        for vertex_id in mesh.vertex_iterator() {
            let mut round = true;
            for walker in mesh.vertex_halfedge_iterator(&vertex_id) {
                if walker.face_id().is_none() { round = false; break; }
            }
            if round { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(&id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin().halfedge_id().unwrap();
        assert_eq!(start_edge, one_round_edge);
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
        let indices: Vec<u32> = vec![0, 2, 3,  0, 4, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0; 5 * 3];
        let mesh = DynamicMesh::create(indices, positions, None);

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
        mesh.update_vertex_normals();

        for vertex_id in mesh.vertex_iterator() {
            let normal = mesh.normal(&vertex_id);
            assert_eq!(0.0, normal.x);
            assert_eq!(1.0, normal.y);
            assert_eq!(0.0, normal.z);
        }
    }

    #[test]
    fn test_remove_face()
    {
        let mut mesh = create_connected_box();
        let face_id = mesh.face_iterator().next().unwrap();
        mesh.remove_face(&face_id);

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(11, mesh.no_faces());

        let mut i = 0;
        for face_id in mesh.face_iterator()
        {
            mesh.remove_face(&face_id);
            i = i+1;
        }
        assert_eq!(i, 11);
        assert_eq!(0, mesh.no_vertices());
        assert_eq!(0, mesh.no_halfedges());
        assert_eq!(0, mesh.no_faces());
    }

    fn create_single_face() -> DynamicMesh
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.0];
        DynamicMesh::create((0..3).collect(), positions, None)
    }

    fn create_three_connected_faces() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 2, 3,  0, 3, 1,  0, 1, 2];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5];
        let normals: Vec<f32> = vec![0.0; 4 * 3];
        DynamicMesh::create(indices, positions, Some(normals))
    }

    fn create_connected_box() -> DynamicMesh
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

        DynamicMesh::create(indices, positions, None)
    }
}