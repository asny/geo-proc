
extern crate ncollide3d;

use std::collections::HashMap;

use ids::*;
use connectivity::*;
use dynamic_mesh::DynamicMesh;

use algorithms::stitching::ncollide3d::query::{proximity, Proximity, Ray, RayCast};
use na::{self, Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let mut intersections = Intersections::new();

    find_intersections(&mut intersections, mesh1, mesh2);

    split(&mut intersections, mesh1, mesh2);

    mesh1.clone()
}

#[derive(Debug)]
struct Intersections
{
    pub face_edge_intersections: HashMap<(FaceID, Edge), Point>,
    pub edge_face_intersections: HashMap<(Edge, FaceID), Point>,

    pub face_vertex_intersections: HashMap<(FaceID, VertexID), Point>,
    pub vertex_face_intersections: HashMap<(VertexID, FaceID), Point>,

    pub edge_edge_intersections: HashMap<(Edge, Edge), Point>,

    pub vertex_edge_intersections: HashMap<(VertexID, Edge), Point>,
    pub edge_vertex_intersections: HashMap<(Edge, VertexID), Point>,

    pub vertex_vertex_intersections: HashMap<(VertexID, VertexID), Point>
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

fn split(intersections: &mut Intersections, mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh)
{
    for ((face_id, edge), point) in intersections.face_edge_intersections.drain() {
        let vertex_id = mesh1.split_face(&face_id, point.coords);
        intersections.vertex_edge_intersections.insert((vertex_id, edge), point);
    }

    for ((edge, face_id), point) in intersections.edge_face_intersections.drain() {
        let vertex_id = mesh2.split_face(&face_id, point.coords);
        intersections.edge_vertex_intersections.insert((edge, vertex_id), point);
    }

    // Todo: Face vertex and vertex face

    for ((edge1, edge2), point) in intersections.edge_edge_intersections.drain() {
        let halfedge_id1 = connecting_edge(mesh1, &edge1.v0, &edge1.v1).unwrap();
        let vertex_id1 = mesh1.split_edge(&halfedge_id1, point.coords);
        let halfedge_id2 = connecting_edge(mesh2, &edge2.v0, &edge2.v1).unwrap();
        let vertex_id2 = mesh2.split_edge(&halfedge_id2, point.coords);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((edge, vertex_id2), point) in intersections.edge_vertex_intersections.drain() {
        let halfedge_id = connecting_edge(mesh1, &edge.v0, &edge.v1).unwrap();
        let vertex_id1 = mesh1.split_edge(&halfedge_id, point.coords);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

    for ((vertex_id1, edge), point) in intersections.vertex_edge_intersections.drain() {
        let halfedge_id = connecting_edge(mesh2, &edge.v0, &edge.v1).unwrap();
        let vertex_id2 = mesh2.split_edge(&halfedge_id, point.coords);
        intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
    }

}

#[derive(Debug)]
struct Face
{
    pub face_id: FaceID,
    pub vertex_ids: [VertexID; 3],
    pub points: [Point; 3]
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

fn stitch_faces(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, face_id2: &FaceID)
{

}

fn find_intersections(intersections: &mut Intersections, mesh1: &DynamicMesh, mesh2: &DynamicMesh)
{
    for face_id1 in mesh1.face_iterator()
    {
        let face1 = face_id_to_face(mesh1, &face_id1);
        let triangle1 = Triangle::from_array(&face1.points);
        for face_id2 in mesh2.face_iterator()
        {
            let face2 = face_id_to_face(mesh2, &face_id2);
            let triangle2 = Triangle::from_array(&face1.points);
            if is_intersecting(triangle1, triangle2)
            {
                for i in 0..3 {
                    let p0 = &face2.points[i];
                    let p1 = &face2.points[(i+1)%3];
                    if let Some(point) = find_intersection_point(triangle1, p0, p1)
                    {
                        if na::distance(&point, p0) < 0.1
                        {
                            let vertex_id2 = face2.vertex_ids[i].clone();
                            if let Some(vertex_id1) = find_close_vertex(&face1, &point)
                            {
                                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
                            }
                            else if let Some(edge1) = find_close_edge(&face1, &point)
                            {
                                intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point);
                            }
                            else {
                                intersections.face_vertex_intersections.insert((face1.face_id, vertex_id2), point);
                            }
                        }
                        else if na::distance(&point, p1) < 0.1
                        {
                            let vertex_id2 = face2.vertex_ids[(i+1)%3].clone();
                            if let Some(vertex_id1) = find_close_vertex(&face1, &point)
                            {
                                intersections.vertex_vertex_intersections.insert((vertex_id1, vertex_id2), point);
                            }
                            else if let Some(edge1) = find_close_edge(&face1, &point)
                            {
                                intersections.edge_vertex_intersections.insert((edge1, vertex_id2), point);
                            }
                            else {
                                intersections.face_vertex_intersections.insert((face1.face_id, vertex_id2), point);
                            }
                        }
                        else {
                            let edge2 = Edge::new(face2.vertex_ids[i].clone(), face2.vertex_ids[(i+1)%3].clone());
                            if let Some(vertex_id1) = find_close_vertex(&face1, &point)
                            {
                                intersections.vertex_edge_intersections.insert((vertex_id1, edge2), point);
                            }
                            else if let Some(edge1) = find_close_edge(&face1, &point)
                            {
                                intersections.edge_edge_intersections.insert((edge1, edge2), point);
                            }
                            else {
                                intersections.face_edge_intersections.insert((face1.face_id, edge2), point);
                            }
                        }
                    };

                    // Todo: mesh 2 triangle vs mesh 1 edges
                }
            }
        }
    }
}

fn find_close_edge(face: &Face, point: &Point) -> Option<Edge>
{
    for i in 0..3 {
        let p0 = &face.points[i];
        let p1 = &face.points[(i + 1) % 3];
        if point_linesegment_distance(point, p0, p1) < 0.1 {
            return Some(Edge::new(face.vertex_ids[i], face.vertex_ids[(i + 1) % 3]))
        }
    }
    None
}

fn point_linesegment_distance( point: &Point, p0: &Point, p1: &Point ) -> f32
{
    let v  = p1 - p0;
    let w  = point - p0;

    let c1 = w.dot(&v);
    if c1 <= 0.0 { return w.norm(); }

    let c2 = v.dot(&v);
    if (c2 <= c1) { return na::distance(point, p1); }

    let b = c1 / c2;
    let pb = p0 + b * v;
    na::distance(point, &pb)
}

fn find_close_vertex(face: &Face, point: &Point) -> Option<VertexID>
{
    for i in 0..3 {
        let p = &face.points[i];
        if na::distance(p, point) < 0.1 {
            return Some(face.vertex_ids[i])
        }
    }
    None
}

fn find_intersection_point(triangle: &Triangle, p0: &Point, p1: &Point) -> Option<Point>
{
    let ray = Ray::new(p0.clone(), p1 - p0);
    triangle.toi_with_ray(&Isometry3::identity(), &ray, false).and_then(|toi| Some(ray.origin + ray.dir * toi))
}

fn is_intersecting(triangle1: &Triangle, triangle2: &Triangle) -> bool
{
    let prox = proximity(&Isometry3::identity(), triangle1,&Isometry3::identity(), triangle2, 0.1);
    prox == Proximity::Intersecting
}

fn face_id_to_face(mesh: &DynamicMesh, face_id: &FaceID) -> Face
{
    let mut points: [Point; 3] = [Point::new(0.0, 0.0, 0.0); 3];
    let mut vertex_ids = [VertexID::new(0); 3];
    let mut i = 0;
    for walker in mesh.face_halfedge_iterator(face_id) {
        let vec3 = mesh.position(&walker.vertex_id().unwrap());
        points[i] = Point::from_coordinates(*vec3);
        vertex_ids[i] = walker.vertex_id().unwrap().clone();
        i = i+1;
    }
    Face {face_id: face_id.clone(), points, vertex_ids}
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesh::Renderable;
    use types::*;

    #[test]
    fn test_finding_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();
        let mut intersections = Intersections::new();

        find_intersections(&mut intersections, &mesh1, &mesh2);
        assert_eq!(intersections.face_edge_intersections.len(), 0);
        assert_eq!(intersections.edge_face_intersections.len(), 0);
        assert_eq!(intersections.edge_edge_intersections.len(), 10);

        /*assert!(intersections.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.25)));
        assert!(intersections.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.75)));
        assert!(intersections.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.25)));
        assert!(intersections.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.75)));
        assert!(intersections.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 2.25)));

        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.25)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.75)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.25)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.75)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 2.25)));*/
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();
        let mut intersections = Intersections::new();

        find_intersections(&mut intersections, &mesh1, &mesh2);

        split(&mut intersections, &mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);
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
}