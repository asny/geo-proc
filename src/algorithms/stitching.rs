
use std::collections::{HashMap, HashSet};

use types::*;
use ids::*;
use connectivity::*;
use collision::*;
use dynamic_mesh::DynamicMesh;

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> DynamicMesh
{
    let stitches = split_at_intersections(mesh1, mesh2);
    // Todo:
    mesh1.clone()
}

fn split_at_intersections(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> Vec<(VertexID, VertexID)>
{
    let mut intersections = find_intersections(mesh1, mesh2);

    // Split faces
    let mut new_intersections = HashMap::new();
    let mut face_splits1 = HashMap::new();
    let mut face_splits2= HashMap::new();
    for ((id1, id2), point) in intersections.drain()
    {
        if let PrimitiveID::Face(face_id) = id1
        {
            match find_type_to_split(&face_splits1, mesh1, face_id.clone(), &point) {
                PrimitiveID::Vertex(vertex_id) => { new_intersections.insert((PrimitiveID::Vertex(vertex_id), id2), point); },
                PrimitiveID::Edge(edge) => { new_intersections.insert((PrimitiveID::Edge(edge), id2), point); },
                PrimitiveID::Face(face_id) => {
                    let vertex_id = mesh1.split_face(&face_id, point);
                    insert_faces(&mut face_splits1, mesh1, face_id, &vertex_id);
                    new_intersections.insert((PrimitiveID::Vertex(vertex_id), id2), point);
                }
            }
        }
        else if let PrimitiveID::Face(face_id) = id2
        {
            match find_type_to_split(&face_splits2, mesh2, face_id.clone(), &point) {
                PrimitiveID::Vertex(vertex_id) => { new_intersections.insert((id1, PrimitiveID::Vertex(vertex_id)), point); },
                PrimitiveID::Edge(edge) => { new_intersections.insert((id1, PrimitiveID::Edge(edge)), point); },
                PrimitiveID::Face(face_id) => {
                    let vertex_id = mesh2.split_face(&face_id, point);
                    insert_faces(&mut face_splits2, mesh2, face_id, &vertex_id);
                    new_intersections.insert((id1, PrimitiveID::Vertex(vertex_id)), point);
                }
            }
        }
        else {
            new_intersections.insert((id1, id2), point);
        }
    }
    println!("Face splits: ");
    println!("1: {:?}", face_splits1);
    println!("2: {:?}", face_splits2);

    // Split edges
    let mut stitches = Vec::new();
    let mut edge_splits1 = HashMap::new();
    let mut edge_splits2 = HashMap::new();
    for ((id1, id2), point) in intersections.drain()
    {
        let vertex_id1 = match id1 {
            PrimitiveID::Vertex(vertex_id) => { vertex_id },
            PrimitiveID::Edge(edge) => {
                match find_type_to_split_edge(&edge_splits1, mesh1, edge, &point) {
                    PrimitiveID::Vertex(vertex_id) => { vertex_id },
                    PrimitiveID::Edge(ref split_edge) => {
                        let halfedge_id = connecting_edge(mesh1, &split_edge.0, &split_edge.1).unwrap();
                        let vertex_id = mesh1.split_edge(&halfedge_id, point);
                        insert_edges(&mut edge_splits1, mesh1, edge, split_edge, &vertex_id);
                        vertex_id
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };
        let vertex_id2 = match id2 {
            PrimitiveID::Vertex(vertex_id) => { vertex_id },
            PrimitiveID::Edge(edge) => {
                match find_type_to_split_edge(&edge_splits2, mesh2, edge, &point) {
                    PrimitiveID::Vertex(vertex_id) => { vertex_id },
                    PrimitiveID::Edge(ref split_edge) => {
                        let halfedge_id = connecting_edge(mesh2, &split_edge.0, &split_edge.1).unwrap();
                        let vertex_id = mesh2.split_edge(&halfedge_id, point);
                        insert_edges(&mut edge_splits2, mesh2, edge, split_edge, &vertex_id);
                        vertex_id
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };

        stitches.push((vertex_id1, vertex_id2));
    }

    stitches
}

fn find_type_to_split(face_splits: &HashMap<FaceID, HashSet<FaceID>>, mesh: &DynamicMesh, face_id: FaceID, point: &Vec3) -> PrimitiveID
{
    if let Some(new_faces) = face_splits.get(&face_id)
    {
        for new_face_id in new_faces.iter()
        {
            if is_inside(mesh, new_face_id, &point)
            {
                match find_close_type(mesh, new_face_id.clone(), &point) {
                    PrimitiveID::Vertex(vertex_id) => { return PrimitiveID::Vertex(vertex_id) },
                    PrimitiveID::Edge(edge_id) => { return PrimitiveID::Edge(edge_id) },
                    PrimitiveID::Face(fid) => { return PrimitiveID::Face(fid) }
                }
            }
        }
        panic!("ARGH")
    }
    PrimitiveID::Face(face_id)
}

fn is_inside(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> bool
{
    let mut walker = mesh.walker_from_face(face_id);
    let p0 = *mesh.position(&walker.vertex_id().unwrap());
    walker.next();
    let p1 = *mesh.position(&walker.vertex_id().unwrap());
    walker.next();
    let p2 = *mesh.position(&walker.vertex_id().unwrap());

    let a = |p0: &Vec3, p1: &Vec3, p2: &Vec3| -> f32 {(p1 - p0).cross(&(p2 - p0)).norm()};

    let f = a(&p0, &p1, &p2) - (a(&p0, &p1, point) + a(&p1, &p2, point) + a(&p2, &p0, point));
    f.abs() <= 0.001
}

fn find_type_to_split_edge(edge_splits: &HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, mesh: &DynamicMesh, edge: (VertexID, VertexID), point: &Vec3) -> PrimitiveID
{
    if let Some(new_edges) = edge_splits.get(&edge)
    {
        for new_edge in new_edges
        {
            let v1 = point - mesh.position(&new_edge.0);
            let v2 = point - mesh.position(&new_edge.1);
            if v1.dot(&v2) < MARGIN
            {
                if let Some(vertex_id) = find_close_vertex_on_edge(mesh, &edge, &point) {
                    return PrimitiveID::Vertex(vertex_id)
                }
                return PrimitiveID::Edge(new_edge.clone())
            }
        }
        panic!("ARGH")
    }
    PrimitiveID::Edge(edge)
}

fn insert_edges(edge_list: &mut HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, mesh: &DynamicMesh, edge: (VertexID, VertexID), split_edge: &(VertexID, VertexID), vertex_id: &VertexID)
{
    if !edge_list.contains_key(&edge) { edge_list.insert(edge.clone(), HashSet::new()); }
    let list = edge_list.get_mut(&edge).unwrap();
    list.insert((edge.0, vertex_id.clone()));
    list.insert((edge.1, vertex_id.clone()));
}

fn insert_faces(face_list: &mut HashMap<FaceID, HashSet<FaceID>>, mesh: &DynamicMesh, face_id: FaceID, vertex_id: &VertexID)
{
    if !face_list.contains_key(&face_id) { face_list.insert(face_id, HashSet::new()); }
    let list = face_list.get_mut(&face_id).unwrap();

    let mut iter = mesh.vertex_halfedge_iterator(vertex_id);
    list.insert(iter.next().unwrap().face_id().unwrap());
    list.insert(iter.next().unwrap().face_id().unwrap());
    list.insert(iter.next().unwrap().face_id().unwrap());
}

fn find_intersections(mesh1: &DynamicMesh, mesh2: &DynamicMesh) -> HashMap<(PrimitiveID, PrimitiveID), Vec3>
{
    let edges1 = mesh1.edge_iterator().collect();
    let edges2 = mesh2.edge_iterator().collect();
    find_intersections_between_edge_face(mesh1, &edges1, mesh2, &edges2)
}

fn find_intersections_between_edge_face(mesh1: &DynamicMesh, edges1: &Vec<(VertexID, VertexID)>, mesh2: &DynamicMesh, edges2: &Vec<(VertexID, VertexID)>) -> HashMap<(PrimitiveID, PrimitiveID), Vec3>
{
    let mut intersections: HashMap<(PrimitiveID, PrimitiveID), Vec3> = HashMap::new();
    for edge1 in edges1
    {
        for face_id2 in mesh2.face_iterator()
        {
            if let Some(point) = find_intersection_point(mesh2, &face_id2, mesh1,edge1)
            {
                let id1 = match find_close_vertex_on_edge(mesh1,&edge1, &point) {
                    Some(vertex_id1) => PrimitiveID::Vertex(vertex_id1),
                    None => PrimitiveID::Edge(*edge1)
                };
                let id2 = match find_close_type(mesh2, face_id2, &point) {
                    PrimitiveID::Vertex(vertex_id) => { PrimitiveID::Vertex(vertex_id) },
                    PrimitiveID::Edge(edge) => { PrimitiveID::Edge(edge) },
                    PrimitiveID::Face(face_id) => { PrimitiveID::Face(face_id) }
                };
                intersections.insert((id1, id2), point);
            };
        }
    }
    for edge2 in edges2
    {
        for face_id1 in mesh1.face_iterator()
        {
            if let Some(point) = find_intersection_point(mesh1, &face_id1, mesh2, edge2)
            {
                let id2 = match find_close_vertex_on_edge(mesh2,&edge2, &point) {
                    Some(vertex_id2) => PrimitiveID::Vertex(vertex_id2),
                    None => PrimitiveID::Edge(*edge2)
                };
                let id1 = match find_close_type(mesh1, face_id1, &point) {
                    PrimitiveID::Vertex(vertex_id) => { PrimitiveID::Vertex(vertex_id) },
                    PrimitiveID::Edge(edge) => { PrimitiveID::Edge(edge) },
                    PrimitiveID::Face(face_id) => { PrimitiveID::Face(face_id) }
                };
                intersections.insert((id1, id2), point);
            };
        }
    }
    println!("{:?}", intersections);
    intersections
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum PrimitiveID {
    Vertex(VertexID),
    Edge((VertexID, VertexID)),
    Face(FaceID)
}

fn find_close_type(mesh: &DynamicMesh, face_id: FaceID, point: &Vec3) -> PrimitiveID
{
    if let Some(vertex_id) = find_close_vertex_on_face(mesh, &face_id, &point)
    {
        return PrimitiveID::Vertex(vertex_id)
    }
    else if let Some(edge) = find_close_edge(mesh, &face_id, &point)
    {
        return PrimitiveID::Edge(edge)
    }
    PrimitiveID::Face(face_id)
}

const MARGIN: f32 = 0.01;

fn find_close_edge(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<(VertexID, VertexID)>
{
    let mut walker = mesh.walker_from_face(face_id);
    let vertex_id1 = walker.vertex_id().unwrap();
    walker.next();
    let vertex_id2 = walker.vertex_id().unwrap();

    if point_line_segment_distance(point, mesh.position(&vertex_id1), mesh.position(&vertex_id2)) < MARGIN {
        return Some((vertex_id1, vertex_id2))
    }

    walker.next();
    let vertex_id3 = walker.vertex_id().unwrap();

    if point_line_segment_distance(point, mesh.position(&vertex_id2), mesh.position(&vertex_id3)) < MARGIN {
        return Some((vertex_id2, vertex_id3))
    }
    if point_line_segment_distance(point, mesh.position(&vertex_id3), mesh.position(&vertex_id1)) < MARGIN {
        return Some((vertex_id3, vertex_id1))
    }
    None
}

fn find_close_vertex_on_face(mesh: &DynamicMesh, face_id: &FaceID, point: &Vec3) -> Option<VertexID>
{
    for walker in mesh.face_halfedge_iterator(face_id) {
        let vertex_id = walker.vertex_id().unwrap();
        if (mesh.position(&vertex_id) - point).norm() < MARGIN {
            return Some(vertex_id)
        }
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



#[cfg(test)]
mod tests {
    use super::*;
    use mesh::Renderable;

    #[test]
    fn test_finding_edge_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 5);

        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 0.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.25));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 1.75));
        assert!(intersections.iter().any(
            |pair| pair.1.x == 0.5 && pair.1.y == 0.0 && pair.1.z == 2.25));
    }

    #[test]
    fn test_finding_face_edge_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, -0.5, 0.0,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 2);
    }

    #[test]
    fn test_finding_face_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_edge_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![0.5, 0.0, 0.25,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_finding_vertex_vertex_intersections()
    {
        let mesh1 = create_simple_mesh_x_z();
        let indices: Vec<u32> = vec![0, 1, 2];
        let positions: Vec<f32> = vec![1.0, 0.0, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mesh2 = DynamicMesh::create(indices, positions, None);

        let intersections = find_intersections(&mesh1, &mesh2);
        assert_eq!(intersections.len(), 1);
    }

    #[test]
    fn test_split_edges()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);

        assert_eq!(stitches.len(), 5);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_faces()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 8);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh1.no_faces(), 19);
        assert_eq!(mesh1.no_halfedges(), 19 * 3 + 7);

        assert_eq!(mesh2.no_vertices(), 14);
        assert_eq!(mesh2.no_faces(), 19);
        assert_eq!(mesh2.no_halfedges(), 19 * 3 + 7);

        assert_eq!(stitches.len(), 8);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_face_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);
        let area1 = mesh1.area(&mesh1.face_iterator().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 5);
        assert_eq!(mesh1.no_halfedges(), 5 * 3 + 3);

        let mut area_test1 = 0.0;
        for face_id in mesh1.face_iterator() {
            area_test1 = area_test1 + mesh1.area(&face_id);
        }
        assert!((area1 - area_test1).abs() < 0.001);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_split_edge_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.0, -0.2, 0.5,  0.0, -0.2, 1.5,  0.0, 1.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let stitches = split_at_intersections(&mut mesh1, &mut mesh2);

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 3);
        assert_eq!(mesh1.no_halfedges(), 3 * 3 + 5);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
    }

    #[test]
    fn test_simple_stitching()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_simple_mesh_y_z();
        let stitched = stitch(&mut mesh1, &mut mesh2);

        mesh1.test_is_valid().unwrap();
        mesh2.test_is_valid().unwrap();
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

    fn create_shifted_simple_mesh_y_z() -> DynamicMesh
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.5, -0.5, -0.2,  0.5, -0.5, 0.8,  0.5, 0.5, 0.3,  0.5, 0.5, 1.3,  0.5, -0.5, 1.8,  0.5, 0.5, 2.3];
        DynamicMesh::create(indices, positions, None)
    }
}