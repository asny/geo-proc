use ids::*;
use dynamic_mesh::DynamicMesh;
use mesh::Renderable;

pub fn stitch(mesh1: &DynamicMesh, mesh2: &DynamicMesh) -> DynamicMesh
{
    mesh1.clone()



}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_connected_component()
    {
        let mesh1 = create_simple_mesh_x_z();
        let mesh2 = create_simple_mesh_y_z();
        let stitched = stitch(&mesh1, &mesh2);
        println!("{:?}", stitched.indices());
        //assert_eq!(stitched.no_vertices(), 1);
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
        let positions: Vec<f32> = vec![0.0, -0.5, 0.0,  0.0, -0.5, 1.0,  0.0, 0.5, 0.5,  0.0, 0.5, 1.5,  0.0, -0.5, 2.0,  0.0, 0.5, 2.5];
        DynamicMesh::create(indices, positions, None)
    }
}