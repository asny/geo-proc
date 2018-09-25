
extern crate ncollide3d;

use na::{self, Real, Vector3};

use types::*;
use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::collision::ncollide3d::query::{proximity, Proximity, Ray};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn find_intersection_point(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, halfedge_id2: &HalfEdgeID) -> Option<Vec3>
{
    let mut walker = mesh2.walker_from_halfedge(halfedge_id2);
    let p0 = Point::from_coordinates(*mesh2.position(&walker.vertex_id().unwrap()));
    let p1 = Point::from_coordinates(*mesh2.position(&walker.twin().vertex_id().unwrap()));
    let ray = Ray::new(p0.clone(), p1 - p0);

    walker = mesh1.walker_from_face(face_id1);
    let a = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));
    walker.next();
    let b = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));
    walker.next();
    let c = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));

    triangle_ray_intersection(&a, &b, &c, &ray)
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

fn triangle_ray_intersection<N: Real>(a: &Point3<N>, b: &Point3<N>, c: &Point3<N>, ray: &Ray<N>) -> Option<Vector3<N>>
{
    let ab = *b - *a;
    let ac = *c - *a;

    // normal
    let n = ab.cross(&ac);
    let d = ::na::dot(&n, &ray.dir);

    // the normal and the ray direction are parallel
    if d.is_zero() {
        return None;
    }

    let ap = ray.origin - *a;
    let t = ::na::dot(&ap, &n);

    // the ray does not intersect the plane defined by the triangle
    if (t < ::na::zero() && d < na::zero()) || (t > ::na::zero() && d > ::na::zero()) {
        return None;
    }

    let d = d.abs();

    //
    // intersection: compute barycentric coordinates
    //
    let e = -ray.dir.cross(&ap);

    let toi;
    if t < ::na::zero() {
        let v = -::na::dot(&ac, &e);

        if v < ::na::zero() || v > d {
            return None;
        }

        let w = ::na::dot(&ab, &e);

        if w < ::na::zero() || v + w > d {
            return None;
        }

        let invd = ::na::one::<N>() / d;
        toi = -t * invd;
    } else {
        let v = ::na::dot(&ac, &e);

        if v < ::na::zero() || v > d {
            return None;
        }

        let w = -::na::dot(&ab, &e);

        if w < ::na::zero() || v + w > d {
            return None;
        }

        let invd = ::na::one::<N>() / d;
        toi = t * invd;
    }

    Some(ray.origin.coords + ray.dir * toi)
}