use paths_types::Point3;

pub fn simplify_line_string(points: &[Point3], epsilon: f32) -> Vec<Point3> {
    let mut result = vec![points[0]];

    recurse(0, points.len() - 1, &points, epsilon, &mut result);

    result.push(points[points.len() - 1]);

    result
}

fn recurse(
    start_idx: usize,
    end_idx: usize,
    all_points: &[Point3],
    epsilon: f32,
    result: &mut Vec<Point3>,
) {
    if start_idx + 1 == end_idx {
        // Only two points left. Skip.
        return;
    }

    // Find point farthest from line between start and end points.
    let mut farthest_idx = 0;
    let mut farthest_distance = 0.0;
    let line = (&all_points[start_idx], &all_points[end_idx]);

    for idx in start_idx + 1..end_idx - 1 {
        let dist = distance_between_point_and_line(&all_points[idx], line);

        if dist > farthest_distance {
            farthest_idx = idx;
            farthest_distance = dist;
        }
    }

    // Recurse if the farthest point is outside the epsilon.
    if farthest_distance > epsilon {
        recurse(start_idx, farthest_idx, all_points, epsilon, result);

        result.push(all_points[farthest_idx]);

        recurse(farthest_idx, end_idx, all_points, epsilon, result);
    }
}

fn distance_between_point_and_line(point: &Point3, line: (&Point3, &Point3)) -> f32 {
    // Thank you, wolfram alpha.
    // https://mathworld.wolfram.com/Point-LineDistance3-Dimensional.html

    let x0 = point;
    let (x1, x2) = line;

    let double_area = (x0 - x1).cross(&(x0 - x2)).norm();
    let triangle_base_len = (x2 - x1).norm();

    double_area / triangle_base_len
}
