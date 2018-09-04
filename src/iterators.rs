use traversal::*;
use std::rc::Rc;

pub struct HalfEdgeIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID,
    is_done: bool
}

impl HalfEdgeIterator {
    pub fn new(connectivity_info: Rc<ConnectivityInfo>) -> HalfEdgeIterator
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

pub struct OneRingIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeWalker,
    start: HalfEdgeID,
    is_done: bool
}

impl OneRingIterator {
    pub fn new(vertex_id: &VertexID, connectivity_info: Rc<ConnectivityInfo>) -> OneRingIterator
    {
        let current = VertexWalker::new(vertex_id.clone(), connectivity_info.clone()).halfedge();
        let start = current.deref();
        OneRingIterator { connectivity_info, current, start, is_done: false }
    }
}

impl Iterator for OneRingIterator {
    type Item = HalfEdgeWalker;

    fn next(&mut self) -> Option<HalfEdgeWalker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        self.current = self.current.previous().twin();

        if self.current.deref().is_null() { // In the case there are holes in the one-ring
            self.current = HalfEdgeWalker::new(self.start.clone(), self.connectivity_info.clone());
            loop {
                let temp = self.current.twin().next();
                match temp.deref_option()
                {
                    Some(e) => { self.current = temp; },
                    None => { break; }
                }
            }
        }
        self.is_done = self.current.deref() == self.start;
        Some(curr)
    }
}

pub struct FaceIterator
{
    current: HalfEdgeWalker,
    start: HalfEdgeID,
    is_done: bool
}

impl FaceIterator {
    pub fn new(face_id: &FaceID, connectivity_info: Rc<ConnectivityInfo>) -> FaceIterator
    {
        let current = FaceWalker::new(face_id.clone(), connectivity_info.clone()).halfedge();
        let start = current.deref().clone();
        FaceIterator { current, start, is_done: false }
    }
}

impl Iterator for FaceIterator {
    type Item = HalfEdgeWalker;

    fn next(&mut self) -> Option<HalfEdgeWalker>
    {
        if self.is_done { return None; }
        let curr = self.current.clone();
        self.current = self.current.next();
        self.is_done = self.current.deref() == self.start;
        Some(curr)
    }
}