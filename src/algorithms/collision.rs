
extern crate ncollide3d;

use na::{Real, Vector3};

use types::*;
use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::collision::ncollide3d::query::{proximity, Proximity};
use na::{Isometry3, Point3};

type Triangle = ncollide3d::shape::Triangle<f32>;
type Point = Point3<f32>;

pub struct Intersection {
    pub id1: PrimitiveID,
    pub id2: PrimitiveID,
    pub point: Vec3
}

pub fn find_face_edge_intersections(mesh1: &DynamicMesh, face_id: &FaceID, mesh2: &DynamicMesh, edge: &(VertexID, VertexID)) -> Vec<Intersection>
{
    let mut intersections = Vec::new();

    let p0 = mesh2.position(&edge.0);
    let p1 = mesh2.position(&edge.1);

    let face_vertices = mesh1.face_vertices(face_id);
    let a = mesh1.position(&face_vertices.0);
    let b = mesh1.position(&face_vertices.1);
    let c = mesh1.position(&face_vertices.2);

    let ab = *b - *a;
    let ac = *c - *a;
    let ap0 = *p0 - *a;
    let ap1 = *p1 - *a;

    let n = ab.cross(&ac).normalize(); // normal

    let d0 = n.dot(&ap0);
    let d1 = n.dot(&ap1);

    if d0.abs() < MARGIN || d1.abs() < MARGIN { // p0 or p1 or both lies in the same plane as the face
        if d0.abs() < MARGIN { // p0 lies in the same plane as the face
            if let Some(id1) = find_face_intersection(p0, a, b, c, face_vertices.0, face_vertices.1, face_vertices.2, face_id.clone() ) {
                let id2 = PrimitiveID::Vertex(edge.0);
                intersections.push(Intersection{id1, id2, point: *p0});
            }
        }
        if d1.abs() < MARGIN { // p1 lies in the same plane as the face
            if let Some(id1) = find_face_intersection(p1, a, b, c, face_vertices.0, face_vertices.1, face_vertices.2, face_id.clone() ) {
                let id2 = PrimitiveID::Vertex(edge.1);
                intersections.push(Intersection{id1, id2, point: *p1});
            }
        }
    }
    else if d0.signum() != d1.signum() // The edge intersects the plane spanned by the face
    {
        // Find intersection point:
        let p01 = *p1 - *p0;
        let t = n.dot(&-ap0) / n.dot(&p01);
        let point = p0 + p01 * t;

        if let Some(id1) = find_face_intersection(&point, a, b, c, face_vertices.0, face_vertices.1, face_vertices.2, face_id.clone() ) {
            let id2 = PrimitiveID::Edge(edge.clone());
            intersections.push(Intersection{id1, id2, point});
        }
    }

    intersections
}

fn find_face_intersection(point: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, v0: VertexID, v1: VertexID, v2: VertexID, face_id: FaceID) -> Option<PrimitiveID>
{
    // Compute barycentric coordinates
    let coords = barycentric(point, a, b, c);

    // Test whether the intersection point lies inside the face
    if -MARGIN < coords.0 && coords.0 < 1.0 + MARGIN && -MARGIN < coords.1 && coords.1 < 1.0 + MARGIN
        && -MARGIN < coords.2 && coords.2 < 1.0 + MARGIN // Intersection!
    {
        if coords.0 > 1.0 - MARGIN { // Through point a
            return Some(PrimitiveID::Vertex(v0));
        }
        else if coords.1 > 1.0 - MARGIN { // Through point b
            return Some(PrimitiveID::Vertex(v1));
        }
        else if coords.2 > 1.0 - MARGIN { // Through point c
            return Some(PrimitiveID::Vertex(v2));
        }
        else if coords.0 < MARGIN { // Through edge bc
            return Some(PrimitiveID::Edge((v1, v2)));
        }
        else if coords.1 < MARGIN { // Through edge ac
            return Some(PrimitiveID::Edge((v0, v2)));
        }
        else if coords.2 < MARGIN { // Through edge ab
            return Some(PrimitiveID::Edge((v0, v1)));
        }
        else { // Inside the face
            return Some(PrimitiveID::Face(face_id));
        }
    }
    None
}

// Compute barycentric coordinates (u, v, w) for
// point p with respect to triangle (a, b, c)
fn barycentric(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, f32, f32)
{
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);
    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}

const MARGIN: f32 = 0.01;
fn is_close_to_vertex(point: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, vertices: &(VertexID, VertexID, VertexID) ) -> Option<VertexID>
{
    if (point - a).norm() < MARGIN {
        return Some(vertices.0)
    }
    if (point - b).norm() < MARGIN {
        return Some(vertices.1)
    }
    if (point - c).norm() < MARGIN {
        return Some(vertices.2)
    }
    None
}

fn find_close_vertex_on_edge(mesh: &DynamicMesh, edge: &(VertexID, VertexID), point: &Vec3) -> Option<VertexID>
{
    if(point - mesh.position(&edge.0)).norm() < MARGIN
    {
        return Some(edge.0)
    }
    if (point - mesh.position(&edge.1)).norm() < MARGIN
    {
        return Some(edge.1)
    }
    None
}

pub fn find_intersection_point(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, edge: &(VertexID, VertexID)) -> Option<Vec3>
{
    let p0 = Point::from_coordinates(*mesh2.position(&edge.0));
    let p1 = Point::from_coordinates(*mesh2.position(&edge.1));

    let face_vertices = mesh1.face_vertices(face_id1);

    let a = Point::from_coordinates(*mesh1.position(&face_vertices.0));
    let b = Point::from_coordinates(*mesh1.position(&face_vertices.1));
    let c = Point::from_coordinates(*mesh1.position(&face_vertices.2));

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