
use types::*;
use dynamic_mesh::*;

const MARGIN: f32 = 0.00001;
const SQR_MARGIN: f32 = MARGIN * MARGIN;

#[derive(Debug)]
pub struct Intersection {
    pub id1: PrimitiveID,
    pub id2: PrimitiveID,
    pub point: Vec3
}

pub fn find_face_edge_intersections(mesh1: &DynamicMesh, face_id: &FaceID, mesh2: &DynamicMesh, edge: &(VertexID, VertexID)) -> Option<(Intersection, Option<Intersection>)>
{
    let p0 = mesh2.position(&edge.0);
    let p1 = mesh2.position(&edge.1);

    let p = mesh1.position(&mesh1.walker_from_face(face_id).vertex_id().unwrap());
    let n = mesh1.face_normal(face_id);

    match plane_line_piece_intersection(&p0, &p1, p, &n) {
        Some(PlaneLinepieceIntersectionResult::LineInPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p0 ) {
                if let Some(id1_) = find_face_intersection(mesh1, face_id,p1 ) {
                    return Some((Intersection{id1, id2: PrimitiveID::Vertex(edge.0), point: *p0},
                                 Some(Intersection{id1: id1_, id2: PrimitiveID::Vertex(edge.1), point: *p1})));
                }
                else {
                    return Some((Intersection{id1, id2: PrimitiveID::Vertex(edge.0), point: *p0}, None));
                }
            }
            else if let Some(id1) = find_face_intersection(mesh1, face_id,p1 ) {
                return Some((Intersection{id1, id2: PrimitiveID::Vertex(edge.1), point: *p1}, None));
            }
        },
        Some(PlaneLinepieceIntersectionResult::P0InPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p0 ) {
                return Some((Intersection{id1, id2: PrimitiveID::Vertex(edge.0), point: *p0}, None));
            }
        },
        Some(PlaneLinepieceIntersectionResult::P1InPlane) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id,p1 ) {
                return Some((Intersection{id1, id2: PrimitiveID::Vertex(edge.1), point: *p1}, None));
            }
        },
        Some(PlaneLinepieceIntersectionResult::Intersection(point)) => {
            if let Some(id1) = find_face_intersection(mesh1, face_id, &point ) {
                return Some((Intersection{id1, id2: PrimitiveID::Edge(edge.clone()), point}, None));
            }
        },
        None => {}
    }

    None
}

pub fn find_edge_intersection(mesh: &DynamicMesh, edge: &(VertexID, VertexID), point: &Vec3) -> Option<PrimitiveID>
{
    let p0 = mesh.position(&edge.0);
    let p1 = mesh.position(&edge.1);
    if (point - p0).norm_squared() < SQR_MARGIN {
        return Some(PrimitiveID::Vertex(edge.0));
    }
    if (point - p1).norm_squared() < SQR_MARGIN {
        return Some(PrimitiveID::Vertex(edge.1));
    }
    if point_line_segment_distance(point, p0, p1) < MARGIN
    {
        return Some(PrimitiveID::Edge(edge.clone()));
    }
    None
}

