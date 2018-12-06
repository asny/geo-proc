use dynamic_mesh::connectivity_info::ConnectivityInfo;
use dynamic_mesh::*;
use types::*;
use std::rc::Rc;
use std::collections::{HashSet, HashMap};

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
pub type HalfEdgeIterator = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIterator = Box<Iterator<Item = FaceID>>;
pub type EdgeIterator = Box<Iterator<Item = (VertexID, VertexID)>>;

#[derive(Debug)]
pub struct DynamicMesh {
    positions: HashMap<VertexID, Vec3>,
    normals: HashMap<VertexID, Vec3>,
    pub(super) connectivity_info: Rc<ConnectivityInfo>
}

impl DynamicMesh
{
    pub fn new(positions: Vec<f32>, normals: Option<Vec<f32>>) -> DynamicMesh
    {
        let mut indices = vec![None; positions.len()/3];
        let mut positions_out = Vec::new();
        let mut normals_out = if normals.is_some() { Some(Vec::new()) } else { None };

        for i in 0..positions.len()/3 {
            if indices[i].is_none()
            {
                let p1 = vec3(positions[3 * i], positions[3 * i + 1], positions[3 * i + 2]);
                positions_out.push(p1.x);
                positions_out.push(p1.y);
                positions_out.push(p1.z);

                if let Some(ref n_in) = normals {
                    if let Some(ref mut n_out) = normals_out {
                        let n = vec3(n_in[3 * i], n_in[3 * i + 1], n_in[3 * i + 2]);
                        n_out.push(n.x);
                        n_out.push(n.y);
                        n_out.push(n.z);
                    }
                }

                let current_index = Some((positions_out.len() / 3 - 1) as u32);
                indices[i] = current_index;
                for j in i+1..positions.len()/3 {
                    let p2 = vec3(positions[3 * j], positions[3 * j + 1], positions[3 * j + 2]);
                    if (p1 - p2).norm() < 0.00001 {
                        indices[j] = current_index;
                    }
                }
            }
        }

        DynamicMesh::new_with_connectivity(indices.iter().map(|x| x.unwrap()).collect(), positions_out, normals_out)
    }

