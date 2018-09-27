
extern crate ncollide3d;

use types::*;
use ids::*;
use dynamic_mesh::DynamicMesh;

use algorithms::collision::ncollide3d::query::{proximity, Proximity};
use na::{Isometry3, Point3};

const MARGIN: f32 = 0.0001;

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

    let n = mesh1.compute_face_normal(face_id);

    match plane_line_piece_intersection(&p0, &p1, a, &n) {
        Some(PlaneLinepieceIntersectionResult::LineInPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p0 ) {
                intersections.push(Intersection{id1, id2: PrimitiveID::Vertex(edge.0), point: *p0});
            }
            if let Some(id1) = find_face_intersection(mesh1, face_id,p1 ) {
                intersections.push(Intersection{id1, id2: PrimitiveID::Vertex(edge.1), point: *p1});
            }
        },
        Some(PlaneLinepieceIntersectionResult::P0InPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p0 ) {
                intersections.push(Intersection{id1, id2: PrimitiveID::Vertex(edge.0), point: *p0});
            }
        },
        Some(PlaneLinepieceIntersectionResult::P1InPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p1 ) {
                intersections.push(Intersection{id1, id2: PrimitiveID::Vertex(edge.1), point: *p1});
            }
        },
        Some(PlaneLinepieceIntersectionResult::Intersection(point)) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id, &point ) {
                intersections.push(Intersection{id1, id2: PrimitiveID::Edge(edge.clone()), point});
            }
        },
        None => {}
    }

    intersections
}

pub fn find_edge_intersection(mesh: &DynamicMesh, edge: &(VertexID, VertexID), point: &Vec3) -> Option<PrimitiveID>
{
    let p0 = mesh.position(&edge.0);
    let p1 = mesh.position(&edge.1);
    let l0 = (point - p0).norm_squared();
    if l0 < MARGIN {
        return Some(PrimitiveID::Vertex(edge.0));
    }
    let l1 = (point - p1).norm_squared();
    if l1 < MARGIN {
        return Some(PrimitiveID::Vertex(edge.1));
    }
    if l0 + l1 < (p1 - p0).norm_squared() + MARGIN
    {
        return Some(PrimitiveID::Edge(edge.clone()));
    }
    None
}

pub fn find_face_intersection(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<PrimitiveID>
{
    let face_vertices = mesh.face_vertices(face_id);
    let v0 = face_vertices.0;
    let v1 = face_vertices.1;
    let v2 = face_vertices.2;

    let a = mesh.position(&v0);
    let b = mesh.position(&v1);
    let c = mesh.position(&v2);

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
            return Some(PrimitiveID::Face(face_id.clone()));
        }
    }
    None
}

enum PlaneLinepieceIntersectionResult
{
    P0InPlane,
    P1InPlane,
    LineInPlane,
    Intersection(Vec3)
}

fn plane_line_piece_intersection(p0: &Vec3, p1: &Vec3, a: &Vec3, n: &Vec3) -> Option<PlaneLinepieceIntersectionResult>
{
    let ap0 = *p0 - *a;
    let ap1 = *p1 - *a;

    let d0 = n.dot(&ap0);
    let d1 = n.dot(&ap1);

    if d0.abs() < MARGIN && d1.abs() < MARGIN { // p0 and p1 lies in the plane
        Some(PlaneLinepieceIntersectionResult::LineInPlane)
    }
    else if d0.abs() < MARGIN { // p0 lies in the plane
        Some(PlaneLinepieceIntersectionResult::P0InPlane)
    }
    else if d1.abs() < MARGIN { // p1 lies in the plane
        Some(PlaneLinepieceIntersectionResult::P1InPlane)
    }
    else if d0.signum() != d1.signum() // The edge intersects the plane
    {
        // Find intersection point:
        let p01 = *p1 - *p0;
        let t = n.dot(&-ap0) / n.dot(&p01);
        let point = p0 + p01 * t;
        Some(PlaneLinepieceIntersectionResult::Intersection(point))
    }
    else {
        None
    }
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

pub fn is_intersecting(mesh1: &DynamicMesh, face_id1: &FaceID, mesh2: &DynamicMesh, face_id2: &FaceID) -> bool
{
    let triangle1 = face_id_to_triangle(mesh1, face_id1);
    let triangle2 = face_id_to_triangle(mesh2, face_id2);

    let prox = proximity(&Isometry3::identity(), &triangle1,&Isometry3::identity(), &triangle2, 0.1);
    prox == Proximity::Intersecting
}

fn face_id_to_triangle(mesh: &DynamicMesh, face_id: &FaceID) -> ncollide3d::shape::Triangle<f32>
{
    let mut walker = mesh.walker_from_face(face_id);
    let p1 = Point3::<f32>::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    walker.next();
    let p2 = Point3::<f32>::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    walker.next();
    let p3 = Point3::<f32>::from_coordinates(*mesh.position(&walker.vertex_id().unwrap()));
    ncollide3d::shape::Triangle::<f32>::new(p1, p2, p3)
}

fn point_line_segment_distance( point: &Vec3, p0: &Vec3, p1: &Vec3 ) -> f32
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


// TODO: TESTS!!!!