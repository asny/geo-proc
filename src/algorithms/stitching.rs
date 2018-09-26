
use std::collections::{HashMap, HashSet};

use types::*;
use ids::*;
use connectivity::*;
use collision::*;
use dynamic_mesh::DynamicMesh;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let stitches = split_at_intersections(mesh1, mesh2);
    // Todo:
    mesh1.clone()
}

fn split_at_intersections(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> Vec<(VertexID, VertexID)>
{
    let mut intersections = find_intersections(mesh1, mesh2);

    for ((vertex_id1, vertex_id2), point) in intersections.vertex_vertex_intersections.iter()
    {
        mesh1.set_position(vertex_id1.clone(), point.clone());
        mesh2.set_position(vertex_id2.clone(), point.clone());
    }

    let mut face_splits1 = HashMap::new();
    for ((face_id1, edge2), point) in intersections.face_edge_intersections.drain()
    {
        match find_type_to_split(&face_splits1, mesh1, face_id1, &point) {
            PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point); },
            PrimitiveID::Edge(edge1) => { intersections.edge_edge_intersections.insert((edge1, edge2), point); },
            PrimitiveID::Face(face_id) => {
                let vertex_id1 = mesh1.split_face(&face_id, point);
                insert_faces(&mut face_splits1, mesh1, face_id1, &vertex_id1);
                intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point);
            }
        }
    }

    let mut face_splits2 = HashMap::new();
    for ((edge1, face_id2), point) in intersections.edge_face_intersections.drain()
    {
        match find_type_to_split(&face_splits2, mesh2, face_id2, &point) {
            PrimitiveID::Vertex(vertex_id2) => { intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point); },
            PrimitiveID::Edge(edge2) => { intersections.edge_edge_intersections.insert((edge1, edge2), point); },
            PrimitiveID::Face(face_id) => {
                let vertex_id2 = mesh2.split_face(&face_id, point);
                insert_faces(&mut face_splits2, mesh2, face_id2, &vertex_id2);
                intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point);
            }
        }
    }

    for ((face_id1, vertex_id2), point) in intersections.face_vertex_intersections.drain()
    {
        match find_type_to_split(&face_splits1, mesh1, face_id1, &point) {
            PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
            PrimitiveID::Edge(edge1) => { intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point); },
            PrimitiveID::Face(face_id) => {
                let vertex_id1 = mesh1.split_face(&face_id, point);
                insert_faces(&mut face_splits1, mesh1, face_id1, &vertex_id1);

                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
            }
        }
    }

    for ((vertex_id1, face_id2), point) in intersections.vertex_face_intersections.drain()
    {
        match find_type_to_split(&face_splits2, mesh2, face_id2, &point) {
            PrimitiveID::Vertex(vertex_id2) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
            PrimitiveID::Edge(edge2) => { intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point); },
            PrimitiveID::Face(face_id) => {
                let vertex_id2 = mesh2.split_face(&face_id, point);
                insert_faces(&mut face_splits2, mesh2, face_id2, &vertex_id2);
                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
            }
        }
    }

    let mut edge_splits1 = HashMap::new();
    for ((edge1, edge2), point) in intersections.edge_edge_intersections.drain()
    {
        match find_type_to_split_edge(&edge_splits1, mesh1, edge1.clone(), &point) {
            PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point); },
            PrimitiveID::Edge(edge) => {
                let halfedge_id1 = connecting_edge(mesh1, &edge.v0, &edge.v1).unwrap();
                let vertex_id1 = mesh1.split_edge(&halfedge_id1, point);
                insert_edges(&mut edge_splits1, mesh1, edge1, &edge,&vertex_id1);
                intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point);
            },
            PrimitiveID::Face(face_id) => {}
        }
    }

    for ((edge1, vertex_id2), point) in intersections.edge_vertex_intersections.drain()
    {
        match find_type_to_split_edge(&edge_splits1, mesh1, edge1.clone(), &point) {
            PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
            PrimitiveID::Edge(edge) => {
                let halfedge_id1 = connecting_edge(mesh1, &edge.v0, &edge.v1).unwrap();
                let vertex_id1 = mesh1.split_edge(&halfedge_id1, point);
                insert_edges(&mut edge_splits1, mesh1, edge1, &edge, &vertex_id1);
                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
            },
            PrimitiveID::Face(face_id) => {}
        }
    }

    let mut edge_splits2 = HashMap::new();
    for ((vertex_id1, edge2), point) in intersections.vertex_edge_intersections.drain()
    {
        match find_type_to_split_edge(&edge_splits2, mesh2, edge2.clone(), &point) {
            PrimitiveID::Vertex(vertex_id2) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
            PrimitiveID::Edge(edge) => {
                let halfedge_id2 = connecting_edge(mesh2, &edge.v0, &edge.v1).unwrap();
                let vertex_id2 = mesh2.split_edge(&halfedge_id2, point);
                insert_edges(&mut edge_splits2, mesh2, edge2, &edge,&vertex_id2);
                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
            },
            PrimitiveID::Face(face_id) => {}
        }
    }

    intersections.vertex_vertex_intersections.iter().map(|pair| pair.0.clone()).collect()
}

