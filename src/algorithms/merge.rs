use crate::prelude::*;

#[derive(Debug)]
pub enum Error {
    MergeWillCreateNonManifoldMesh {message: String},
    MergeOverlappingPrimitives(crate::mesh::merge_overlapping_primitives::Error),
    CannotCheckOrientationOfMesh {message: String},
}

impl From<crate::mesh::merge_overlapping_primitives::Error> for Error {
    fn from(other: crate::mesh::merge_overlapping_primitives::Error) -> Self {
        Error::MergeOverlappingPrimitives(other)
    }
}

impl Mesh
{
    pub fn merge_with(&mut self, other: &Self) -> Result<(), Error>
    {
        self.append(other);
        self.merge_overlapping_primitives()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utility::*;
    use crate::MeshBuilder;

    #[test]
    fn test_face_face_merging_at_edge()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_indices(indices1).with_positions(positions1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.0, -2.0, -2.0, 0.5, 0.0];
        let mesh2 = MeshBuilder::new().with_indices(indices2).with_positions(positions2).build().unwrap();

        mesh1.merge_with(&mesh2).unwrap();

        assert_eq!(mesh1.no_faces(), 2);
        assert_eq!(mesh1.no_vertices(), 4);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }

    #[test]
    fn test_face_face_merging_at_edge_when_orientation_is_opposite()
    {
        let indices1: Vec<u32> = vec![0, 1, 2];
        let positions1: Vec<f32> = vec![-2.0, 0.0, -2.0, -2.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let mut mesh1 = MeshBuilder::new().with_indices(indices1).with_positions(positions1).build().unwrap();

        let indices2: Vec<u32> = vec![0, 1, 2];
        let positions2: Vec<f32> = vec![-2.0, 0.0, 2.0, -2.0, 0.5, 0.0, -2.0, 0.0, -2.0];
        let mesh2 = MeshBuilder::new().with_indices(indices2).with_positions(positions2).build().unwrap();

        mesh1.merge_with(&mesh2).unwrap();

        assert_eq!(mesh1.no_faces(), 2);
        assert_eq!(mesh1.no_vertices(), 4);

        test_is_valid(&mesh1).unwrap();
        test_is_valid(&mesh2).unwrap();
    }
}