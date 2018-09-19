
extern crate ncollide3d;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use ids::*;
use connectivity::*;
use dynamic_mesh::DynamicMesh;

use algorithms::stitching::ncollide3d::query::{proximity, Proximity, Ray, RayCast};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let mut intersections_for_mesh1 = HashMap::new();
    let mut intersections_for_mesh2 = HashMap::new();

    let m = mesh1.clone();
    for face_id1 in mesh1.face_iterator()
    {
        let face1 = face_id_to_face(mesh1, &face_id1);
        for face_id2 in mesh2.face_iterator()
        {
            let face2 = face_id_to_face(mesh2, &face_id2);
            //println!("{} and {}", face_id1, face_id2);
            //println!("{:?}", face1);
            //println!("{:?}", face2);
            if is_intersecting(&face1, &face2)
            {
                find_intersections(&face1, &face2).iter().for_each(|i| {intersections_for_mesh1.insert(i.edge,i.point);} );
                find_intersections(&face2, &face1).iter().for_each(|i| {intersections_for_mesh2.insert(i.edge,i.point);} );
            }
        }
    }

    println!("Intersections 1: {:?}", intersections_for_mesh1);
    for intersection in intersections_for_mesh1.iter() {
        println!("Splitting on: {:?}", intersection);
        let halfedge_id = connecting_edge(mesh1, &(intersection.0).0, &(intersection.0).1).unwrap();
        mesh1.split_edge(&halfedge_id, intersection.1.coords);
    }

    println!("Intersections 2: {:?}", intersections_for_mesh2);
    for intersection in intersections_for_mesh2.iter() {
        println!("Splitting on: {:?}", intersection);
        let halfedge_id = connecting_edge(mesh2, &(intersection.0).0, &(intersection.0).1).unwrap();
        mesh2.split_edge(&halfedge_id, intersection.1.coords);
    }


    m
}

#[derive(Debug)]
struct Face
{
    pub face_id: FaceID,
    pub vertex_ids: [VertexID; 3],
    pub points: [Point; 3]
}

#[derive(Debug)]
struct Intersection
{
    pub point: Point,
    pub edge: (VertexID, VertexID)
}

fn stitch_faces(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, face_id2: &FaceID)
{

}

fn find_intersections(face: &Face, other_face: &Face) -> Vec<Intersection>
{
    let mut intersection_points = Vec::new();
    for i in 0..3 {
        let triangle = Triangle::from_array(&other_face.points);
        if let Some(point) = find_intersection_point(triangle, &face.points[i], &face.points[(i+1)%3]) {
            let v1 = face.vertex_ids[i].clone();
            let v2 = face.vertex_ids[(i+1)%3].clone();
            let edge = if v1 < v2 {(v1, v2)} else {(v2, v1)};
            intersection_points.push(Intersection {point, edge});
        };
    }
    intersection_points
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