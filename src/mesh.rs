
pub mod math {
    use cgmath::{Vector2, Vector3};
    pub use cgmath::prelude::*;

    pub type Vec2 = Vector2<f32>;
    pub type Vec3 = Vector3<f32>;

    pub fn vec2(x: f32, y: f32) -> Vec2
    {
        Vector2::new(x, y)
    }
    pub fn vec3(x: f32, y: f32, z: f32) -> Vec3
    {
        Vector3::new(x, y, z)
    }
}

pub mod ids;
pub mod traversal;
pub mod merge_overlapping_primitives;
pub mod connectivity;
pub mod basic_operations;
pub mod quality;
pub mod edge_measures;
pub mod face_measures;

mod connectivity_info;

use crate::mesh::connectivity_info::ConnectivityInfo;
use std::rc::Rc;
use std::collections::HashMap;
use crate::mesh::ids::*;
use crate::mesh::math::*;

#[derive(Debug)]
pub struct Mesh {
    positions: HashMap<VertexID, Vec3>,
    normals: HashMap<VertexID, Vec3>,
    connectivity_info: Rc<ConnectivityInfo>
}

impl Mesh
{
    fn new(positions: Vec<f32>, normals: Option<Vec<f32>>) -> Mesh
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
                    if (p1 - p2).magnitude() < 0.00001 {
                        indices[j] = current_index;
                    }
                }
            }
        }

        Mesh::new_with_connectivity(indices.iter().map(|x| x.unwrap()).collect(), positions_out, normals_out)
    }

    pub(crate) fn new_with_connectivity(indices: Vec<u32>, positions: Vec<f32>, normals: Option<Vec<f32>>) -> Mesh
    {
        let no_vertices = positions.len()/3;
        let no_faces = indices.len()/3;
        let mut mesh = Mesh { connectivity_info: Rc::new(ConnectivityInfo::new(no_vertices, no_faces)),
            positions: HashMap::new(), normals: HashMap::new()};

        // Create vertices
        for i in 0..no_vertices {
            let nor = match normals { Some(ref data) => Some(vec3(data[i*3], data[i*3+1], data[i*3+2])), None => None };
            mesh.create_vertex(vec3(positions[i*3], positions[i*3+1], positions[i*3+2]), nor);
        }

        // Create faces and twin connectivity
        let mut walker = mesh.walker();
        for face in 0..no_faces {
            let v0 = VertexID::new(indices[face * 3] as usize);
            let v1 = VertexID::new(indices[face * 3 + 1] as usize);
            let v2 = VertexID::new(indices[face * 3 + 2] as usize);

            let face_id = mesh.connectivity_info.create_face(&v0, &v1, &v2);

            for twin_id in mesh.halfedge_iter() {
                walker.as_halfedge_walker(&twin_id);
                if walker.twin_id().is_none() && walker.face_id().unwrap() != face_id {
                    let vertex_id0 = walker.vertex_id().unwrap();
                    let vertex_id1 = walker.as_previous().vertex_id().unwrap();

                    if vertex_id0 == v0 && vertex_id1 == v1 || vertex_id0 == v1 && vertex_id1 == v0 {
                        let halfedge_id = mesh.walker_from_face(&face_id).halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v1 && vertex_id1 == v2 || vertex_id0 == v2 && vertex_id1 == v1 {
                        let halfedge_id = mesh.walker_from_face(&face_id).as_next().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                    if vertex_id0 == v2 && vertex_id1 == v0 || vertex_id0 == v0 && vertex_id1 == v2 {
                        let halfedge_id = mesh.walker_from_face(&face_id).as_previous().halfedge_id().unwrap();
                        mesh.connectivity_info.set_halfedge_twin(halfedge_id, twin_id);
                    }
                }
            }
        }

        // Create boundary halfedges
        let mut walker = mesh.walker();
        for halfedge_id in mesh.halfedge_iter()
        {
            walker.as_halfedge_walker(&halfedge_id);
            if walker.twin_id().is_none()
            {
                let boundary_halfedge_id = mesh.connectivity_info.new_halfedge(walker.as_previous().vertex_id(), None, None);
                mesh.connectivity_info.set_halfedge_twin(halfedge_id, boundary_halfedge_id);
            }
        }
        mesh
    }

    fn new_internal(positions: HashMap<VertexID, Vec3>, normals: HashMap<VertexID, Vec3>, connectivity_info: Rc<ConnectivityInfo>) -> Mesh
    {
        Mesh {positions, normals, connectivity_info}
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

    pub fn clone_subset(&self, faces: &std::collections::HashSet<FaceID>) -> Mesh
    {
        let info = ConnectivityInfo::new(faces.len(), faces.len());
        for face_id in faces {
            let face = self.connectivity_info.face(face_id).unwrap();
            for mut walker in self.face_halfedge_iter(face_id) {
                let halfedge_id = walker.halfedge_id().unwrap();
                let halfedge = self.connectivity_info.halfedge(&halfedge_id).unwrap();
                info.add_halfedge(halfedge_id, halfedge);

                let vertex_id = walker.vertex_id().unwrap();
                let vertex = self.connectivity_info.vertex(&vertex_id).unwrap();
                info.add_vertex(vertex_id, vertex);
                info.set_vertex_halfedge(&vertex_id, walker.next_id());

                walker.as_twin();
                if walker.face_id().is_none()
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    info.add_halfedge(twin_id, twin);

                }
                else if !faces.contains(&walker.face_id().unwrap())
                {
                    let twin_id = walker.halfedge_id().unwrap();
                    let mut twin = self.connectivity_info.halfedge(&twin_id).unwrap();
                    twin.face = None;
                    twin.next = None;
                    info.add_halfedge(twin_id, twin);
                }
            }

            info.add_face(face_id.clone(), face);
        }

        let mut positions = HashMap::with_capacity(info.no_vertices());
        let mut normals = HashMap::with_capacity(info.no_vertices());
        for vertex_id in info.vertex_iterator() {
            let p = self.position(&vertex_id).clone();
            positions.insert(vertex_id.clone(), p);
            if let Some(normal) = self.normal(&vertex_id) {
                normals.insert(vertex_id, normal.clone());
            }
        }

        Mesh::new_internal(positions, normals, Rc::new(info))
    }

    pub fn append(&mut self, other: &Self)
    {
        let mut mapping: HashMap<VertexID, VertexID> = HashMap::new();
        let mut get_or_create_vertex = |mesh: &mut Mesh, vertex_id| -> VertexID {
            if let Some(vid) = mapping.get(&vertex_id) {return vid.clone();}
            let p = other.position(&vertex_id);
            let n = other.normal(&vertex_id).map(|n| n.clone());
            let vid = mesh.create_vertex(p.clone(), n);
            mapping.insert(vertex_id, vid);
            vid
        };

        for face_id in other.face_iter() {
            let vertex_ids = other.face_vertices(&face_id);

            let vertex_id0 = get_or_create_vertex(self, vertex_ids.0);
            let vertex_id1 = get_or_create_vertex(self, vertex_ids.1);
            let vertex_id2 = get_or_create_vertex(self, vertex_ids.2);
            self.connectivity_info.create_face(&vertex_id0, &vertex_id1, &vertex_id2);
        }

        self.create_twin_connectivity();
    }

    pub(crate) fn flip_orientation_of_face(&mut self, face_id: &FaceID)
    {
        let mut update_list = [(None, None, None); 3];

        let mut i = 0;
        for mut walker in self.face_halfedge_iter(face_id) {
            let halfedge_id = walker.halfedge_id();
            let vertex_id = walker.vertex_id();
            walker.as_previous();
            update_list[i] = (halfedge_id.clone(), walker.vertex_id(), walker.halfedge_id());
            i += 1;

            self.connectivity_info.set_vertex_halfedge(&walker.vertex_id().unwrap(), walker.halfedge_id());

            walker.as_next().as_twin();
            if walker.face_id().is_none() {
                self.connectivity_info.set_vertex_halfedge(&walker.vertex_id().unwrap(), walker.halfedge_id());
                self.connectivity_info.set_halfedge_vertex(&walker.halfedge_id().unwrap(), vertex_id.unwrap());
            }
        }

        for (halfedge_id, new_vertex_id, new_next_id) in update_list.iter() {
            self.connectivity_info.set_halfedge_vertex(&halfedge_id.unwrap(), new_vertex_id.unwrap());
            self.connectivity_info.set_halfedge_next(&halfedge_id.unwrap(), *new_next_id);
        }
    }

    pub fn flip_orientation(&mut self)
    {
        for face_id in self.face_iter() {
            self.flip_orientation_of_face(&face_id);
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
        for vertex_id in self.vertex_iter() {
            let p = *self.position(&vertex_id);
            self.set_position(vertex_id, p * scale);
        }
    }

    pub fn translate(&mut self, translation: &Vec3)
    {
        for vertex_id in self.vertex_iter() {
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
        for walker in self.vertex_halfedge_iter(&vertex_id) {
            if let Some(face_id) = walker.face_id() {
                normal = normal + self.face_normal(&face_id)
            }
        }
        normal.normalize()
    }

    pub fn update_vertex_normals(&mut self)
    {
        for vertex_id in self.vertex_iter() {
            let normal = self.compute_vertex_normal(&vertex_id);
            self.set_normal(vertex_id, normal);
        }
    }

    ///////////////////////////////////////////////////
    // *** Internal connectivity changing functions ***
    ///////////////////////////////////////////////////

    fn create_vertex(&mut self, position: Vec3, normal: Option<Vec3>) -> VertexID
    {
        let id = self.connectivity_info.new_vertex();
        self.positions.insert(id.clone(), position);
        if let Some(nor) = normal {self.normals.insert(id.clone(), nor);}
        id
    }

    fn create_twin_connectivity(&mut self)
    {
        let mut walker = self.walker();
        let edges: Vec<HalfEdgeID> = self.halfedge_iter().collect();

        for i1 in 0..edges.len()
        {
            let halfedge_id1 = edges[i1];
            if walker.as_halfedge_walker(&halfedge_id1).twin_id().is_none()
            {
                let vertex_id1 = walker.vertex_id().unwrap();
                let vertex_id2 = walker.as_previous().vertex_id().unwrap();

                let mut halfedge2 = None;
                for i2 in i1+1..edges.len()
                {
                    let halfedge_id2 = &edges[i2];
                    if walker.as_halfedge_walker(halfedge_id2).twin_id().is_none()
                    {
                        if walker.vertex_id().unwrap() == vertex_id2 && walker.as_previous().vertex_id().unwrap() == vertex_id1
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

impl Clone for Mesh {
    fn clone(&self) -> Mesh {
        Mesh::new_internal(self.positions.clone(), self.normals.clone(), Rc::new((*self.connectivity_info).clone()))
    }
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "**** Connectivity: ****")?;
        writeln!(f, "{}", self.connectivity_info)?;
        writeln!(f, "**** Positions: ****")?;
        writeln!(f, "{:?}", self.positions)?;
        writeln!(f, "**** Normals: ****")?;
        writeln!(f, "{:?}", self.normals)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utility::*;

    #[test]
    fn test_one_face_connectivity() {
        let mut mesh = Mesh::new_with_connectivity(vec![], vec![], None);

        let v1 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v2 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let v3 = mesh.create_vertex(vec3(0.0, 0.0, 0.0), None);
        let f1 = mesh.connectivity_info.create_face(&v1, &v2, &v3);
        mesh.create_twin_connectivity();

        let t1 = mesh.walker_from_vertex(&v1).vertex_id();
        assert_eq!(t1, Some(v2.clone()));

        let t2 = mesh.walker_from_vertex(&v1).as_twin().vertex_id();
        assert_eq!(t2, Some(v1));

        let t3 = mesh.walker_from_vertex(&v2.clone()).as_next().as_next().vertex_id();
        assert_eq!(t3, Some(v2.clone()));

        let t4 = mesh.walker_from_face(&f1.clone()).as_twin().face_id();
        assert!(t4.is_none());

        let t5 = mesh.walker_from_face(&f1.clone()).as_twin().next_id();
        assert!(t5.is_none());

        let t6 = mesh.walker_from_face(&f1.clone()).as_previous().as_previous().as_twin().as_twin().face_id();
        assert_eq!(t6, Some(f1.clone()));

        let t7 = mesh.walker_from_vertex(&v2.clone()).as_next().as_next().next_id();
        assert_eq!(t7, mesh.walker_from_vertex(&v2).halfedge_id());

        let t8 = mesh.walker_from_vertex(&v3).face_id();
        assert_eq!(t8, Some(f1));
    }

    #[test]
    fn test_three_face_connectivity() {
        let mesh = create_three_connected_faces();
        let mut id = None;
        for vertex_id in mesh.vertex_iter() {
            let mut round = true;
            for walker in mesh.vertex_halfedge_iter(&vertex_id) {
                if walker.face_id().is_none() { round = false; break; }
            }
            if round { id = Some(vertex_id); break; }
        }
        let mut walker = mesh.walker_from_vertex(&id.unwrap());
        let start_edge = walker.halfedge_id().unwrap();
        let one_round_edge = walker.as_previous().as_twin().as_previous().as_twin().as_previous().twin_id().unwrap();
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

        for vertex_id in mesh.vertex_iter() {
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

        let mesh = Mesh::new(positions, None);

        assert_eq!(4, mesh.no_vertices());
        assert_eq!(3, mesh.no_faces());
        test_is_valid(&mesh).unwrap();
    }

    #[test]
    fn test_create_sub_mesh()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = crate::MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let mut faces = std::collections::HashSet::new();
        for face_id in mesh.face_iter() {
            faces.insert(face_id);
            break;
        }

        let sub_mesh = mesh.clone_subset(&faces);

        test_is_valid(&mesh).unwrap();
        test_is_valid(&sub_mesh).unwrap();
    }

    #[test]
    fn test_flip_orientation_of_face()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  1, 2, 3];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5];
        let mut mesh = crate::MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        mesh.flip_orientation_of_face(&mesh.face_iter().next().unwrap());
        test_is_valid(&mesh).unwrap();

    }
}