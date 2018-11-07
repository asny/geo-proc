use types::*;

pub fn indices_from_positions(positions: &Vec<f32>) -> (Vec<u32>, Vec<f32>)
{
    let mut indices = vec![None; positions.len()/3];
    let mut positions_out = Vec::new();

    for i in 0..positions.len()/3 {
        if indices[i].is_none()
        {
            let p1 = vec3(positions[3 * i], positions[3 * i + 1], positions[3 * i + 2]);
            positions_out.push(p1.x);
            positions_out.push(p1.y);
            positions_out.push(p1.z);
            let current_index = Some((positions_out.len() / 3 - 1) as u32);
            indices[i] = current_index;
            for j in i+1..positions.len()/3 {
                let p2 = vec3(positions[3 * j], positions[3 * j + 1], positions[3 * j + 2]);
                if (p1 - p2).norm() < 0.001 {
                    indices[j] = current_index;
                }
            }
        }
    }

    (indices.iter().map(|x| x.unwrap()).collect(), positions_out)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_indices_from_positions()
    {
        let positions: Vec<f32> = vec![0.0, 0.0, 0.0,  1.0, 0.0, -0.5,  -1.0, 0.0, -0.5,
                                       0.0, 0.0, 0.0,  -1.0, 0.0, -0.5, 0.0, 0.0, 1.0,
                                       0.0, 0.0, 0.0,  0.0, 0.0, 1.0,  1.0, 0.0, -0.5];

        let (indices, positions_out) = indices_from_positions(&positions);

        let positions_result: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0, 0.0, -0.5, -1.0, 0.0, -0.5, 0.0, 0.0, 1.0];
        let indices_result: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 0, 3, 1];

        assert_eq!(indices, indices_result);
        assert_eq!(positions_result, positions_out);
    }
}