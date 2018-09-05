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
    pub fn new_from_vertex(vertex_id: &VertexID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: connectivity_info.vertex_halfedge(vertex_id), connectivity_info: connectivity_info.clone()}
    }

    pub fn new(halfedge_id: &HalfEdgeID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: halfedge_id.clone(), connectivity_info: connectivity_info.clone()}
    }

    pub fn new_from_face(face_id: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> Walker
    {
        Walker {current: connectivity_info.face_halfedge(face_id), connectivity_info: connectivity_info.clone()}
    }

    pub fn vertex_id(&self) -> VertexID
    {
        match self.current.is_null() {
            true => { VertexID::null() },
            false => { self.connectivity_info.halfedge_vertex(&self.current) }
        }
    }

    pub fn twin(&self) -> Walker
    {
        let id = match self.current.is_null() {
            true => { HalfEdgeID::null() },
            false => { self.connectivity_info.halfedge_twin(&self.current) }
        };
        Walker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn twin_mut(&mut self) -> &mut Walker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_twin(&self.current);
        }
        self
    }

    pub fn next(&self) -> Walker
    {
        let id = match self.current.is_null() {
            true => { HalfEdgeID::null() },
            false => { self.connectivity_info.halfedge_next(&self.current) }
        };
        Walker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn next_mut(&mut self) -> &mut Walker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_next(&self.current);
        }
        self
    }

    pub fn previous(&self) -> Walker
    {
        self.next().next()
    }

    pub fn previous_mut(&mut self) -> &mut Walker
    {
        self.next_mut().next_mut()
    }

    pub fn face(&self) -> FaceWalker
    {
        let id = match self.current.is_null() {
            true => { FaceID::null() },
            false => { self.connectivity_info.halfedge_face(&self.current) }
        };
        FaceWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn face_id(&self) -> FaceID
    {
        match self.current.is_null() {
            true => { FaceID::null() },
            false => { self.connectivity_info.halfedge_face(&self.current) }
        }
    }

    pub fn id(&self) -> HalfEdgeID
    {
        self.current.clone()
    }
}

#[derive(Debug, Clone)]
pub struct FaceWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: FaceID
}

impl FaceWalker
{
    pub fn new(current: &FaceID, connectivity_info: &Rc<ConnectivityInfo>) -> FaceWalker
    {
        FaceWalker {current: current.clone(), connectivity_info: connectivity_info.clone()}
    }

    pub fn halfedge(&self) -> Walker
    {
        let id = match self.current.is_null() {
            true => { HalfEdgeID::null() },
            false => { self.connectivity_info.face_halfedge(&self.current) }
        };
        Walker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn id(&self) -> FaceID
    {
        self.current.clone()
    }
}

