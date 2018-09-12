use std::rc::{Rc};
use ids::*;
use connectivity_info::ConnectivityInfo;

#[derive(Debug, Clone)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: Option<HalfEdgeID>
}

impl Walker
{
    pub fn create(connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: None, connectivity_info: connectivity_info.clone()}
    }

    pub fn create_from_vertex(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: connectivity_info.vertex_halfedge(vertex_id), connectivity_info: connectivity_info.clone()}
    }

    pub fn create_from_halfedge(halfedge_id: &HalfEdgeID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: Some(halfedge_id.clone()), connectivity_info: connectivity_info.clone()}
    }

    pub fn create_from_face(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: connectivity_info.face_halfedge(face_id), connectivity_info: connectivity_info.clone()}
    }

    pub fn jump_to_vertex(&mut self, vertex_id: &VertexID) -> &mut Walker
    {
        self.current = self.connectivity_info.vertex_halfedge(vertex_id);
        self
    }

    pub fn jump_to_edge(&mut self, halfedge_id: &HalfEdgeID) -> &mut Walker
    {
        self.current = Some(halfedge_id.clone());
        self
    }

    pub fn jump_to_face(&mut self, face_id: &FaceID) -> &mut Walker
    {
        self.current = self.connectivity_info.face_halfedge(face_id);
        self
    }

    pub fn twin(&mut self) -> &mut Walker
    {
        self.current = match self.current {
            Some(ref current) => {self.connectivity_info.halfedge_twin(current)},
            None => None
        };
        self
    }

    pub fn next(&mut self) -> &mut Walker
    {
        self.current = match self.current {
            Some(ref current) => {self.connectivity_info.halfedge_next(current)},
            None => None
        };
        self
    }

    pub fn previous(&mut self) -> &mut Walker
    {
        self.next().next()
    }

    pub fn vertex_id(&self) -> Option<VertexID>
    {
        match self.current {
            Some(ref current) => self.connectivity_info.halfedge_vertex(current),
            None => None
        }
    }

    pub fn halfedge_id(&self) -> Option<HalfEdgeID>
    {
        self.current.clone()
    }

    pub fn face_id(&self) -> Option<FaceID>
    {
        match self.current {
            Some(ref current) => self.connectivity_info.halfedge_face(current),
            None => None
        }
    }
}