fn find_type_to_split(face_splits: &HashMap<FaceID, HashSet<FaceID>>, mesh: &DynamicMesh, face_id: FaceID, point: &Vec3) -> PrimitiveID
{
    if let Some(new_faces) = face_splits.get(&face_id)
    {
        for new_face_id in new_faces.iter()
        {
            if is_inside(mesh, new_face_id, &point)
            {
                match find_close_type(mesh, new_face_id.clone(), &point) {
                    PrimitiveID::Vertex(vertex_id) => { return PrimitiveID::Vertex(vertex_id) },
                    PrimitiveID::Edge(edge_id) => { return PrimitiveID::Edge(edge_id) },
                    PrimitiveID::Face(fid) => { return PrimitiveID::Face(fid) }
                }
            }
        }
        panic!("ARGH")
    }
    PrimitiveID::Face(face_id)
}

fn is_inside(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> bool
{
    let mut walker = mesh.walker_from_face(face_id);
    let p0 = *mesh.position(&walker.vertex_id().unwrap());
    walker.next();
    let p1 = *mesh.position(&walker.vertex_id().unwrap());
    walker.next();
    let p2 = *mesh.position(&walker.vertex_id().unwrap());

    let a = |p0: &Vec3, p1: &Vec3, p2: &Vec3| -> f32 {(p1 - p0).cross(&(p2 - p0)).norm()};

    let f = a(&p0, &p1, &p2) - (a(&p0, &p1, point) + a(&p1, &p2, point) + a(&p2, &p0, point));
    f.abs() <= 0.001
}

fn find_type_to_split_edge(edge_splits: &HashMap<Edge, HashSet<Edge>>, mesh: &DynamicMesh, edge: Edge, point: &Vec3) -> PrimitiveID
{
    if let Some(new_edges) = edge_splits.get(&edge)
    {
        for new_edge in new_edges
        {
            let v1 = point - mesh.position(&new_edge.v0);
            let v2 = point - mesh.position(&new_edge.v1);
            if v1.dot(&v2) < MARGIN
            {
                if let Some(vertex_id) = find_close_vertex_on_edge(mesh, &edge, &point) {
                    return PrimitiveID::Vertex(vertex_id)
                }
                return PrimitiveID::Edge(new_edge.clone())
            }
        }
        panic!("ARGH")
    }
    PrimitiveID::Edge(edge)
}

fn insert_edges(edge_list: &mut HashMap<Edge, HashSet<Edge>>, mesh: &DynamicMesh, edge: Edge, split_edge: &Edge, vertex_id: &VertexID)
{
    if !edge_list.contains_key(&edge) { edge_list.insert(edge.clone(), HashSet::new()); }
    let list = edge_list.get_mut(&edge).unwrap();
    list.insert(Edge::new(edge.v0, vertex_id.clone()));
    list.insert(Edge::new(edge.v1, vertex_id.clone()));
}

fn insert_faces(face_list: &mut HashMap<FaceID, HashSet<FaceID>>, mesh: &DynamicMesh, face_id: FaceID, vertex_id: &VertexID)
{
    if !face_list.contains_key(&face_id) { face_list.insert(face_id, HashSet::new()); }
    let list = face_list.get_mut(&face_id).unwrap();

    let mut iter = mesh.vertex_halfedge_iterator(vertex_id);
    list.insert(iter.next().unwrap().face_id().unwrap());
    list.insert(iter.next().unwrap().face_id().unwrap());
    list.insert(iter.next().unwrap().face_id().unwrap());
}

#[derive(Debug)]
struct Intersections
{
    pub face_edge_intersections: HashMap<(FaceID, Edge), Vec3>,
    pub edge_face_intersections: HashMap<(Edge, FaceID), Vec3>,

    pub face_vertex_intersections: HashMap<(FaceID, VertexID), Vec3>,
    pub vertex_face_intersections: HashMap<(VertexID, FaceID), Vec3>,

    pub edge_edge_intersections: HashMap<(Edge, Edge), Vec3>,

    pub vertex_edge_intersections: HashMap<(VertexID, Edge), Vec3>,
    pub edge_vertex_intersections: HashMap<(Edge, VertexID), Vec3>,

    pub vertex_vertex_intersections: HashMap<(VertexID, VertexID), Vec3>
}

impl Intersections
{
    pub fn new() -> Intersections
    {
        Intersections {face_edge_intersections: HashMap::new(), edge_face_intersections: HashMap::new(), face_vertex_intersections: HashMap::new(),
            vertex_face_intersections: HashMap::new(), edge_edge_intersections: HashMap::new(),
            vertex_edge_intersections: HashMap::new(), edge_vertex_intersections: HashMap::new(), vertex_vertex_intersections: HashMap::new()}
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Edge
{
    pub v0: VertexID,
    pub v1: VertexID
}

impl Edge {
    fn new(v0: VertexID, v1: VertexID) -> Edge
    {
        if v0 < v1 {Edge{v0, v1}} else {Edge{v0: v1, v1: v0}}
    }
}

fn find_intersections(mesh1: &DynamicMesh, mesh2: &DynamicMesh) -> Intersections
{
    let edges1 = mesh1.edge_iterator().collect();
    let edges2 = mesh2.edge_iterator().collect();
    find_intersections_between_edge_face(mesh1, &edges1, mesh2, &edges2)
}

fn find_intersections_between_edge_face(mesh1: &DynamicMesh, edges1: &Vec<(VertexID, VertexID)>, mesh2: &DynamicMesh, edges2: &Vec<(VertexID, VertexID)>) -> Intersections
{
    let mut intersections = Intersections::new();
    for halfedge_id1 in edges1
    {
        for face_id2 in mesh2.face_iterator()
        {
            if let Some(point) = find_intersection_point(mesh2, &face_id2, mesh1,halfedge_id1)
            {
                let edge1 = Edge::new(halfedge_id1.0, halfedge_id1.1);
                if let Some(vertex_id1) = find_close_vertex_on_edge(mesh1,&edge1, &point)
                {
                    match find_close_type(mesh2, face_id2, &point) {
                        PrimitiveID::Vertex(vertex_id2) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
                        PrimitiveID::Edge(edge2) => { intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point); },
                        PrimitiveID::Face(face_id2) => { intersections.vertex_face_intersections.insert((vertex_id1, face_id2), point); }
                    }
                }
                else {
                    match find_close_type(mesh2, face_id2, &point) {
                        PrimitiveID::Vertex(vertex_id2) => { intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point); },
                        PrimitiveID::Edge(edge2) => { intersections.edge_edge_intersections.insert((edge1, edge2), point); },
                        PrimitiveID::Face(face_id2) => { intersections.edge_face_intersections.insert((edge1, face_id2), point); }
                    }
                }
            };
        }
    }
    for halfedge_id2 in edges2
    {
        for face_id1 in mesh1.face_iterator()
        {
            if let Some(point) = find_intersection_point(mesh1, &face_id1, mesh2, halfedge_id2)
            {
                let edge2 = Edge::new(halfedge_id2.0, halfedge_id2.1);
                if let Some(vertex_id2) = find_close_vertex_on_edge(mesh2,&edge2, &point)
                {
                    match find_close_type(mesh1, face_id1, &point) {
                        PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point); },
                        PrimitiveID::Edge(edge1) => { intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point); },
                        PrimitiveID::Face(face_id1) => { intersections.face_vertex_intersections.insert((face_id1, vertex_id2), point); }
                    }
                }
                else {
                    match find_close_type(mesh1, face_id1, &point) {
                        PrimitiveID::Vertex(vertex_id1) => { intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point); },
                        PrimitiveID::Edge(edge1) => { intersections.edge_edge_intersections.insert((edge1, edge2), point); },
                        PrimitiveID::Face(face_id1) => { intersections.face_edge_intersections.insert((face_id1, edge2), point); }
                    }
                }
            };
        }
    }
    println!("{:?}", intersections);
    intersections
}

