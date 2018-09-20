
extern crate ncollide3d;

use std::collections::HashMap;

use ids::*;
use connectivity::*;
use dynamic_mesh::DynamicMesh;

use algorithms::stitching::ncollide3d::query::{proximity, Proximity, Ray, RayCast};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let mut intersections = Intersections::new();
    let mut intersections_for_mesh1 = HashMap::new();
    let mut intersections_for_mesh2 = HashMap::new();

    find_intersections(mesh1, &mut intersections_for_mesh1, mesh2, &mut intersections_for_mesh2);

    split_edges(mesh1, &intersections_for_mesh1);
    split_edges(mesh2, &intersections_for_mesh2);

    mesh1.clone()
}

#[derive(Debug)]
struct Intersections
{
    pub face_edge_intersections: HashMap<(FaceID, Edge), Point>,
    pub edge_face_intersections: HashMap<(Edge, FaceID), Point>,

    pub vertex_edge_intersections: HashMap<(VertexID, Edge), Point>,
    pub edge_vertex_intersections: HashMap<(Edge, VertexID), Point>
}

impl Intersections
{
    pub fn new() -> Intersections
    {
        Intersections {face_edge_intersections: HashMap::new(), edge_face_intersections: HashMap::new(),
            vertex_edge_intersections: HashMap::new(), edge_vertex_intersections: HashMap::new()}
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

fn split_edges(mesh: &mut DynamicMesh, intersections_for_mesh: &HashMap<Edge, Point>)
{
    for intersection in intersections_for_mesh.iter() {
        let halfedge_id = connecting_edge(mesh, &(intersection.0).v0, &(intersection.0).v1).unwrap();
        mesh.split_edge(&halfedge_id, intersection.1.coords);
    }
}

fn find_intersections(mesh1: &DynamicMesh, intersections_for_mesh1: &mut HashMap<Edge, Point>, mesh2: &DynamicMesh, intersections_for_mesh2: &mut HashMap<Edge, Point>)
{
    for face_id1 in mesh1.face_iterator()
    {
        let face1 = face_id_to_face(mesh1, &face_id1);
        for face_id2 in mesh2.face_iterator()
        {
            let face2 = face_id_to_face(mesh2, &face_id2);
            if is_intersecting(&face1, &face2)
            {
                add_intersections(intersections_for_mesh1,&face1, &face2);
                add_intersections(intersections_for_mesh2,&face2, &face1);
            }
        }
    }
}

fn add_intersections(intersections: &mut HashMap<Edge, Point>, face: &Face, other_face: &Face)
{
    for i in 0..3 {
        let triangle = Triangle::from_array(&other_face.points);
        if let Some(point) = find_intersection_point(triangle, &face.points[i], &face.points[(i+1)%3]) {
            let edge = Edge::new(face.vertex_ids[i].clone(), face.vertex_ids[(i+1)%3].clone());
            intersections.insert(edge, point);
        };
    }
}

fn find_intersection_point(triangle: &Triangle, p0: &Point, p1: &Point) -> Option<Point>
{
    let ray = Ray::new(p0.clone(), p1 - p0);
    triangle.toi_with_ray(&Isometry3::identity(), &ray, false).and_then(|toi| Some(ray.origin + ray.dir * toi))
}

fn is_intersecting(face1: &Face, face2: &Face) -> bool
{
    let prox = proximity(&Isometry3::identity(), Triangle::from_array(&face1.points),
                         &Isometry3::identity(), Triangle::from_array(&face2.points), 0.1);
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
        let mut intersections_for_mesh1 = HashMap::new();
        let mut intersections_for_mesh2 = HashMap::new();

        find_intersections(&mesh1, &mut intersections_for_mesh1, &mesh2, &mut intersections_for_mesh2);
        assert_eq!(intersections_for_mesh1.len(), 5);
        assert_eq!(intersections_for_mesh2.len(), 5);

        assert!(intersections_for_mesh1.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.25)));
        assert!(intersections_for_mesh1.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.75)));
        assert!(intersections_for_mesh1.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.25)));
        assert!(intersections_for_mesh1.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.75)));
        assert!(intersections_for_mesh1.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 2.25)));

        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.25)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 0.75)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.25)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 1.75)));
        assert!(intersections_for_mesh2.iter().any(|pair| pair.1.coords == vec3(0.5, 0.0, 2.25)));
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();
        let mut intersections_for_mesh1 = HashMap::new();
        let mut intersections_for_mesh2 = HashMap::new();

        find_intersections(&mesh1, &mut intersections_for_mesh1, &mesh2, &mut intersections_for_mesh2);

        split_edges(&mut mesh1, &intersections_for_mesh1);
        split_edges(&mut mesh2, &intersections_for_mesh2);

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