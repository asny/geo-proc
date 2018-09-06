use std::rc::{Rc};
use ids::*;
use connectivity_info::ConnectivityInfo;

#[derive(Debug, Clone)]
pub struct Walker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID
}

impl Walker
{
    pub fn create_from_vertex(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: connectivity_info.vertex_halfedge(vertex_id), connectivity_info: connectivity_info.clone()}
    }

    pub fn create_from_halfedge(halfedge_id: &HalfEdgeID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: halfedge_id.clone(), connectivity_info: connectivity_info.clone()}
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
        self.current = halfedge_id.clone();
        self
    }

    pub fn jump_to_face(&mut self, face_id: &FaceID) -> &mut Walker
    {
        self.current = self.connectivity_info.face_halfedge(face_id);
        self
    }

    pub fn twin(&mut self) -> &mut Walker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_twin(&self.current);
        }
        self
    }

    pub fn next(&mut self) -> &mut Walker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_next(&self.current);
        }
        self
    }

    pub fn previous(&mut self) -> &mut Walker
    {
        self.next().next()
    }

    pub fn vertex_id(&self) -> VertexID
    {
        match self.current.is_null() {
            true => { VertexID::null() },
            false => { self.connectivity_info.halfedge_vertex(&self.current) }
        }
    }

    pub fn halfedge_id(&self) -> HalfEdgeID
    {
        self.current.clone()
    }

    pub fn face_id(&self) -> FaceID
    {
        match self.current.is_null() {
            true => { FaceID::null() },
            false => { self.connectivity_info.halfedge_face(&self.current) }
        }
    }
}

