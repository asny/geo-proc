
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
        let triangle1 = face_to_triangle(mesh1, &face_id1);
        for face_id2 in mesh2.face_iterator()
        {
            let triangle2 = face_to_triangle(mesh2, &face_id2);
            if intersecting(&triangle1, &triangle2)
            {
                println!("{} and {}", face_id1, face_id2);
                let p0 = intersection_point(&triangle1, triangle2.a(), triangle2.b());
                println!("{:?}", p0);
                let p1 = intersection_point(&triangle1, triangle2.b(), triangle2.c());
                println!("{:?}", p1);
                let p2 = intersection_point(&triangle1, triangle2.c(), triangle2.a());
                println!("{:?}", p2);

            }
        }
    }


    m
}

fn stitch_faces(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, face_id2: &FaceID)
{

}

fn intersection_point(triangle: &Triangle, p0: &Point, p1: &Point) -> Option<Point>
{
    let ray = Ray::new(p0.clone(), p1 - p0);
    triangle.toi_with_ray(&Isometry3::identity(), &ray, false).and_then(|toi| Some(ray.origin + ray.dir * toi))

}

fn intersecting(triangle1: &Triangle, triangle2: &Triangle) -> bool
{
    let prox = proximity(&Isometry3::identity(), triangle1, &Isometry3::identity(), triangle2, 0.1);
    prox == Proximity::Intersecting
}


fn face_to_triangle(mesh: &DynamicMesh, face_id: &FaceID) -> Triangle
{
    let mut pos = Vec::with_capacity(3);
    for walker in mesh.face_halfedge_iterator(face_id) {
        let vec3 = mesh.position(&walker.vertex_id().unwrap());
        pos.push(Point3::from_coordinates(*vec3));
    }
    Triangle::new(pos[0], pos[1], pos[2])
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