
use std::collections::HashMap;

use types::*;
use ids::*;
use connectivity::*;
use collision::*;
use dynamic_mesh::DynamicMesh;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let stitches = split_at_intersections(mesh1, mesh2);

    mesh1.clone()
}

fn split_at_intersections(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> Vec<(VertexID, VertexID)>
{
    let mut intersections = find_intersections(mesh1, mesh2);

    for ((face_id, edge), point) in intersections.face_edge_intersections.drain() {
        let vertex_id = mesh1.split_face(&face_id, point);
        intersections.vertex_edge_intersections.insert((vertex_id, edge), point);
    }

    for ((edge, face_id), point) in intersections.edge_face_intersections.drain() {
        let vertex_id = mesh2.split_face(&face_id, point);
        intersections.edge_vertex_intersections.insert((edge, vertex_id), point);
    }

    for ((face_id, vertex_id2), point) in intersections.face_vertex_intersections.drain() {
        let vertex_id1 = mesh1.split_face(&face_id, point);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((vertex_id1, face_id), point) in intersections.vertex_face_intersections.drain() {
        let vertex_id2 = mesh2.split_face(&face_id, point);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((edge1, edge2), point) in intersections.edge_edge_intersections.drain() {
        let halfedge_id1 = connecting_edge(mesh1, &edge1.v0, &edge1.v1).unwrap();
        let vertex_id1 = mesh1.split_edge(&halfedge_id1, point);
        let halfedge_id2 = connecting_edge(mesh2, &edge2.v0, &edge2.v1).unwrap();
        let vertex_id2 = mesh2.split_edge(&halfedge_id2, point);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((edge, vertex_id2), point) in intersections.edge_vertex_intersections.drain() {
        let halfedge_id = connecting_edge(mesh1, &edge.v0, &edge.v1).unwrap();
        let vertex_id1 = mesh1.split_edge(&halfedge_id, point);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((vertex_id1, edge), point) in intersections.vertex_edge_intersections.drain() {
        let halfedge_id = connecting_edge(mesh2, &edge.v0, &edge.v1).unwrap();
        let vertex_id2 = mesh2.split_edge(&halfedge_id, point);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    intersections.vertex_vertex_intersections.iter().map(|pair| pair.0.clone()).collect()
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

#[derive(Debug, Hash, Eq, PartialEq)]
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
    let mut intersections = Intersections::new();
    for face_id1 in mesh1.face_iterator()
    {
        for face_id2 in mesh2.face_iterator()
        {
            if is_intersecting(mesh1, &face_id1, mesh2, &face_id2)
            {
                for walker in mesh2.face_halfedge_iterator(&face_id2)
                {
                    if let Some(point) = find_intersection_point(mesh1, &face_id1, mesh2,&walker.halfedge_id().unwrap())
                    {
                        let edge2 = Edge::new(walker.vertex_id().unwrap(), walker.clone().twin().vertex_id().unwrap());
                        if let Some(vertex_id2) = find_close_vertex_on_edge(mesh2,&edge2, &point)
                        {
                            if let Some(vertex_id1) = find_close_vertex_on_face(mesh1, &face_id1, &point)
                            {
                                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
                            }
                            else if let Some(edge1) = find_close_edge(mesh1,&face_id1, &point)
                            {
                                intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point);
                            }
                            else {
                                intersections.face_vertex_intersections.insert((face_id1, vertex_id2), point);
                            }
                        }
                        else {
                            if let Some(vertex_id1) = find_close_vertex_on_face(mesh1, &face_id1, &point)
                            {
                                intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point);
                            }
                            else if let Some(edge1) = find_close_edge(&mesh1,&face_id1, &point)
                            {
                                intersections.edge_edge_intersections.insert((edge1, edge2), point);
                            }
                            else {
                                intersections.face_edge_intersections.insert((face_id1, edge2), point);
                            }
                        }
                    };
                }
                for walker in mesh1.face_halfedge_iterator(&face_id1)
                {
                    if let Some(point) = find_intersection_point(mesh2, &face_id2, mesh1,&walker.halfedge_id().unwrap())
                    {
                        let edge1 = Edge::new(walker.vertex_id().unwrap(), walker.clone().twin().vertex_id().unwrap());
                        if let Some(vertex_id1) = find_close_vertex_on_edge(mesh1,&edge1, &point)
                        {
                            if let Some(vertex_id2) = find_close_vertex_on_face(mesh2, &face_id2, &point)
                            {
                                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
                            }
                            else if let Some(edge2) = find_close_edge(mesh2,&face_id2, &point)
                            {
                                intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point);
                            }
                            else {
                                intersections.vertex_face_intersections.insert((vertex_id1, face_id2), point);
                            }
                        }
                        else {
                            if let Some(vertex_id2) = find_close_vertex_on_face(mesh2, &face_id2, &point)
                            {
                                intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point);
                            }
                            else if let Some(edge2) = find_close_edge(mesh2,&face_id2, &point)
                            {
                                intersections.edge_edge_intersections.insert((edge1, edge2), point);
                            }
                            else {
                                intersections.edge_face_intersections.insert((edge1, face_id2), point);
                            }
                        }
                    };
                }
            }
        }
    }
    intersections
}

const MARGIN: f32 = 0.01;

fn find_close_edge(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<Edge>
{
    let mut walker = mesh.walker_from_face(face_id);
    let vertex_id1 = walker.vertex_id().unwrap();
    walker.next();
    let vertex_id2 = walker.vertex_id().unwrap();

    if point_linesegment_distance(point, mesh.position(&vertex_id1), mesh.position(&vertex_id2)) < MARGIN {
        return Some(Edge::new(vertex_id1, vertex_id2))
    }

    walker.next();
    let vertex_id3 = walker.vertex_id().unwrap();

    if point_linesegment_distance(point, mesh.position(&vertex_id2), mesh.position(&vertex_id3)) < MARGIN {
        return Some(Edge::new(vertex_id2, vertex_id3))
    }
    if point_linesegment_distance(point, mesh.position(&vertex_id3), mesh.position(&vertex_id1)) < MARGIN {
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

fn point_linesegment_distance( point: &Vec3, p0: &Vec3, p1: &Vec3 ) -> f32
{
    let v  = p1 - p0;
    let w  = point - p0;

    let c1 = w.dot(&v);
    if c1 <= 0.0 { return w.norm(); }

    let c2 = v.dot(&v);
    if c2 <= c1 { return (point - p1).norm(); }

    let b = c1 / c2;
    let pb = p0 + b * v;
    (point - &pb).norm()
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
    }

    #[test]
    fn test_split_edges2()
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
    }

    #[test]
    fn test_simple_stitching()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();
        let stitched = stitch(&mut mesh1, &mut mesh2);
        println!("{:?}", stitched.indices());
        //assert_eq!(stitched.no_vertices(), 1);
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