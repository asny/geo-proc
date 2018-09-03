use traversal::*;
use std::rc::Rc;

pub struct OneRingIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeWalker,
    start: HalfEdgeID
}

impl OneRingIterator {
    pub fn new(vertex_id: &VertexID, connectivity_info: Rc<ConnectivityInfo>) -> OneRingIterator
    {
        let current = VertexWalker::new(vertex_id.clone(), connectivity_info.clone()).halfedge();
        let start = current.deref();
        OneRingIterator { connectivity_info, current, start }
    }
}

impl Iterator for OneRingIterator {
    type Item = HalfEdgeWalker;

    fn next(&mut self) -> Option<HalfEdgeWalker>
    {
        if self.current.deref() == self.start {
            return None;
        }
        let curr = self.current.clone();
        self.current = self.current.clone().previous().twin();

        if self.current.deref().is_null() { // In the case there are holes in the one-ring
            self.current = HalfEdgeWalker::new(self.start.clone(), self.connectivity_info.clone());
            loop {
                let temp = self.current.clone().twin().next();
                match temp.deref_option()
                {
                    Some(e) => { self.current = temp; },
                    None => { break; }
                }
            }
        }
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
        if self.current.deref() == self.start {
            self.is_done = true;;
        }
        Some(curr)
    }
}