pub fn find_face_intersection(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<PrimitiveID>
{
    let face_vertices = mesh.ordered_face_vertices(face_id);
    let v0 = face_vertices.0;
    let v1 = face_vertices.1;
    let v2 = face_vertices.2;

    let a = mesh.position(&v0);
    let b = mesh.position(&v1);
    let c = mesh.position(&v2);

    if point_line_segment_distance(point, a, b) < MARGIN
    {
        if (*a - *point).norm_squared() < SQR_MARGIN { return Some(PrimitiveID::Vertex(v0)); }
        if (*b - *point).norm_squared() < SQR_MARGIN { return Some(PrimitiveID::Vertex(v1)); }
        return Some(PrimitiveID::Edge((v0, v1)));
    }
    if (*c - *point).norm_squared() < SQR_MARGIN { return Some(PrimitiveID::Vertex(v2)); }

    if point_line_segment_distance(point, b, c) < MARGIN { return Some(PrimitiveID::Edge((v1, v2))); }
    if point_line_segment_distance(point, a, c) < MARGIN { return Some(PrimitiveID::Edge((v0, v2))); }

    // Test whether the intersection point lies inside the face
    let coords = barycentric(point, a, b, c);
    if 0.0 < coords.0 && coords.0 < 1.0 && 0.0 < coords.1 && coords.1 < 1.0 && 0.0 < coords.2 && coords.2 < 1.0
    {
        return Some(PrimitiveID::Face(face_id.clone()));
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

fn plane_line_piece_intersection(p0: &Vec3, p1: &Vec3, p: &Vec3, n: &Vec3) -> Option<PlaneLinepieceIntersectionResult>
{
    let ap0 = *p0 - *p;
    let ap1 = *p1 - *p;

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

#[cfg(test)]
mod tests {
    use super::*;
    use dynamic_mesh::test_utility::*;

    #[test]
    fn test_find_face_intersection()
    {
        let mut mesh = create_single_face();
        mesh.scale(3.0);
        let face_id = mesh.face_iterator().next().unwrap();
        let (v0, v1, v2) = mesh.ordered_face_vertices(&face_id);
        let p0 = mesh.position(&v0);
        let p1 = mesh.position(&v1);
        let p2 = mesh.position(&v2);

        let edge_midpoint = (p1 + p2) * 0.5;
        let face_midpoint = mesh.face_center(&face_id);

        // Vertex intersection
        let mut result = find_face_intersection(&mesh, &face_id, p0);
        assert_eq!(result, Some(PrimitiveID::Vertex(v0)));

        let dir_away_from_p0 = -(0.5 * (p1 + p2) - p0).normalize();
        result = find_face_intersection(&mesh, &face_id, &(p0 + 0.9999 * MARGIN * dir_away_from_p0));
        assert_eq!(result, Some(PrimitiveID::Vertex(v0)));

        result = find_face_intersection(&mesh, &face_id, &(p0 + MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        result = find_face_intersection(&mesh, &face_id, &edge_midpoint);
        assert_eq!(result, Some(PrimitiveID::Edge((v1, v2))));

        let dir_away_from_edge = vec3(0.0, 1.0, 0.0);
        result = find_face_intersection(&mesh, &face_id, &(edge_midpoint + 0.9999 * MARGIN * dir_away_from_edge));
        assert_eq!(result, Some(PrimitiveID::Edge((v1, v2))));

        result = find_face_intersection(&mesh, &face_id, &(edge_midpoint + MARGIN * dir_away_from_edge));
        assert_eq!(result, None);

        // Face intersection
        result = find_face_intersection(&mesh, &face_id, &face_midpoint);
        assert_eq!(result, Some(PrimitiveID::Face(face_id)));
    }

    #[test]
    fn test_find_edge_intersection()
    {
        let mut mesh = create_single_face();
        mesh.scale(3.0);
        let edge_id = mesh.halfedge_iterator().next().unwrap();
        let (v0, v1) = mesh.ordered_edge_vertices(&edge_id);
        let p0 = mesh.position(&v0);
        let p1 = mesh.position(&v1);

        // Vertex intersection
        let mut result = find_edge_intersection(&mesh, &(v0, v1), p0);
        assert_eq!(result, Some(PrimitiveID::Vertex(v0)));

        let dir_away_from_p0 = -(p1 - p0).normalize();
        result = find_edge_intersection(&mesh, &(v0, v1), &(p0 + 0.9999 * MARGIN * dir_away_from_p0));
        assert_eq!(result, Some(PrimitiveID::Vertex(v0)));

        result = find_edge_intersection(&mesh, &(v0, v1), &(p0 + MARGIN * dir_away_from_p0));
        assert_eq!(result, None);

        // Edge intersection
        let edge_midpoint = (p0 + p1) * 0.5;
        result = find_edge_intersection(&mesh, &(v0, v1), &edge_midpoint);
        assert_eq!(result, Some(PrimitiveID::Edge((v0, v1))));

        let dir_away_from_edge = dir_away_from_p0.cross(&vec3(1.0, 1.0, 1.0)).normalize();
        result = find_edge_intersection(&mesh, &(v0, v1), &(edge_midpoint + 0.9999 * MARGIN * dir_away_from_edge));
        assert_eq!(result, Some(PrimitiveID::Edge((v0, v1))));

        result = find_edge_intersection(&mesh, &(v0, v1), &(edge_midpoint + MARGIN * dir_away_from_edge));
        assert_eq!(result, None);
    }
}