    pub fn new_with_connectivity(indices: Vec<u32>, positions: Vec<f32>, normals: Option<Vec<f32>>) -> DynamicMesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = DynamicMesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)),
            positions: HashMap::new(), normals: HashMap::new()};

        for i in 0..no_vertices {
            let nor = match normals { Some(ref data) => Some(vec3(data[i*3], data[i*3+1], data[i*3+2])), None => None };
            mesh.create_vertex(vec3(positions[i*3], positions[i*3+1], positions[i*3+2]), nor);
        }

        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);
            mesh.connectivity_info.create_face(&v0, &v1, &v2);
        }
        mesh.create_twin_connectivity();
        mesh
    }

    pub(super) fn new_internal(positions: HashMap<VertexID, Vec3>, normals: HashMap<VertexID, Vec3>, connectivity_info: Rc<ConnectivityInfo>) -> DynamicMesh
    {
        DynamicMesh {positions, normals, connectivity_info}
    }

    fn find_overlapping_vertices(&self) -> Vec<Vec<VertexID>>
    {
        let mut to_check = HashSet::new();
        self.vertex_iterator().for_each(|v| { to_check.insert(v); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            let mut to_merge = Vec::new();
            for id2 in to_check.iter()
            {
                if (self.position(&id1) - self.position(id2)).norm() < 0.00001
                {
                    to_merge.push(*id2);
                }
            }
            if !to_merge.is_empty()
            {
                for id in to_merge.iter()
                {
                    to_check.remove(id);
                }
                to_merge.push(id1);
                set_to_merge.push(to_merge);
            }
        }
        set_to_merge
    }

    fn find_overlapping_faces(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<FaceID>>
    {
        let vertices_to_merge = |vertex_id| {set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))};

        let mut set_of_faces_to_merge = Vec::new();
        for face_id1 in self.face_iterator() {
            let (v0, v1, v2) = self.face_vertices(&face_id1);
            if let Some(vertices_to_merge0) = vertices_to_merge(v0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(v1)
                {
                    if let Some(vertices_to_merge2) = vertices_to_merge(v2)
                    {
                        let mut faces_to_merge = Vec::new();
                        faces_to_merge.push(face_id1);
                        for face_id2 in self.face_iterator() {
                            if face_id1 < face_id2 {
                                let mut is_overlapping = true;
                                for walker in self.face_halfedge_iterator(&face_id2) {
                                    let vid = walker.vertex_id().unwrap();
                                    if !vertices_to_merge0.contains(&vid) && !vertices_to_merge1.contains(&vid) && !vertices_to_merge2.contains(&vid)
                                    {
                                        is_overlapping = false;
                                    }
                                }
                                if is_overlapping { faces_to_merge.push(face_id2) }
                            }
                        }
                        if faces_to_merge.len() > 1 {
                            set_of_faces_to_merge.push(faces_to_merge);
                        }
                    }
                }
            }
        }
        set_of_faces_to_merge
    }

    fn find_overlapping_edges(&self, set_of_vertices_to_merge: &Vec<Vec<VertexID>>) -> Vec<Vec<HalfEdgeID>>
    {
        let vertices_to_merge = |vertex_id| {
            set_of_vertices_to_merge.iter().find(|vec| vec.contains(&vertex_id))
        };
        let mut to_check = HashSet::new();
        self.edge_iterator().for_each(|e| { to_check.insert(e); } );

        let mut set_to_merge = Vec::new();
        while !to_check.is_empty() {
            let id1 = *to_check.iter().next().unwrap();
            to_check.remove(&id1);

            if let Some(vertices_to_merge0) = vertices_to_merge(id1.0)
            {
                if let Some(vertices_to_merge1) = vertices_to_merge(id1.1)
                {
                    let mut to_merge = Vec::new();
                    for id2 in to_check.iter()
                    {
                        if vertices_to_merge0.contains(&id2.0) && vertices_to_merge1.contains(&id2.1)
                            || vertices_to_merge1.contains(&id2.0) && vertices_to_merge0.contains(&id2.1)
                        {
                            to_merge.push(self.connecting_edge(&id2.0, &id2.1).unwrap());
                        }
                    }
                    if !to_merge.is_empty()
                    {
                        for id in to_merge.iter()
                        {
                            to_check.remove(&self.ordered_edge_vertices(id));
                        }
                        to_merge.push(self.connecting_edge(&id1.0, &id1.1).unwrap());
                        set_to_merge.push(to_merge);
                    }
                }
            }
        }
        set_to_merge
    }

    pub fn merge_overlapping_primitives(&mut self) -> Result<(), basic_operations::Error>
    {
        let set_of_vertices_to_merge = self.find_overlapping_vertices();
        let set_of_edges_to_merge = self.find_overlapping_edges(&set_of_vertices_to_merge);
        let set_of_faces_to_merge = self.find_overlapping_faces(&set_of_vertices_to_merge);

        for faces_to_merge in set_of_faces_to_merge {
            let mut iter = faces_to_merge.iter();
            let mut face_id1 = *iter.next().unwrap();
            for face_id2 in iter {
                self.remove_face(&face_id2);
            }
        }

        for vertices_to_merge in set_of_vertices_to_merge {
            let mut iter = vertices_to_merge.iter();
            let mut vertex_id1 = *iter.next().unwrap();
            for vertex_id2 in iter {
                vertex_id1 = self.merge_vertices(&vertex_id1, vertex_id2)?;
            }
        }

        for edges_to_merge in set_of_edges_to_merge {
            let mut iter = edges_to_merge.iter();
            let mut edge_id1 = *iter.next().unwrap();
            for edge_id2 in iter {
                edge_id1 = self.merge_halfedges(&edge_id1, edge_id2)?;
            }
        }

        Ok(())
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

    pub fn flip_orientation(&mut self)
    {
        for vertex_id in self.vertex_iterator() {
            let twin_id = self.walker_from_vertex(&vertex_id).twin_id().unwrap();
            self.connectivity_info.set_vertex_halfedge(&vertex_id, twin_id);
        }

        let mut map = HashMap::new();
        for halfedge_id in self.halfedge_iterator() {
            let mut walker = self.walker_from_halfedge(&halfedge_id);
            let new_next_id = walker.previous_id();
            let new_vertex_id = walker.twin().vertex_id().unwrap();
            map.insert(halfedge_id, (new_vertex_id, new_next_id));
        }
        for (halfedge_id, (new_vertex_id, new_next_id)) in map {
            self.connectivity_info.set_halfedge_vertex(&halfedge_id, new_vertex_id);
            if let Some(next_id) = new_next_id {
                self.connectivity_info.set_halfedge_next(&halfedge_id, Some(next_id));
            }
        }
    }

    ////////////////////////////////////////////
    // *** Functions related to the position ***
    ////////////////////////////////////////////

    pub fn position(&self, vertex_id: &VertexID) -> &Vec3
    {
        self.positions.get(vertex_id).unwrap()
    }

    pub fn set_position(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.positions.insert(vertex_id, value);
    }

    pub fn move_vertex(&mut self, vertex_id: VertexID, value: Vec3)
    {
        let mut p = value;
        {
            p = p + *self.positions.get(&vertex_id).unwrap();
        }
        self.positions.insert(vertex_id, p);
    }

    pub fn scale(&mut self, scale: f32)
    {
        for vertex_id in self.vertex_iterator() {
            let p = *self.position(&vertex_id);
            self.set_position(vertex_id, p * scale);
        }
    }

    pub fn translate(&mut self, translation: &Vec3)
    {
        for vertex_id in self.vertex_iterator() {
            self.move_vertex(vertex_id, *translation);
        }
    }

    pub fn edge_positions(&self, halfedge_id: &HalfEdgeID) -> (&Vec3, &Vec3)
    {
        let vertices = self.ordered_edge_vertices(halfedge_id);
        (self.position(&vertices.0), self.position(&vertices.1))
    }

    pub fn face_positions(&self, face_id: &FaceID) -> (&Vec3, &Vec3, &Vec3)
    {
        let vertices = self.ordered_face_vertices(face_id);
        (self.position(&vertices.0), self.position(&vertices.1), self.position(&vertices.2))
    }

    //////////////////////////////////////////
    // *** Functions related to the normal ***
    //////////////////////////////////////////

    pub fn normal(&self, vertex_id: &VertexID) ->  Option<&Vec3>
    {
        self.normals.get(vertex_id)
    }

    pub fn set_normal(&mut self, vertex_id: VertexID, value: Vec3)
    {
        self.normals.insert(vertex_id, value);
    }

    pub fn compute_vertex_normal(&self, vertex_id: &VertexID) -> Vec3
    {
        let mut normal = vec3(0.0, 0.0, 0.0);
        for walker in self.vertex_halfedge_iterator(&vertex_id) {
            if let Some(face_id) = walker.face_id() {
                normal = normal + self.face_normal(&face_id)
            }
        }
        normal.normalize_mut();
        normal
    }

    pub fn update_vertex_normals(&mut self)
    {
        for vertex_id in self.vertex_iterator() {
            let normal = self.compute_vertex_normal(&vertex_id);
            self.set_normal(vertex_id, normal);
        }
    }

    ///////////////////////////////////////////////////
    // *** Internal connectivity changing functions ***
    ///////////////////////////////////////////////////

    pub(super) fn create_vertex(&mut self, position: Vec3, normal: Option<Vec3>) -> VertexID
    {
        let id = self.connectivity_info.new_vertex();
        self.positions.insert(id.clone(), position);
        if let Some(nor) = normal {self.normals.insert(id.clone(), nor);}
        id
    }

    pub(super) fn create_twin_connectivity(&mut self)
    {
        let mut walker = Walker::create(&self.connectivity_info);
        let edges: Vec<HalfEdgeID> = self.halfedge_iterator().collect();

        for i1 in 0..edges.len()
        {
            let halfedge_id1 = edges[i1];
            if walker.jump_to_edge(&halfedge_id1).twin_id().is_none()
            {
                let vertex_id1 = walker.vertex_id().unwrap();
                let vertex_id2 = walker.previous().vertex_id().unwrap();

                let mut halfedge2 = None;
                for i2 in i1+1..edges.len()
                {
                    let halfedge_id2 = &edges[i2];
                    if walker.jump_to_edge(halfedge_id2).twin_id().is_none()
                    {
                        if walker.vertex_id().unwrap() == vertex_id2 && walker.previous().vertex_id().unwrap() == vertex_id1
                        {
                            halfedge2 = Some(halfedge_id2.clone());
                            break;
                        }
                    }
                }
                let halfedge_id2 = halfedge2.unwrap_or_else(|| {
                        self.connectivity_info.new_halfedge(Some(vertex_id2), None, None)
                    });
                self.connectivity_info.set_halfedge_twin(halfedge_id1, halfedge_id2);

            }
        }
    }
}

