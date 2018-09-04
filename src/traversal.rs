use std::rc::{Rc};
use ids::*;
use connectivity_info::ConnectivityInfo;

#[derive(Debug, Clone)]
pub struct VertexWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: VertexID
}

impl VertexWalker
{
    pub fn new(current: VertexID, connectivity_info: Rc<ConnectivityInfo>) -> VertexWalker
    {
        VertexWalker {current, connectivity_info}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.vertex_halfedge(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn id(&self) -> VertexID
    {
        self.current.clone()
    }
}

#[derive(Debug, Clone)]
pub struct HalfEdgeWalker
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID
}

impl HalfEdgeWalker
{
    pub fn new(current: HalfEdgeID, connectivity_info: Rc<ConnectivityInfo>) -> HalfEdgeWalker
    {
        HalfEdgeWalker {current, connectivity_info}
    }

    pub fn vertex(&self) -> VertexWalker
    {
        let id = match self.current.is_null() {
            true => { VertexID::null() },
            false => { self.connectivity_info.halfedge_vertex(&self.current) }
        };
        VertexWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn vertex_id(&self) -> VertexID
    {
        match self.current.is_null() {
            true => { VertexID::null() },
            false => { self.connectivity_info.halfedge_vertex(&self.current) }
        }
    }

    pub fn twin(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_twin(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn twin_mut(&mut self) -> &mut HalfEdgeWalker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_twin(&self.current);
        }
        self
    }

    pub fn next(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_next(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn next_mut(&mut self) -> &mut HalfEdgeWalker
    {
        if !self.current.is_null()
        {
            self.current = self.connectivity_info.halfedge_next(&self.current);
        }
        self
    }

    pub fn previous(&self) -> HalfEdgeWalker
    {
        self.next().next()
    }

    pub fn previous_mut(&mut self) -> &mut HalfEdgeWalker
    {
        self.next_mut().next_mut()
    }

    pub fn face(&self) -> FaceWalker
    {
        if self.current.is_null()
        {
            return FaceWalker { current: FaceID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.halfedge_face(&self.current);
        FaceWalker { current: id, connectivity_info: self.connectivity_info.clone() }
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
    pub fn new(current: FaceID, connectivity_info: Rc<ConnectivityInfo>) -> FaceWalker
    {
        FaceWalker {current, connectivity_info}
    }

    pub fn halfedge(&self) -> HalfEdgeWalker
    {
        if self.current.is_null()
        {
            return HalfEdgeWalker { current: HalfEdgeID::null(), connectivity_info: self.connectivity_info.clone() }
        }
        let id = self.connectivity_info.face_halfedge(&self.current);
        HalfEdgeWalker { current: id, connectivity_info: self.connectivity_info.clone() }
    }

    pub fn id(&self) -> FaceID
    {
        self.current.clone()
    }
}

