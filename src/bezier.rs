//! Shared adaptive cubic geometry; 0.1 metre tolerance and bounded subdivision.
use crate::{Point, Result, curves::Anchor, distance};
const TOLERANCE: f64 = 0.1;
fn mix(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
pub(crate) fn controls(a: &Anchor, b: &Anchor) -> [Point; 4] {
    [
        a.point,
        a.outgoing.unwrap_or(a.point),
        b.incoming.unwrap_or(b.point),
        b.point,
    ]
}
pub(crate) fn split(c: [Point; 4], t: f64) -> ([Point; 4], [Point; 4]) {
    let a = mix(c[0], c[1], t);
    let b = mix(c[1], c[2], t);
    let d = mix(c[2], c[3], t);
    let e = mix(a, b, t);
    let f = mix(b, d, t);
    let point = mix(e, f, t);
    ([c[0], a, e, point], [point, f, d, c[3]])
}
pub(crate) fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let den = dx * dx + dy * dy;
    let t = if den > 0.0 {
        ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / den
    } else {
        0.0
    };
    distance(p, mix(a, b, t.clamp(0.0, 1.0)))
}
pub(crate) fn sample(
    c: [Point; 4],
    range: [f64; 2],
    depth: u8,
    out: &mut Vec<(Point, f64)>,
    limit: usize,
) -> Result<()> {
    if out.len() >= limit {
        return Err("Curve exceeds the vertex limit.".into());
    }
    if segment_distance(c[1], c[0], c[3]).max(segment_distance(c[2], c[0], c[3])) <= TOLERANCE {
        out.push((c[3], range[1]));
        return Ok(());
    }
    if depth >= 24 {
        return Err("Curve cannot be flattened at the required precision.".into());
    }
    let (a, b) = split(c, 0.5);
    let middle = (range[0] + range[1]) / 2.0;
    sample(a, [range[0], middle], depth + 1, out, limit)?;
    sample(b, [middle, range[1]], depth + 1, out, limit)
}
