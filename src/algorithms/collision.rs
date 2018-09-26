
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
    let p01 = *p1 - *p0;
    let ap0 = *p0 - *a;
    let ap1 = *p1 - *a;

    let n = ab.cross(&ac); // normal

    let d0 = n.dot(&ap0); // Todo: Use distance to plane
    let d1 = n.dot(&ap1); // Todo: Use distance to plane

    if d0.abs() < MARGIN && d1.abs() < MARGIN { // p0 and p1 lies in the same plane as the face
        // TODO: p0 and p1 lies in the same plane as the face
        if let Some(vertex_id) = is_close_to_vertex(p0, a, b, c, &face_vertices) {
            intersections.push(Intersection {id1: PrimitiveID::Vertex(edge.0), id2: PrimitiveID::Vertex(vertex_id), point: p0.clone()});
        }
        if let Some(vertex_id) = is_close_to_vertex(p1, a, b, c, &face_vertices) {
            intersections.push(Intersection {id1: PrimitiveID::Vertex(edge.1), id2: PrimitiveID::Vertex(vertex_id), point: p1.clone()});
        }
    }
    else if d0.abs() < MARGIN { // p0 lies in the same plane as the face
        // TODO: p0 lies in the same plane as the face
        if let Some(vertex_id) = is_close_to_vertex(p0, a, b, c, &face_vertices) {
            intersections.push(Intersection {id1: PrimitiveID::Vertex(edge.0), id2: PrimitiveID::Vertex(vertex_id), point: p0.clone()});
        }
    }
    else if d1.abs() < MARGIN { // p1 lies in the same plane as the face
        // TODO: p1 lies in the same plane as the face
        if let Some(vertex_id) = is_close_to_vertex(p1, a, b, c, &face_vertices) {
            intersections.push(Intersection {id1: PrimitiveID::Vertex(edge.1), id2: PrimitiveID::Vertex(vertex_id), point: p1.clone()});
        }
    }
    else if d0.signum() == d1.signum() // The edge lies on one side of the plane spanned by the face
    {
        return intersections;
    }
    else // The edge intersects the plane spanned by the face
    {

    }

    let d = n.dot(&p01);

    let ap = p0 - *a;
    let t = ap.dot(&n);

    let d = d.abs();

    //
    // intersection: compute barycentric coordinates
    //
    let e = -p01.cross(&ap);

    let toi;
    if t < 0.0 {
        let v = -ac.dot( &e);

        if v < -MARGIN || v + MARGIN > d {
            return intersections;
        }

        let w = ab.dot( &e);

        if w < -MARGIN || v + w + MARGIN > d {
            return intersections;
        }

        toi = -t / d;
    } else {
        let v = ac.dot(&e);

        if v < -MARGIN || v + MARGIN > d {
            return intersections;
        }

        let w = -ab.dot(&e);

        if w < -MARGIN || v + w + MARGIN > d {
            return intersections;
        }

        toi = t / d;
    }

    if MARGIN < toi && toi < 1.0 - MARGIN
    {
        intersections.push(Intersection{id1: PrimitiveID::Face(face_id.clone()), id2: PrimitiveID::Edge(edge.clone()), point: p0 + p01 * toi});
        return intersections;
    }


    intersections
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