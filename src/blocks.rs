//! Assign imported lines to the block containing their greatest acquisition length.
//! Geometry remains unchanged. Disjoint bounding boxes avoid unnecessary clipping.
use crate::{Point, Project, clip_line, distance};
fn bounds(points: &[Point]) -> [f64; 4] {
    points.iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |mut b, p| {
            b[0] = b[0].min(p[0]);
            b[1] = b[1].min(p[1]);
            b[2] = b[2].max(p[0]);
            b[3] = b[3].max(p[1]);
            b
        },
    )
}
/// The first polygon wins equal-length overlaps, independently of selection order.
/// Lines outside every block remain unassigned and accessible through their layer.
pub fn assign(project: &Project) -> Vec<Option<usize>> {
    let boxes: Vec<_> = project.polygons.iter().map(|p| bounds(&p.outer)).collect();
    project
        .lines
        .iter()
        .map(|line| {
            let b = bounds(&line.points);
            let mut best = 1e-7;
            let mut owner = None;
            for (i, polygon) in project.polygons.iter().enumerate() {
                let a = boxes[i];
                if b[2] < a[0] || b[0] > a[2] || b[3] < a[1] || b[1] > a[3] {
                    continue;
                }
                let length: f64 = clip_line(line, std::slice::from_ref(polygon))
                    .iter()
                    .flat_map(|points| points.windows(2))
                    .map(|pair| distance(pair[0], pair[1]))
                    .sum();
                if length > best {
                    best = length;
                    owner = Some(i);
                }
            }
            owner
        })
        .collect()
}
