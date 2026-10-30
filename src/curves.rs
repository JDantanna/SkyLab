//! Editable cubic polygon paths. Geometry used by mission tools is flattened in metres.
use crate::bezier::{controls, sample, segment_distance, split};
use crate::{Point, Polygon, Project, Result, distance, layers};
use serde::{Deserialize, Serialize};
fn point(c: [Point; 4], t: f64) -> Point {
    split(c, t).0[3]
}
const MAX_POINTS: usize = crate::MAX_RING_POINTS;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Anchor {
    pub point: Point,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incoming: Option<Point>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outgoing: Option<Point>,
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
            MAX_POINTS,
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
    next.version = next.version.max(4);
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
                parent: None,
                hidden: false,
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
        let id = if let Some(layer) = next
            .layers
            .iter()
            .find(|l| l.source == "Drawing" && layers::visible(&next, Some(l.id)))
        {
            layer.id
        } else {
            let id = layers::next_id(&next)?;
            next.layers.push(layers::Layer {
                parent: None,
                hidden: false,
                id,
                name: "Drawn polygons".into(),
                source: "Drawing".into(),
                warnings: vec![],
            });
            id
        };
        next.polygons.push(Polygon {
            name: None,
            hidden: false,
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
        sample(c, [0.0, 1.0], 0, &mut samples, MAX_POINTS)?;
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
