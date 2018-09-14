use std::rc::{Rc};
use ids::*;
use connectivity_info::{HalfEdge, ConnectivityInfo};

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
        self.current_info = if let Some(ref id) = halfedge_id { self.connectivity_info.halfedge_info(id) } else { None };
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
        let halfedge_id = match self.current {
            Some(ref current) => {self.connectivity_info.halfedge_twin(current)},
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
        let halfedge_id = match self.current {
            Some(ref current) => {self.connectivity_info.halfedge_next(current)},
            None => None
        };
        self.set_current(halfedge_id);
        self
    }

    pub fn previous(&mut self) -> &mut Walker
    {
        self.next().next()
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

