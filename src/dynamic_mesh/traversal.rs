use std::rc::{Rc};
use crate::dynamic_mesh::*;
use crate::dynamic_mesh::connectivity_info::{HalfEdge, ConnectivityInfo};
use std::collections::HashSet;
use std::iter::FromIterator;

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
pub type HalfEdgeIterator = Box<Iterator<Item = HalfEdgeID>>;
pub type FaceIterator = Box<Iterator<Item = FaceID>>;
pub type HalfEdgeTwinsIterator = Box<Iterator<Item = (HalfEdgeID, HalfEdgeID)>>;
pub type EdgeIterator = Box<Iterator<Item = (VertexID, VertexID)>>;

impl DynamicMesh
{
    pub fn walker(&self) -> Walker
    {
        Walker::new(&self.connectivity_info)
    }

    pub fn walker_from_vertex(&self, vertex_id: &VertexID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_vertex_halfedge_walker(vertex_id)
    }

    pub fn walker_from_halfedge(&self, halfedge_id: &HalfEdgeID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_halfedge_walker(halfedge_id)
    }

    pub fn walker_from_face(&self, face_id: &FaceID) -> Walker
    {
        Walker::new(&self.connectivity_info).into_face_halfedge_walker(face_id)
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
        self.connectivity_info.vertex_iterator()
    }

    pub fn halfedge_iterator(&self) -> HalfEdgeIterator
    {
        self.connectivity_info.halfedge_iterator()
    }

    pub fn halfedge_twins_iterator(&self) -> HalfEdgeTwinsIterator
    {
        let mut values = Vec::with_capacity(self.no_halfedges()/2);
        for halfedge_id in self.halfedge_iterator() {
            let twin_id = self.walker_from_halfedge(&halfedge_id).twin_id().unwrap();
            if halfedge_id < twin_id {
                values.push((halfedge_id, twin_id))
            }
        }
        Box::new(values.into_iter())
    }

    pub fn face_iterator(&self) -> FaceIterator
    {
        self.connectivity_info.face_iterator()
    }

    pub fn edge_iterator(&self) -> EdgeIterator
    {
        let set: HashSet<(VertexID, VertexID)> = HashSet::from_iter(self.halfedge_iterator().map(|halfedge_id| self.ordered_edge_vertices(&halfedge_id)));
        Box::new(set.into_iter())
    }
}

pub struct VertexHalfedgeIterator
{
    current: Walker,
    start: HalfEdgeID,
    is_done: bool
}

impl VertexHalfedgeIterator {
    pub(crate) fn new(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> VertexHalfedgeIterator
    {
        let current = Walker::new(connectivity_info).into_vertex_halfedge_walker(vertex_id);
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
    pub(crate) fn new(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> FaceHalfedgeIterator
    {
        let current = Walker::new(connectivity_info).into_face_halfedge_walker(face_id);
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

#[derive(Debug, Clone)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>
}

impl Walker
{
    pub(crate) fn new(connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: None, current_info: None, connectivity_info: connectivity_info.clone()}
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>)
    {
        self.current_info = if let Some(ref id) = halfedge_id { self.connectivity_info.halfedge(id) } else { None };
        self.current = halfedge_id;
    }

    pub fn into_vertex_halfedge_walker(mut self, vertex_id: &VertexID) -> Self
    {
        self.as_vertex_halfedge_walker(vertex_id);
        self
    }

    pub fn into_halfedge_walker(mut self, halfedge_id: &HalfEdgeID) -> Self
    {
        self.as_halfedge_walker(halfedge_id);
        self
    }

    pub fn into_face_halfedge_walker(mut self, face_id: &FaceID) -> Self
    {
        self.as_face_halfedge_walker(face_id);
        self
    }

    pub fn as_vertex_halfedge_walker(&mut self, vertex_id: &VertexID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn as_halfedge_walker(&mut self, halfedge_id: &HalfEdgeID) -> &mut Self
    {
        let halfedge_id = Some(halfedge_id.clone());
        self.set_current(halfedge_id);
        self
    }

    pub fn as_face_halfedge_walker(&mut self, face_id: &FaceID) -> &mut Self
    {
        let halfedge_id = self.connectivity_info.face_halfedge(face_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn twin(&mut self) -> &mut Walker
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.twin.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn twin_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.twin.clone() }
        else { None }
    }

    pub fn next(&mut self) -> &mut Walker
    {
        let halfedge_id = match self.current_info {
            Some(ref current_info) => { current_info.next.clone() },
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn next_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.next.clone() }
        else { None }
    }

    pub fn previous(&mut self) -> &mut Walker
    {
        self.next().next()
    }

    pub fn previous_id(&self) -> Option<HalfEdgeID>
    {
        if let Some(ref next_id) = self.next_id() { Walker::new(&self.connectivity_info.clone()).into_halfedge_walker(next_id).next_id() }
        else { None }
    }

    pub fn vertex_id(&self) -> Option<VertexID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.vertex.clone() }
        else { None }
    }

    pub fn halfedge_id(&self) -> Option<HalfEdgeID>
    {
        self.current.clone()
    }

    pub fn face_id(&self) -> Option<FaceID>
    {
        if let Some(ref halfedge) = self.current_info { halfedge.face.clone() }
        else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic_mesh::test_utility::*;

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
        let mesh = DynamicMesh::new_with_connectivity(indices, positions, None);

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
}