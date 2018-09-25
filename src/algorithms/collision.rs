
extern crate ncollide3d;

use na::{Real, Vector3};

use types::*;
use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::collision::ncollide3d::query::{proximity, Proximity};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub fn find_intersection_point(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, halfedge_id2: &HalfEdgeID) -> Option<Vec3>
{
    let mut walker = mesh2.walker_from_halfedge(halfedge_id2);
    let p0 = Point::from_coordinates(*mesh2.position(&walker.vertex_id().unwrap()));
    let p1 = Point::from_coordinates(*mesh2.position(&walker.twin().vertex_id().unwrap()));

    walker = mesh1.walker_from_face(face_id1);
    let a = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));
    walker.next();
    let b = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));
    walker.next();
    let c = Point::from_coordinates(*mesh1.position(&walker.vertex_id().unwrap()));

    triangle_line_piece_intersection(&a, &b, &c, &p0, &p1)
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

pub fn point_line_segment_distance( point: &Vec3, p0: &Vec3, p1: &Vec3 ) -> f32
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

pub fn is_point_in_triangle<N: Real>(p: &Point3<N>, p1: &Point3<N>, p2: &Point3<N>, p3: &Point3<N>) -> bool
{
    let p1p2 = *p2 - *p1;
    let p2p3 = *p3 - *p2;
    let p3p1 = *p1 - *p3;

    let p1p = *p - *p1;
    let p2p = *p - *p2;
    let p3p = *p - *p3;

    let d11 = ::na::dot(&p1p, &p1p2);
    let d12 = ::na::dot(&p2p, &p2p3);
    let d13 = ::na::dot(&p3p, &p3p1);

    d11 >= ::na::zero() && d11 <= ::na::norm_squared(&p1p2) && d12 >= ::na::zero()
        && d12 <= ::na::norm_squared(&p2p3) && d13 >= ::na::zero() && d13 <= ::na::norm_squared(&p3p1)
}

pub fn triangle_line_piece_intersection<N: Real>(a: &Point3<N>, b: &Point3<N>, c: &Point3<N>, p0: &Point3<N>, p1: &Point3<N>) -> Option<Vector3<N>>
{
    let ab = *b - *a;
    let ac = *c - *a;
    let dir = *p1 - *p0;

    // normal
    let n = ab.cross(&ac);
    let d = ::na::dot(&n, &dir);

    // the normal and the direction are orthogonal
    if d.is_zero() {
        /*if ::na::dot(&dir.cross(&(*a - p0)), &n) == ::na::zero() // TODO
        {
            if is_point_in_triangle(p0, a, b, c) { return Some(p0.coords) }
            if is_point_in_triangle(p1, a, b, c) { return Some(p1.coords) }
        }*/
        return None;
    }

    let ap = p0 - *a;
    let t = ::na::dot(&ap, &n);

    let d = d.abs();

    //
    // intersection: compute barycentric coordinates
    //
    let e = -dir.cross(&ap);

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

        toi = -t / d;
    } else {
        let v = ::na::dot(&ac, &e);

        if v < ::na::zero() || v > d {
            return None;
        }

        let w = -::na::dot(&ab, &e);

        if w < ::na::zero() || v + w > d {
            return None;
        }

        toi = t / d;
    }

    if toi < ::na::zero() && toi > ::na::one::<N>()
    {
        return None;
    }
    Some(p0.coords + dir * toi)
}