enum PrimitiveID {
    Vertex(VertexID),
    Edge(Edge),
    Face(FaceID)
}

fn find_close_type(mesh: &DynamicMesh, face_id: FaceID, point: &Vec3) -> PrimitiveID
{
    if let Some(vertex_id) = find_close_vertex_on_face(mesh, &face_id, &point)
    {
        return PrimitiveID::Vertex(vertex_id)
    }
    else if let Some(edge) = find_close_edge(mesh, &face_id, &point)
    {
        return PrimitiveID::Edge(edge)
    }
    PrimitiveID::Face(face_id)
}

const MARGIN: f32 = 0.01;

fn find_close_edge(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<Edge>
{
    let mut walker = mesh.walker_from_face(face_id);
    let vertex_id1 = walker.vertex_id().unwrap();
    walker.next();
    let vertex_id2 = walker.vertex_id().unwrap();

    if point_line_segment_distance(point, mesh.position(&vertex_id1), mesh.position(&vertex_id2)) < MARGIN {
        return Some(Edge::new(vertex_id1, vertex_id2))
    }

    walker.next();
    let vertex_id3 = walker.vertex_id().unwrap();

    if point_line_segment_distance(point, mesh.position(&vertex_id2), mesh.position(&vertex_id3)) < MARGIN {
        return Some(Edge::new(vertex_id2, vertex_id3))
    }
    if point_line_segment_distance(point, mesh.position(&vertex_id3), mesh.position(&vertex_id1)) < MARGIN {
        return Some(Edge::new(vertex_id3, vertex_id1))
    }
    None
}

fn find_close_vertex_on_face(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<VertexID>
{
    for walker in mesh.face_halfedge_iterator(face_id) {
        let vertex_id = walker.vertex_id().unwrap();
        if (mesh.position(&vertex_id) - point).norm() < MARGIN {
            return Some(vertex_id)
        }
    }
    None
}

fn find_close_vertex_on_edge(mesh: &DynamicMesh, edge: &Edge, point: &Vec3) -> Option<VertexID>
{
    if(point - mesh.position(&edge.v0)).norm() < MARGIN
    {
        return Some(edge.v0)
    }
    if (point - mesh.position(&edge.v1)).norm() < MARGIN
    {
        return Some(edge.v1)
    }
    None
}



#[cfg(test)]
mod tests {
    use super::*;
    use mesh::Renderable;

    #[test]
    fn test_finding_edge_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 5);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);

        assert!(intersections.edge_edge_intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.25));
        assert!(intersections.edge_edge_intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.75));
        assert!(intersections.edge_edge_intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.25));
        assert!(intersections.edge_edge_intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.75));
        assert!(intersections.edge_edge_intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 2.25));
    }

    #[test]
    fn test_finding_face_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 1);
        assert_eq!(intersections.edge_face_intersections.len(), 1);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);
    }

    #[test]
    fn test_finding_face_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 1);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);
    }

    #[test]
    fn test_finding_edge_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.25,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 1);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);
    }

    #[test]
    fn test_finding_vertex_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![1.0, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 1);
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);

        assert_eq!(stitches.len(), 5);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_faces()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.face_edge_intersections.len(), 4);
        assert_eq!(intersections.edge_face_intersections.len(), 4);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh1.no_faces(), 19);
        assert_eq!(mesh1.no_halfedges(), 19 * 3 + 7);

        assert_eq!(mesh2.no_vertices(), 14);
        assert_eq!(mesh2.no_faces(), 19);
        assert_eq!(mesh2.no_halfedges(), 19 * 3 + 7);

        assert_eq!(stitches.len(), 8);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_face_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);
        let area1 = mesh1.area(&mesh1.face_iterator().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.face_edge_intersections.len(), 2);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 5);
        assert_eq!(mesh1.no_halfedges(), 5 * 3 + 3);

        let mut area_test1 = 0.0;
        for face_id in mesh1.face_iterator() {
            area_test1 = area_test1 + mesh1.area(&face_id);
        }
        assert!((area1 - area_test1).abs() < 0.001);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_edge_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.0, -0.2, 0.5,  0.0, -0.2, 1.5,  0.0, 1.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.face_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 2);
        assert_eq!(intersections.edge_vertex_intersections.len(), 0);
        assert_eq!(intersections.vertex_edge_intersections.len(), 0);
        assert_eq!(intersections.vertex_vertex_intersections.len(), 0);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 3);
        assert_eq!(mesh1.no_halfedges(), 3 * 3 + 5);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_simple_stitching()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();
        let stitched = stitch(&mut mesh1, &mut mesh2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    fn create_simple_mesh_x_z() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        DynamicMesh::create(indices, positions, None)
    }

    fn create_simple_mesh_y_z() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, -0.5, 1.0,  0.5, 0.5, 0.5,  0.5, 0.5, 1.5,  0.5, -0.5, 2.0,  0.5, 0.5, 2.5];
        DynamicMesh::create(indices, positions, None)
    }

    fn create_shifted_simple_mesh_y_z() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, -0.2,  0.5, -0.5, 0.8,  0.5, 0.5, 0.3,  0.5, 0.5, 1.3,  0.5, -0.5, 1.8,  0.5, 0.5, 2.3];
        DynamicMesh::create(indices, positions, None)
    }
}