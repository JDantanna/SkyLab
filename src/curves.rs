//! Editable cubic polygon paths. Geometry used by mission tools is flattened in metres.
use crate::{Point, Polygon, Project, Result, distance, layers};
use serde::{Deserialize, Serialize};
const TOLERANCE: f64 = 0.1;
const MAX_POINTS: usize = crate::MAX_RING_POINTS;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Anchor {
    pub point: Point,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incoming: Option<Point>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outgoing: Option<Point>,
}
fn mix(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn controls(a: &Anchor, b: &Anchor) -> [Point; 4] {
    [
        a.point,
        a.outgoing.unwrap_or(a.point),
        b.incoming.unwrap_or(b.point),
        b.point,
    ]
}
fn split(c: [Point; 4], t: f64) -> ([Point; 4], [Point; 4]) {
    let a = mix(c[0], c[1], t);
    let b = mix(c[1], c[2], t);
    let d = mix(c[2], c[3], t);
    let e = mix(a, b, t);
    let f = mix(b, d, t);
    let point = mix(e, f, t);
    ([c[0], a, e, point], [point, f, d, c[3]])
}
fn point(c: [Point; 4], t: f64) -> Point {
    split(c, t).0[3]
}
fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
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
fn sample(c: [Point; 4], range: [f64; 2], depth: u8, out: &mut Vec<(Point, f64)>) -> Result<()> {
    if out.len() >= MAX_POINTS {
        return Err("Polygon curve exceeds the vertex limit; simplify its anchors.".into());
    }
    if segment_distance(c[1], c[0], c[3]).max(segment_distance(c[2], c[0], c[3])) <= TOLERANCE {
        out.push((c[3], range[1]));
        return Ok(());
    }
    if depth >= 24 {
        return Err("Polygon curve cannot be flattened at the required precision.".into());
    }
    let (a, b) = split(c, 0.5);
    let middle = (range[0] + range[1]) / 2.0;
    sample(a, [range[0], middle], depth + 1, out)?;
    sample(b, [middle, range[1]], depth + 1, out)
}
fn validate_anchors(anchors: &[Anchor]) -> Result<()> {
    if !(3..MAX_POINTS).contains(&anchors.len()) {
        return Err("A polygon needs between 3 and 2000 anchors.".into());
    }
    for a in anchors {
        for p in std::iter::once(&a.point)
            .chain(a.incoming.iter())
            .chain(a.outgoing.iter())
        {
            if p.iter().any(|v| !v.is_finite() || v.abs() > 1e10) {
                return Err("Invalid polygon control point.".into());
            }
        }
    }
    Ok(())
}
pub fn flatten(anchors: &[Anchor]) -> Result<Vec<Point>> {
    validate_anchors(anchors)?;
    let mut out = vec![(anchors[0].point, 0.0)];
    for i in 0..anchors.len() {
        sample(
            controls(&anchors[i], &anchors[(i + 1) % anchors.len()]),
            [0.0, 1.0],
            0,
            &mut out,
        )?;
    }
    let mut ring = Vec::with_capacity(out.len());
    for (point, _) in out {
        if ring.last().is_none_or(|p| distance(*p, point) > 1e-7) {
            ring.push(point);
        }
    }
    if let Some(first) = ring.first().copied() {
        if ring.last().is_some_and(|p| distance(*p, first) > 1e-7) {
            ring.push(first);
        } else if let Some(last) = ring.last_mut() {
            *last = first;
        }
    }
    Ok(ring)
}
pub fn anchors(polygon: &Polygon) -> Vec<Anchor> {
    polygon.curve.clone().unwrap_or_else(|| {
        polygon.outer[..polygon.outer.len().saturating_sub(1)]
            .iter()
            .map(|point| Anchor {
                point: *point,
                incoming: None,
                outgoing: None,
            })
            .collect()
    })
}
pub fn upsert(project: &Project, index: Option<usize>, anchors: Vec<Anchor>) -> Result<Project> {
    project.validate()?;
    upsert_validated(project, index, anchors)
}
fn upsert_validated(
    project: &Project,
    index: Option<usize>,
    anchors: Vec<Anchor>,
) -> Result<Project> {
    let outer = flatten(&anchors)?;
    let mut next = project.clone();
    next.version = 4;
    if let Some(index) = index {
        let polygon = next
            .polygons
            .get_mut(index)
            .ok_or("Selected polygon is unavailable")?;
        polygon.outer = outer;
        polygon.curve = Some(anchors);
    } else {
        // Older projects acquire provenance before inserting a drawn polygon.
        if next.layers.is_empty() && (!next.lines.is_empty() || !next.polygons.is_empty()) {
            let id = layers::next_id(&next)?;
            next.layers.push(layers::Layer {
                id,
                name: "Imported geometry".into(),
                source: "Project".into(),
                warnings: vec![],
            });
            for line in &mut next.lines {
                line.layer = Some(id);
            }
            for polygon in &mut next.polygons {
                polygon.layer = Some(id);
            }
        }
        let id = if let Some(layer) = next.layers.iter().find(|l| l.source == "Drawing") {
            layer.id
        } else {
            let id = layers::next_id(&next)?;
            next.layers.push(layers::Layer {
                id,
                name: "Drawn polygons".into(),
                source: "Drawing".into(),
                warnings: vec![],
            });
            id
        };
        next.polygons.push(Polygon {
            layer: Some(id),
            outer,
            holes: vec![],
            curve: Some(anchors),
        });
    }
    next.validate()?;
    Ok(next)
}
pub fn insert(project: &Project, index: usize, target: Point) -> Result<Project> {
    project.validate()?;
    if target.iter().any(|v| !v.is_finite() || v.abs() > 1e10) {
        return Err("Invalid anchor location".into());
    }
    let polygon = project
        .polygons
        .get(index)
        .ok_or("Selected polygon is unavailable")?;
    let mut anchors = anchors(polygon);
    validate_anchors(&anchors)?;
    let mut nearest = (f64::INFINITY, 0usize, [0.0, 1.0]);
    for i in 0..anchors.len() {
        let c = controls(&anchors[i], &anchors[(i + 1) % anchors.len()]);
        let mut samples = vec![(c[0], 0.0)];
        sample(c, [0.0, 1.0], 0, &mut samples)?;
        for pair in samples.windows(2) {
            let d = segment_distance(target, pair[0].0, pair[1].0);
            if d < nearest.0 {
                nearest = (d, i, [pair[0].1, pair[1].1]);
            }
        }
    }
    let (_, i, mut range) = nearest;
    let c = controls(&anchors[i], &anchors[(i + 1) % anchors.len()]);
    // Refine on the actual cubic: a straight cubic need not have linear parameter speed.
    for _ in 0..45 {
        let a = range[0] + (range[1] - range[0]) / 3.0;
        let b = range[1] - (range[1] - range[0]) / 3.0;
        if distance(point(c, a), target) < distance(point(c, b), target) {
            range[1] = b;
        } else {
            range[0] = a;
        }
    }
    let t = (range[0] + range[1]) / 2.0;
    if t <= 1e-6 || t >= 1.0 - 1e-6 {
        return Err("Choose a point between existing anchors.".into());
    }
    let j = (i + 1) % anchors.len();
    let (a, b) = split(controls(&anchors[i], &anchors[j]), t);
    anchors[i].outgoing = Some(a[1]);
    anchors[j].incoming = Some(b[2]);
    anchors.insert(
        i + 1,
        Anchor {
            point: a[3],
            incoming: Some(a[2]),
            outgoing: Some(b[1]),
        },
    );
    upsert_validated(project, Some(index), anchors)
}
