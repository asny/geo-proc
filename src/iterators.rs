use traversal::*;
use std::rc::Rc;

pub struct OneRingIterator
{
    connectivity_info: Rc<ConnectivityInfo>,
    current: HalfEdgeID,
    start: HalfEdgeID
}

impl OneRingIterator {
    pub fn new(vertex_id: &VertexID, connectivity_info: Rc<ConnectivityInfo>) -> OneRingIterator
    {
        let halfedge = VertexWalker::new(vertex_id.clone(), connectivity_info.clone()).halfedge().deref();
        OneRingIterator { connectivity_info, current: halfedge.clone(), start: halfedge }
    }
}

impl Iterator for OneRingIterator {
    type Item = HalfEdgeID;

    fn next(&mut self) -> Option<HalfEdgeID>
    {
        let mut next = HalfEdgeWalker::new(self.current.clone(), self.connectivity_info.clone()).twin().next().deref();
        if next.is_null() { // In the case there are holes in the one-ring
            let mut temp = self.start.clone();
            loop {
                match HalfEdgeWalker::new(temp.clone(), self.connectivity_info.clone()).next().next().twin().deref_option()
                {
                    Some(e) => { temp = e },
                    None => { break; }
                }
            }
            next = temp.clone();
        }
        if self.start == next {
            None
        }
        else {
            self.current = next.clone();
            Some(next)
        }
    }
}