impl Clone for DynamicMesh {
    fn clone(&self) -> DynamicMesh {
        DynamicMesh::new_internal(self.positions.clone(), self.normals.clone(), Rc::new((*self.connectivity_info).clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dynamic_mesh::test_utility::*;

    #[test]
    fn test_one_face_connectivity() {
        let mut mesh = DynamicMesh::new_with_connectivity(vec![], vec![], None);

        let v1 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v2 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v3 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let f1 = mesh.connectivity_info.create_face(&v1, &v2, &v3);
        mesh.create_twin_connectivity();

        let t1 = mesh.walker_from_vertex(&v1).vertex_id();
        assert_eq!(t1, Some(v2.clone()));

        let t2 = mesh.walker_from_vertex(&v1).twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(&v2.clone()).next().next().vertex_id();
        assert_eq!(t3, Some(v2.clone()));

        let t4 = mesh.walker_from_face(&f1.clone()).twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(&f1.clone()).twin().next_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(&f1.clone()).previous().previous().twin().twin().face_id();
        assert_eq!(t6, Some(f1.clone()));

        let t7 = mesh.walker_from_vertex(&v2.clone()).next().next().next_id();
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
        let one_round_edge = walker.previous().twin().previous().twin().previous().twin_id().unwrap();
        assert_eq!(start_edge, one_round_edge);
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
            let normal = mesh.normal(&vertex_id).unwrap();
            assert_eq!(0.0, normal.x);
            assert_eq!(1.0, normal.y);
            assert_eq!(0.0, normal.z);
        }
    }

    #[test]
    fn test_new_from_positions()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let mesh = DynamicMesh::new(positions, None);

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let mut mesh = DynamicMesh::new_with_connectivity((0..9).collect(), positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(12, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_primitives_of_cube()
    {
        let mut mesh = ::models::create_unconnected_cube().unwrap().to_dynamic();
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(8, mesh.no_vertices());
        assert_eq!(36, mesh.no_halfedges());
        assert_eq!(12, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_individual_faces()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];

        let mut mesh = DynamicMesh::new_with_connectivity((0..9).collect(), positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(10, mesh.no_halfedges());
        assert_eq!(2, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_merge_overlapping_faces()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 3, 2,  4, 6, 5,  6, 7, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,
                                       -1.0, 0.0, 0.0,  -0.5, 0.0, 1.0,  -1.5, 0.0, 1.0,  -1.0, 0.0, 1.5];

        let mut mesh = DynamicMesh::new_with_connectivity(indices, positions, None);
        mesh.merge_overlapping_primitives().unwrap();

        assert_eq!(5, mesh.no_vertices());
        assert_eq!(14, mesh.no_halfedges());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }
}