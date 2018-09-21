
extern crate ncollide3d;

use types::*;
use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::collision::ncollide3d::query::{proximity, Proximity, Ray, RayCast};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn find_intersection_point(mesh: &DynamicMesh, face_id: &FaceID, p0: &Vec3, p1: &Vec3) -> Option<Vec3>
{
    let triangle = face_id_to_triangle(mesh, face_id);
    let p0_ = Point::from_coordinates(*p0);
    let p1_ = Point::from_coordinates(*p1);
    let ray = Ray::new(p0_.clone(), p1_ - p0_);
    triangle.toi_with_ray(&Isometry3::identity(), &ray, false).and_then(|toi| Some(ray.origin.coords + ray.dir * toi))
}

pub fn is_intersecting(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, face_id2: &FaceID) -> bool
{
    let triangle1 = face_id_to_triangle(mesh1, face_id1);
    let triangle2 = face_id_to_triangle(mesh2, face_id2);

    let prox = proximity(&Isometry3::identity(), &triangle1,&Isometry3::identity(), &triangle2, 0.1);
    prox == Proximity::Intersecting
}

fn face_id_to_triangle(mesh: &DynamicMesh, face_id: &FaceID) -> Triangle
{
    let mut walker = mesh.walker_from_face(face_id);
    let p1 = Point::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    walker.next();
    let p2 = Point::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    walker.next();
    let p3 = Point::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    Triangle::new(p1, p2, p3)
}