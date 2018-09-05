use ids::*;
use traversal::*;
use connectivity_info::ConnectivityInfo;
use std::rc::Rc;

pub struct VertexIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: VertexID,
    is_done: bool
}

impl VertexIterator {
    pub fn new(connectivity_info: &Rc<ConnectivityInfo>) -> VertexIterator
    {
        match connectivity_info.vertex_first_iter() {
            Some(vertex_id) => {
                VertexIterator { connectivity_info: connectivity_info.clone(), current: vertex_id, is_done: false }
            }
            None => {
                VertexIterator { connectivity_info: connectivity_info.clone(), current: VertexID::new(0), is_done: true }
            }
        }

    }
}

impl Iterator for VertexIterator {
    type Item = VertexID;

    fn next(&mut self) -> Option<VertexID>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        match self.connectivity_info.vertex_next_iter(&self.current) {
            Some(vertex) => { self.current = vertex },
            None => { self.is_done = true; }
        }
        Some(curr)
    }
}

pub struct HalfEdgeIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID,
    is_done: bool
}

impl HalfEdgeIterator {
    pub fn new(connectivity_info: &Rc<ConnectivityInfo>) -> HalfEdgeIterator
    {
        match connectivity_info.halfedge_first_iter() {
            Some(halfedge_id) => {
                HalfEdgeIterator { connectivity_info: connectivity_info.clone(), current: halfedge_id, is_done: false }
            }
            None => {
                HalfEdgeIterator { connectivity_info: connectivity_info.clone(), current: HalfEdgeID::new(0), is_done: true }
            }
        }

    }
}

impl Iterator for HalfEdgeIterator {
    type Item = HalfEdgeID;

    fn next(&mut self) -> Option<HalfEdgeID>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        match self.connectivity_info.halfedge_next_iter(&self.current) {
            Some(halfedge) => { self.current = halfedge },
            None => { self.is_done = true; }
        }
        Some(curr)
    }
}

pub struct FaceIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: FaceID,
    is_done: bool
}

impl FaceIterator {
    pub fn new(connectivity_info: &Rc<ConnectivityInfo>) -> FaceIterator
    {
        match connectivity_info.face_first_iter() {
            Some(face_id) => {
                FaceIterator { connectivity_info: connectivity_info.clone(), current: face_id, is_done: false }
            }
            None => {
                FaceIterator { connectivity_info: connectivity_info.clone(), current: FaceID::new(0), is_done: true }
            }
        }

    }
}

impl Iterator for FaceIterator {
    type Item = FaceID;

    fn next(&mut self) -> Option<FaceID>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        match self.connectivity_info.face_next_iter(&self.current) {
            Some(vertex) => { self.current = vertex },
            None => { self.is_done = true; }
        }
        Some(curr)
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
        let current = Walker::new_from_vertex(vertex_id, connectivity_info);
        let start = current.id();
        VertexHalfedgeIterator { current, start, is_done: false }
    }
}

impl Iterator for VertexHalfedgeIterator {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();

        if self.current.face_id().is_null() { // In the case there are holes in the one-ring
            self.current.twin_mut();
            loop {
                self.current.next_mut().twin_mut();
                if self.current.face_id().is_null() { self.current.twin_mut(); break; }
            }
        }
        else {
            self.current.previous_mut().twin_mut();
        }
        self.is_done = self.current.id() == self.start;
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
        let current = FaceWalker::new(face_id, connectivity_info).halfedge();
        let start = current.id().clone();
        FaceHalfedgeIterator { current, start, is_done: false }
    }
}

impl Iterator for FaceHalfedgeIterator {
    type Item = Walker;

    fn next(&mut self) -> Option<Walker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        self.current = self.current.next();
        self.is_done = self.current.id() == self.start;
        Some(curr)
    }
}