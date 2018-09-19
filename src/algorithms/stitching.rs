
extern crate ncollide3d;

use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::stitching::ncollide3d::query::{proximity, Proximity, Ray, RayCast};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn stitch(mesh1: &DynamicMesh, mesh2: &DynamicMesh) -> DynamicMesh
{
    let m = mesh1.clone();
    for face_id1 in mesh1.face_iterator()
    {
        let face1 = face_to_triangle(mesh1, &face_id1);
        for face_id2 in mesh2.face_iterator()
        {
            let face2 = face_to_triangle(mesh2, &face_id2);
            //println!("{} and {}", face_id1, face_id2);
            //println!("{:?}", face1);
            //println!("{:?}", face2);
            if intersecting(&face1, &face2)
            {
                let points1 = intersections(&face1, &face2);
                println!("Intersections 1: {:?}", points1);
                if points1.len() == 2 {

                }

                let points2 = intersections(&face2, &face1);
                println!("Intersections 2: {:?}", points2);
            }
        }
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

fn intersections(face: &Face, test_face: &Face) -> Vec<Intersection>
{
    let mut intersection_points = Vec::new();
    for i in 0..3 {
        let triangle = Triangle::from_array(&test_face.points);
        if let Some(point) = intersection_point(triangle, &face.points[i], &face.points[(i+1)%3]) {
            let edge = (face.vertex_ids[i].clone(), face.vertex_ids[(i+1)%3].clone());
            intersection_points.push(Intersection {point, edge});
        };
    }
    intersection_points
}

fn intersection_point(triangle: &Triangle, p0: &Point, p1: &Point) -> Option<Point>
{
    let ray = Ray::new(p0.clone(), p1 - p0);
    triangle.toi_with_ray(&Isometry3::identity(), &ray, false).and_then(|toi| Some(ray.origin + ray.dir * toi))
}

fn intersecting(face1: &Face, face2: &Face) -> bool
{
    let prox = proximity(&Isometry3::identity(), Triangle::from_array(&face1.points),
                         &Isometry3::identity(), Triangle::from_array(&face2.points), 0.1);
    prox == Proximity::Intersecting
}

fn face_to_triangle(mesh: &DynamicMesh, face_id: &FaceID) -> Face
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
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();
        let stitched = stitch(&mesh1, &mesh2);
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