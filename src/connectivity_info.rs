use std::cell::{RefCell};
use ids::*;
use std;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ConnectivityInfo {
    vertices: RefCell<HashMap<VertexID, Vertex>>,
    halfedges: RefCell<HashMap<HalfEdgeID, HalfEdge>>,
    faces: RefCell<HashMap<FaceID, Face>>
}

impl ConnectivityInfo {
    pub fn new(no_vertices: usize, no_faces: usize) -> ConnectivityInfo
    {
        ConnectivityInfo {
            vertices: RefCell::new(HashMap::with_capacity(no_vertices)),
            halfedges: RefCell::new(HashMap::with_capacity(4 * no_faces)),
            faces: RefCell::new(HashMap::with_capacity(no_faces))
        }
    }

    pub fn no_vertices(&self) -> usize
    {
        RefCell::borrow(&self.vertices).len()
    }

    pub fn no_halfedges(&self) -> usize
    {
        RefCell::borrow(&self.halfedges).len()
    }

    pub fn no_faces(&self) -> usize
    {
        RefCell::borrow(&self.faces).len()
    }

    pub fn create_vertex(&self) -> VertexID
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);

        let len = vertices.len();
        let mut id = VertexID::new(len);
        for i in len+1..std::usize::MAX {
            if !vertices.contains_key(&id) { break }
            id = VertexID::new(i);
        }

        vertices.insert(id.clone(), Vertex { halfedge: None });
        id
    }

    pub fn create_halfedge(&self, vertex: Option<VertexID>, face: Option<FaceID>) -> HalfEdgeID
    {
        let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);

        let len = halfedges.len();
        let mut id = HalfEdgeID::new(len);
        for i in len+1..std::usize::MAX {
            if !halfedges.contains_key(&id) { break }
            id = HalfEdgeID::new(i);
        }

        halfedges.insert(id.clone(), HalfEdge { vertex, twin: None, next: None, face });
        id
    }

    pub fn create_face(&self) -> FaceID
    {
        let faces = &mut *RefCell::borrow_mut(&self.faces);

        let len = faces.len();
        let mut id = FaceID::new(len);
        for i in len+1..std::usize::MAX {
            if !faces.contains_key(&id) { break }
            id = FaceID::new(i);
        }

        faces.insert(id.clone(), Face { halfedge: None });
        id
    }

    fn remove_vertex_if_lonely(&self, vertex_id: &VertexID)
    {
        let vertices = &mut *RefCell::borrow_mut(&self.vertices);

        for halfedge_id in self.halfedge_iterator() {
            if let Some(ref test_vertex_id) = self.halfedge_vertex(&halfedge_id) {
                if test_vertex_id == vertex_id {
                    vertices.get_mut(&vertex_id).unwrap().halfedge = self.halfedge_twin(&halfedge_id);
                    return;
                }
            }
        }
        vertices.remove(vertex_id);
    }

    fn remove_halfedge(&self, halfedge_id: &HalfEdgeID, twin_id: &HalfEdgeID, prev_id: &HalfEdgeID)
    {
        let vertex_id1 = self.halfedge_vertex(halfedge_id).unwrap();
        let vertex_id2 = self.halfedge_vertex(twin_id).unwrap();

        {
            let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
            if let Some(ref mut prev) = halfedges.get_mut(prev_id) { prev.next = None };
            halfedges.remove(twin_id);
            halfedges.remove(halfedge_id);
        }

        if &self.vertex_halfedge(&vertex_id1).unwrap() == twin_id {
            self.remove_vertex_if_lonely(&vertex_id1);
        }

        if &self.vertex_halfedge(&vertex_id2).unwrap() == halfedge_id {
            self.remove_vertex_if_lonely(&vertex_id2);
        }
    }

    pub fn remove_face(&self, face_id: &FaceID)
    {
        let mut edge_ids = vec![];
        let mut id = self.face_halfedge(face_id).unwrap(); edge_ids.push(id);
        id = self.halfedge_next(&edge_ids[0]).unwrap(); edge_ids.push(id);
        id = self.halfedge_next(&edge_ids[1]).unwrap(); edge_ids.push(id);

        let faces = &mut *RefCell::borrow_mut(&self.faces);
        faces.remove(face_id);

        {
            let halfedges = &mut *RefCell::borrow_mut(&self.halfedges);
            for halfedge_id in edge_ids.iter() {
                halfedges.get_mut(halfedge_id).unwrap().face = None;
            }
        }

        for i in 0..3
        {
            let halfedge_id = &edge_ids[i];
            let twin_id = self.halfedge_twin(halfedge_id).unwrap();
            if self.halfedge_face(halfedge_id).is_none() && self.halfedge_face(&twin_id).is_none()
            {
                self.remove_halfedge(halfedge_id, &twin_id, &edge_ids[(i+2)%3]);
            }
        }
    }

    pub fn set_vertex_halfedge(&self, id: &VertexID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.vertices).get_mut(id).unwrap().halfedge = Some(val.clone());
    }

    pub fn set_halfedge_vertex(&self, id: &HalfEdgeID, val: &VertexID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().vertex = Some(val.clone());
    }

    pub fn set_halfedge_next(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().next = Some(val.clone());
    }

    pub fn set_halfedge_twin(&self, id: &HalfEdgeID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().twin = Some(val.clone());
    }

    pub fn set_halfedge_face(&self, id: &HalfEdgeID, val: &FaceID)
    {
        RefCell::borrow_mut(&self.halfedges).get_mut(id).unwrap().face = Some(val.clone());
    }

    pub fn set_face_halfedge(&self, id: &FaceID, val: &HalfEdgeID)
    {
        RefCell::borrow_mut(&self.faces).get_mut(id).unwrap().halfedge = Some(val.clone());
    }

    pub fn vertex_iterator(&self) -> Box<Iterator<Item = VertexID>>
    {
        let vertices = RefCell::borrow(&self.vertices);
        let t: Vec<VertexID> = vertices.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn halfedge_iterator(&self) -> Box<Iterator<Item = HalfEdgeID>>
    {
        let halfedges = RefCell::borrow(&self.halfedges);
        let t: Vec<HalfEdgeID> = halfedges.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn face_iterator(&self) -> Box<Iterator<Item = FaceID>>
    {
        let faces = RefCell::borrow(&self.faces);
        let t: Vec<FaceID> = faces.iter().map(|pair| pair.0.clone()).collect();
        Box::new(t.into_iter())
    }

    pub fn vertex_halfedge(&self, vertex_id: &VertexID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.vertices).get(vertex_id).unwrap().halfedge.clone()
    }

    pub fn halfedge_vertex(&self, halfedge_id: &HalfEdgeID) -> Option<VertexID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().vertex.clone()
    }

    pub fn halfedge_twin(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().twin.clone()
    }

    pub fn halfedge_next(&self, halfedge_id: &HalfEdgeID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().next.clone()
    }

    pub fn halfedge_face(&self, halfedge_id: &HalfEdgeID) -> Option<FaceID>
    {
        RefCell::borrow(&self.halfedges).get(halfedge_id).unwrap().face.clone()
    }

    pub fn face_halfedge(&self, face_id: &FaceID) -> Option<HalfEdgeID>
    {
        RefCell::borrow(&self.faces).get(face_id).unwrap().halfedge.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub halfedge: Option<HalfEdgeID>
}

#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub vertex: Option<VertexID>,
    pub twin: Option<HalfEdgeID>,
    pub next: Option<HalfEdgeID>,
    pub face: Option<FaceID>
}

#[derive(Clone, Debug)]
pub struct Face {
    pub halfedge: Option<HalfEdgeID>
}