
use tri_mesh::prelude::*;
use crate::connected_components::*;
use crate::split_primitives_at_intersection::*;

pub fn cut(mesh: &Mesh, is_at_cut: &Fn(&Mesh, HalfEdgeID) -> bool) -> Vec<Mesh>
{
    let components = connected_components_with_limit(mesh, &|halfedge_id| is_at_cut(mesh, halfedge_id));
    components.iter().map(|cc| mesh.clone_subset(cc)).collect()
}

pub fn cut_at_intersection(mesh1: &mut Mesh, mesh2: &mut Mesh) -> Result<(Vec<Mesh>, Vec<Mesh>), Error>
{
    split_primitives_at_intersection(mesh1, mesh2)?;
    let meshes1 = cut(&mesh1, &|mesh, halfedge_id| is_at_intersection(mesh, mesh2, halfedge_id));
    let meshes2 = cut(&mesh2, &|mesh, halfedge_id| is_at_intersection(mesh, mesh1, halfedge_id));
    Ok((meshes1, meshes2))
}

fn is_at_intersection(mesh1: &Mesh, mesh2: &Mesh, halfedge_id: HalfEdgeID) -> bool
{
    let (p10, p11) = mesh1.edge_positions(halfedge_id);
    for halfedge_id2 in mesh2.edge_iter() {
        let (p20, p21) = mesh2.edge_positions(halfedge_id2);
        if point_and_point_intersects(p10, p20) && point_and_point_intersects(p11, p21) ||
            point_and_point_intersects(p11, p20) && point_and_point_intersects(p10, p21)
        {
            if mesh1.is_edge_on_boundary(halfedge_id) || mesh2.is_edge_on_boundary(halfedge_id2) {
                return true;
            }
            let mut walker1 = mesh1.walker_from_halfedge(halfedge_id);
            let mut walker2 = mesh2.walker_from_halfedge(halfedge_id2);
            let face_id10 = walker1.face_id().unwrap();
            let face_id11 = walker1.as_twin().face_id().unwrap();
            let face_id20 = walker2.face_id().unwrap();
            let face_id21 = walker2.as_twin().face_id().unwrap();
            if (!mesh1.face_and_face_overlaps(face_id10, mesh2, face_id20) &&
                !mesh1.face_and_face_overlaps(face_id10, mesh2, face_id21)) ||
                (!mesh1.face_and_face_overlaps(face_id11, mesh2, face_id20) &&
                !mesh1.face_and_face_overlaps(face_id11, mesh2, face_id21))
            {
                return true;
            }
        }
    }
    false
}

fn point_and_point_intersects(point1: &Vec3, point2: &Vec3) -> bool
{
    const MARGIN: f32 = 0.00001;
    const SQR_MARGIN: f32 = MARGIN * MARGIN;
    (point1 - point2).magnitude2() < SQR_MARGIN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut()
    {
        let indices: Vec<u32> = vec![0, 1, 2,  2, 1, 3,  3, 1, 4,  3, 4, 5];
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, 0.5,  1.0, 0.0, 1.5,  0.0, 0.0, 2.0,  1.0, 0.0, 2.5];
        let mesh = MeshBuilder::new().with_indices(indices).with_positions(positions).build().unwrap();

        let meshes = cut(&mesh, &|mesh,
                                  he_id| {
                let (p0, p1) = mesh.edge_positions(he_id);
                p0.z > 0.75 && p0.z < 1.75 && p1.z > 0.75 && p1.z < 1.75
            });

        assert_eq!(meshes.len(), 2);
        let m1 = &meshes[0];
        let m2 = &meshes[1];

        mesh.is_valid().unwrap();
        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m2.no_faces(), 2);
    }

    #[test]
    fn test_face_face_stitching_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0,  -2.0, 0.0, -2.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let (meshes1, meshes2) = cut_at_intersection(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 2);
        assert_eq!(m1.no_vertices(), 4);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_face_face_stitching_at_mid_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0,  -2.0, 0.0, 2.0,  2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_positions(positions1).with_indices(indices1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 1.0,  -2.0, 0.0, -1.0,  -2.0, 0.5, 0.0];
        let mut mesh2 = MeshBuilder::new().with_positions(positions2).with_indices(indices2).build().unwrap();

        let (meshes1, meshes2) = cut_at_intersection(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 1);
        assert_eq!(meshes2.len(), 1);

        let mut m1 = meshes1[0].clone();
        let m2 = meshes2[0].clone();
        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        assert_eq!(m1.no_faces(), 4);
        assert_eq!(m1.no_vertices(), 6);

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_box_box_stitching()
    {
        let mut mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 0.5, 0.5));

        let (meshes1, meshes2) = cut_at_intersection(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() { meshes1[0].clone() } else { meshes1[1].clone() };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() { meshes2[0].clone() } else { meshes2[1].clone() };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_sphere_box_stitching()
    {
        let mut mesh1 = MeshBuilder::new().icosahedron().build().unwrap();
        for _ in 0..1 {
            for face_id in mesh1.face_iter() {
                let p = mesh1.face_center(face_id).normalize();
                mesh1.split_face(face_id, p);
            }
            mesh1.smooth_vertices(1.0);
            for vertex_id in mesh1.vertex_iter() {
                let p = mesh1.vertex_position(vertex_id).normalize();
                mesh1.move_vertex_to(vertex_id, p)
            }
            mesh1.flip_edges(0.5);
        }
        mesh1.translate(vec3(0.0, 1.5, 0.0));
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.5, 2.0, 0.5));

        let (meshes1, meshes2) = cut_at_intersection(&mut mesh1, &mut mesh2).unwrap();
        assert_eq!(meshes1.len(), 2);
        assert_eq!(meshes2.len(), 2);

        let mut m1 = if meshes1[0].no_faces() > meshes1[1].no_faces() { meshes1[0].clone() } else { meshes1[1].clone() };
        let m2 = if meshes2[0].no_faces() > meshes2[1].no_faces() { meshes2[0].clone() } else { meshes2[1].clone() };

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();

        m1.merge_with(&m2).unwrap();

        mesh1.is_valid().unwrap();
        mesh2.is_valid().unwrap();

        m1.is_valid().unwrap();
        m2.is_valid().unwrap();
    }

    #[test]
    fn test_is_at_intersection_cube_cube()
    {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();
        let mut mesh2 = MeshBuilder::new().cube().build().unwrap();
        mesh2.translate(vec3(0.0, 2.0, 0.0));

        let result = connected_components_with_limit(&mesh1, &|halfedge_id| is_at_intersection(&mesh1, &mesh2, halfedge_id));

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());
        assert!(result.iter().find(|cc| cc.len() == 10).is_some());
    }

    #[test]
    fn test_is_at_intersection()
    {
        let mesh1 = MeshBuilder::new().cube().build().unwrap();

        let positions = vec![-1.0, 1.0, 1.0,  -1.0, -1.0, 1.0,  1.0, -1.0, -1.0,  1.0, 1.0, -1.0, 0.0, 2.0, 0.0 ];
        let indices = vec![0, 1, 2,  0, 2, 3,  0, 3, 4];
        let mesh2 = MeshBuilder::new().with_positions(positions).with_indices(indices).build().unwrap();

        let result = connected_components_with_limit(&mesh2, &|halfedge_id| is_at_intersection(&mesh2, &mesh1, halfedge_id));

        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|cc| cc.len() == 1).is_some());
        assert!(result.iter().find(|cc| cc.len() == 2).is_some());
    }
}