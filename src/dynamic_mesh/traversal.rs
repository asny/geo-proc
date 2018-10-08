use std::rc::{Rc};
use dynamic_mesh::*;
use dynamic_mesh::connectivity_info::{HalfEdge, ConnectivityInfo};

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

#[derive(Debug, Clone)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: Option<HalfEdgeID>,
    current_info: Option<HalfEdge>
}

impl Walker
{
    pub fn create(connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: None, current_info: None, connectivity_info: connectivity_info.clone()}
    }

    pub fn create_from_vertex(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        let mut walker = Walker::create(connectivity_info);
        walker.set_current(connectivity_info.vertex_halfedge(vertex_id));
        walker
    }

    pub fn create_from_halfedge(halfedge_id: &HalfEdgeID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        let mut walker = Walker::create(connectivity_info);
        walker.set_current(Some(halfedge_id.clone()));
        walker
    }

    pub fn create_from_face(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        let mut walker = Walker::create(connectivity_info);
        walker.set_current(connectivity_info.face_halfedge(face_id));
        walker
    }

    fn set_current(&mut self, halfedge_id: Option<HalfEdgeID>)
    {
        self.current_info = if let Some(ref id) = halfedge_id { self.connectivity_info.halfedge(id) } else { None };
        self.current = halfedge_id;
    }

    pub fn jump_to_vertex(&mut self, vertex_id: &VertexID) -> &mut Walker
    {
        let halfedge_id = self.connectivity_info.vertex_halfedge(vertex_id);
        self.set_current(halfedge_id);
        self
    }

    pub fn jump_to_edge(&mut self, halfedge_id: &HalfEdgeID) -> &mut Walker
    {
        let halfedge_id = Some(halfedge_id.clone());
        self.set_current(halfedge_id);
        self
    }

    pub fn jump_to_face(&mut self, face_id: &FaceID) -> &mut Walker
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
        if let Some(ref next_id) = self.next_id() { Walker::create_from_halfedge(next_id, &self.connectivity_info.clone()).next_id() }
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

