
use std::collections::{HashMap, HashSet};

use types::*;
use dynamic_mesh::*;
use collision::*;

#[derive(Debug)]
pub enum Error {
    SplittingAndMerging(splitting_and_merging::Error)
}

impl From<splitting_and_merging::Error> for Error {
    fn from(other: splitting_and_merging::Error) -> Self {
        Error::SplittingAndMerging(other)
    }
}

pub fn stitch(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> Result<DynamicMesh, Error>
{
    let stitches = split_meshes(mesh1, mesh2);
    if stitches.iter().len() == 0 { return Ok(mesh1.clone()) }

    let mut seam1 = HashMap::new();
    stitches.iter().for_each(|pair| {seam1.insert(pair.0.clone(), pair.1.clone());});
    let (mesh11, mesh12) = mesh1.split(
                                 &|mesh, halfedge_id| { is_at_seam(mesh, mesh2, &seam1, halfedge_id) });

    let mut seam2 = HashMap::new();
    stitches.iter().for_each(|pair| {seam2.insert(pair.1, pair.0);});
    let (mesh21, mesh22) = mesh2.split(
                                 &|mesh, halfedge_id| { is_at_seam(mesh, mesh1, &seam2, halfedge_id) });

    let mut m1 = if mesh11.no_faces() > mesh12.no_faces() { mesh11 } else { mesh12 };
    let m2 = if mesh21.no_faces() > mesh22.no_faces() { mesh21 } else { mesh22 };
    m1.merge_with(&m2, &seam2)?;

    Ok(m1)
}

fn is_at_seam(mesh1: &DynamicMesh, mesh2: &DynamicMesh, seam: &HashMap<VertexID, VertexID>, halfedge_id: &HalfEdgeID) -> bool
{
    let vertices = mesh1.ordered_edge_vertices(halfedge_id);
    if let Some(vertex_id1) = seam.get(&vertices.0) {
        if let Some(vertex_id2) = seam.get(&vertices.1) {
            return mesh2.connecting_edge(vertex_id1, vertex_id2).is_some()
        }
    }
    false
}

fn split_meshes(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh) -> HashSet<(VertexID, VertexID)>
{
    let mut intersections = find_intersections(mesh1, mesh2);
    let mut stitches = HashSet::new();
    while let Some((ref new_edges1, ref new_edges2)) = split_at_intersections(mesh1, mesh2, &intersections, &mut stitches)
    {
        intersections = find_intersections_between_edge_face(mesh1, new_edges1, mesh2, new_edges2);
    }
    stitches
}

fn split_at_intersections(mesh1: &mut DynamicMesh, mesh2: &mut DynamicMesh, intersections: &HashMap<(PrimitiveID, PrimitiveID), Vec3>, stitches: &mut HashSet<(VertexID, VertexID)>) -> Option<(Vec<(VertexID, VertexID)>, Vec<(VertexID, VertexID)>)>
{
    let mut new_edges1 = Vec::new();
    let mut new_edges2 = Vec::new();

    // Split faces
    let mut new_intersections: HashMap<(PrimitiveID, PrimitiveID), Vec3> = HashMap::new();
    let mut face_splits1 = HashMap::new();
    let mut face_splits2= HashMap::new();
    for ((id1, id2), point) in intersections.iter()
    {
        if let PrimitiveID::Face(face_id) = id1
        {
            match find_face_primitive_to_split(&face_splits1, mesh1, face_id.clone(), point) {
                PrimitiveID::Vertex(vertex_id) => { new_intersections.insert((PrimitiveID::Vertex(vertex_id), id2.clone()), *point); },
                PrimitiveID::Edge(edge) => { new_intersections.insert((PrimitiveID::Edge(edge), id2.clone()), *point); },
                PrimitiveID::Face(ref split_face_id) => {
                    let vertex_id = mesh1.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits1, mesh1, face_id.clone(), &vertex_id);
                    for walker in mesh1.vertex_halfedge_iterator(&vertex_id) {
                        new_edges1.push(mesh1.ordered_edge_vertices(&walker.halfedge_id().unwrap()));
                    }
                    new_intersections.insert((PrimitiveID::Vertex(vertex_id), id2.clone()), *point);
                }
            }
        }
        else if let PrimitiveID::Face(face_id) = id2
        {
            match find_face_primitive_to_split(&face_splits2, mesh2, face_id.clone(), point) {
                PrimitiveID::Vertex(vertex_id) => { new_intersections.insert((id1.clone(), PrimitiveID::Vertex(vertex_id)), *point); },
                PrimitiveID::Edge(edge) => { new_intersections.insert((id1.clone(), PrimitiveID::Edge(edge)), *point); },
                PrimitiveID::Face(ref split_face_id) => {
                    let vertex_id = mesh2.split_face(split_face_id, point.clone());
                    insert_faces(&mut face_splits2, mesh2, face_id.clone(), &vertex_id);
                    for walker in mesh2.vertex_halfedge_iterator(&vertex_id) {
                        new_edges2.push(mesh2.ordered_edge_vertices(&walker.halfedge_id().unwrap()));
                    }
                    new_intersections.insert((id1.clone(), PrimitiveID::Vertex(vertex_id)), *point);
                }
            }
        }
        else {
            new_intersections.insert((id1.clone(), id2.clone()), *point);
        }
    }

    // Split edges
    let mut edge_splits1 = HashMap::new();
    let mut edge_splits2 = HashMap::new();
    for ((id1, id2), point) in new_intersections.drain()
    {
        let vertex_id1 = match id1 {
            PrimitiveID::Vertex(vertex_id) => { vertex_id },
            PrimitiveID::Edge(edge) => {
                match find_edge_primitive_to_split(&edge_splits1, mesh1, edge, &point) {
                    PrimitiveID::Vertex(vertex_id) => { vertex_id },
                    PrimitiveID::Edge(ref split_edge) => {
                        let halfedge_id = mesh1.connecting_edge(&split_edge.0, &split_edge.1).unwrap();
                        let vertex_id = mesh1.split_edge(&halfedge_id, point);
                        insert_edges(&mut edge_splits1, edge, split_edge, &vertex_id);
                        for walker in mesh1.vertex_halfedge_iterator(&vertex_id) {
                            let vid = walker.vertex_id().unwrap();
                            if vid != split_edge.0 && vid != split_edge.1
                            {
                                new_edges1.push(mesh1.ordered_edge_vertices(&walker.halfedge_id().unwrap()));
                            }
                        }
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
                match find_edge_primitive_to_split(&edge_splits2, mesh2, edge, &point) {
                    PrimitiveID::Vertex(vertex_id) => { vertex_id },
                    PrimitiveID::Edge(ref split_edge) => {
                        let halfedge_id = mesh2.connecting_edge(&split_edge.0, &split_edge.1).unwrap();
                        let vertex_id = mesh2.split_edge(&halfedge_id, point);
                        insert_edges(&mut edge_splits2, edge, split_edge, &vertex_id);
                        for walker in mesh2.vertex_halfedge_iterator(&vertex_id) {
                            let vid = walker.vertex_id().unwrap();
                            if vid != split_edge.0 && vid != split_edge.1
                            {
                                new_edges2.push(mesh2.ordered_edge_vertices(&walker.halfedge_id().unwrap()));
                            }
                        }
                        vertex_id
                    },
                    _ => {unreachable!()}
                }
            },
            _ => {unreachable!()}
        };

        stitches.insert((vertex_id1, vertex_id2));
    }
    if new_edges1.len() > 0 && new_edges2.len() > 0 { Some((new_edges1, new_edges2)) }
    else {None}
}

fn find_face_primitive_to_split(face_splits: &HashMap<FaceID, HashSet<FaceID>>, mesh: &DynamicMesh, face_id: FaceID, point: &Vec3) -> PrimitiveID
{
    if let Some(new_faces) = face_splits.get(&face_id)
    {
        for new_face_id in new_faces.iter()
        {
            if let Some(id) = find_face_intersection(mesh, new_face_id, point) { return id; }
        }
        unreachable!()
    }
    PrimitiveID::Face(face_id)
}

fn find_edge_primitive_to_split(edge_splits: &HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, mesh: &DynamicMesh, edge: (VertexID, VertexID), point: &Vec3) -> PrimitiveID
{
    if let Some(new_edges) = edge_splits.get(&edge)
    {
        for new_edge in new_edges
        {
            if let Some(id) = find_edge_intersection(mesh, new_edge, point) { return id; }
        }
        unreachable!()
    }
    PrimitiveID::Edge(edge)
}

fn insert_edges(edge_list: &mut HashMap<(VertexID, VertexID), HashSet<(VertexID, VertexID)>>, edge: (VertexID, VertexID), split_edge: &(VertexID, VertexID), vertex_id: &VertexID)
{
    if !edge_list.contains_key(&edge) { edge_list.insert(edge.clone(), HashSet::new()); }
    let list = edge_list.get_mut(&edge).unwrap();
    list.remove(split_edge);
    list.insert((split_edge.0, vertex_id.clone()));
    list.insert((split_edge.1, vertex_id.clone()));
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
            if let Some(result) = find_face_edge_intersections(mesh2, &face_id2, mesh1,edge1)
            {
                let intersection = result.0;
                intersections.insert((intersection.id2, intersection.id1), intersection.point);
                if let Some(other_intersection) = result.1
                {
                    intersections.insert((other_intersection.id2, other_intersection.id1), other_intersection.point);
                }
            }
        }
    }
    for edge2 in edges2
    {
        for face_id1 in mesh1.face_iterator()
        {
            if let Some(result) = find_face_edge_intersections(mesh1, &face_id1, mesh2, edge2)
            {
                let intersection = result.0;
                intersections.insert((intersection.id1, intersection.id2), intersection.point);
                if let Some(other_intersection) = result.1
                {
                    intersections.insert((other_intersection.id1, other_intersection.id2), other_intersection.point);
                }
            }
        }
    }
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;
    use dynamic_mesh::test_utility::*;

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

        let intersections = find_intersections(&mesh1, &mesh2);
        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 11);
        assert_eq!(mesh1.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh1.no_faces(), 12);

        assert_eq!(mesh2.no_vertices(), 11);
        assert_eq!(mesh2.no_halfedges(), 12 * 3 + 8);
        assert_eq!(mesh2.no_faces(), 12);

        assert_eq!(stitches.len(), 5);
        assert_eq!(new_edges1.len(), 8);
        assert_eq!(new_edges2.len(), 8);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_split_faces()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 8);

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 14);
        assert_eq!(mesh1.no_faces(), 19);
        assert_eq!(mesh1.no_halfedges(), 19 * 3 + 7);

        assert_eq!(mesh2.no_vertices(), 14);
        assert_eq!(mesh2.no_faces(), 19);
        assert_eq!(mesh2.no_halfedges(), 19 * 3 + 7);

        assert_eq!(stitches.len(), 8);
        assert_eq!(new_edges1.len(), 19);
        assert_eq!(new_edges2.len(), 19);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

    }

    #[test]
    fn test_split_face_two_times()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);
        let area1 = mesh1.face_area(&mesh1.face_iterator().next().unwrap());

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let intersections = find_intersections(&mesh1, &mesh2);

        assert_eq!(intersections.len(), 2);

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 5);
        assert_eq!(mesh1.no_halfedges(), 5 * 3 + 3);

        let mut area_test1 = 0.0;
        for face_id in mesh1.face_iterator() {
            area_test1 = area_test1 + mesh1.face_area(&face_id);
        }
        assert!((area1 - area_test1).abs() < 0.001);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);
        assert_eq!(new_edges1.len(), 6);
        assert_eq!(new_edges2.len(), 2);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
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

        let mut stitches = HashSet::new();
        let (new_edges1, new_edges2) = split_at_intersections(&mut mesh1, &mut mesh2, &intersections, &mut stitches).unwrap();

        assert_eq!(mesh1.no_vertices(), 5);
        assert_eq!(mesh1.no_faces(), 3);
        assert_eq!(mesh1.no_halfedges(), 3 * 3 + 5);

        assert_eq!(mesh2.no_vertices(), 5);
        assert_eq!(mesh2.no_faces(), 3);
        assert_eq!(mesh2.no_halfedges(), 3 * 3 + 5);

        assert_eq!(stitches.len(), 2);
        assert_eq!(new_edges1.len(), 2);
        assert_eq!(new_edges2.len(), 2);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_face_face_splitting()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![0.2, -0.2, 0.5,  0.5, 0.5, 0.75,  0.5, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let stitches = split_meshes(&mut mesh1, &mut mesh2);

        assert_eq!(stitches.len(), 2);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_simple_simple_splitting()
    {
        let mut mesh1 = create_simple_mesh_x_z();
        let mut mesh2 = create_shifted_simple_mesh_y_z();

        let stitches = split_meshes(&mut mesh1, &mut mesh2);

        assert_eq!(stitches.len(), 8);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_box_box_splitting()
    {
        let mut mesh1 = ::models::create_cube().unwrap().to_dynamic();
        let mut mesh2 = ::models::create_cube().unwrap().to_dynamic();
        for vertex_id in mesh2.vertex_iterator() {
            mesh2.move_vertex(vertex_id, vec3(0.5, 0.5, 0.5));
        }
        split_meshes(&mut mesh1, &mut mesh2);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0,  -2.0, 0.0, -2.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let stitched = stitch(&mut mesh1, &mut mesh2).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        assert_eq!(stitched.no_faces(), 2);
        assert_eq!(stitched.no_vertices(), 4);
        test_is_valid(&stitched).unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_mid_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = DynamicMesh::create(indices1, positions1, None);

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 1.0,  -2.0, 0.0, -1.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = DynamicMesh::create(indices2, positions2, None);

        let stitched = stitch(&mut mesh1, &mut mesh2).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        assert_eq!(stitched.no_faces(), 4);
        assert_eq!(stitched.no_vertices(), 6);
        test_is_valid(&stitched).unwrap();
    }

    #[test]
    fn test_box_box_stitching()
    {
        let mut mesh1 = ::models::create_cube().unwrap().to_dynamic();
        let mut mesh2 = ::models::create_cube().unwrap().to_dynamic();
        for vertex_id in mesh2.vertex_iterator() {
            mesh2.move_vertex(vertex_id, vec3(0.5, 0.5, 0.5));
        }
        let stitched = stitch(&mut mesh1, &mut mesh2).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        test_is_valid(&stitched).unwrap();
    }

    #[test]
    fn test_sphere_box_stitching()
    {
        let mut mesh1 = ::models::create_sphere(1).unwrap().to_dynamic();
        mesh1.translate(&vec3(0.0, 1.5, 0.0));
        mesh1.update_vertex_normals();
        let mut mesh2 = ::models::create_cube().unwrap().to_dynamic();
        mesh2.translate(&vec3(0.5, 2.0, 0.5));
        mesh2.update_vertex_normals();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        let stitched = stitch(&mut mesh1, &mut mesh2).unwrap();

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();

        test_is_valid(&stitched).unwrap();
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