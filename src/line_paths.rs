//! Editable open cubic paths. MIS exports the validated flattened vertices.
use crate::{Line, Point, Project, Result, curves::Anchor, distance, layers, length};
const TOLERANCE: f64 = 0.1;
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
    let p = mix(e, f, t);
    ([c[0], a, e, p], [p, f, d, c[3]])
}
fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let den = d[0] * d[0] + d[1] * d[1];
    let t = if den > 0. {
        ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / den
    } else {
        0.
    };
    distance(p, mix(a, b, t.clamp(0., 1.)))
}
fn sample(c: [Point; 4], range: [f64; 2], depth: u8, out: &mut Vec<(Point, f64)>) -> Result<()> {
    if out.len() >= 200000 {
        return Err("Line curve exceeds the vertex limit.".into());
    }
    if segment_distance(c[1], c[0], c[3]).max(segment_distance(c[2], c[0], c[3])) <= TOLERANCE {
        out.push((c[3], range[1]));
        return Ok(());
    }
    if depth >= 24 {
        return Err("Line curve cannot be flattened at the required precision.".into());
    }
    let (a, b) = split(c, 0.5);
    let m = (range[0] + range[1]) / 2.;
    sample(a, [range[0], m], depth + 1, out)?;
    sample(b, [m, range[1]], depth + 1, out)
}
pub fn flatten(anchors: &[Anchor]) -> Result<Vec<Point>> {
    if !(2..=20000).contains(&anchors.len()) {
        return Err("A line needs 2 to 20,000 control points.".into());
    }
    for a in anchors {
        for p in std::iter::once(&a.point)
            .chain(a.incoming.iter())
            .chain(a.outgoing.iter())
        {
            if p.iter().any(|n| !n.is_finite() || n.abs() > 1e10) {
                return Err("Invalid line control point.".into());
            }
        }
    }
    let mut out = vec![(anchors[0].point, 0.)];
    for pair in anchors.windows(2) {
        sample(controls(&pair[0], &pair[1]), [0., 1.], 0, &mut out)?;
    }
    let mut points = Vec::with_capacity(out.len());
    for (p, _) in out {
        if points.last().is_none_or(|last| distance(*last, p) > 1e-7) {
            points.push(p);
        }
    }
    if points.len() < 2 {
        return Err("A line needs two distinct points.".into());
    }
    Ok(points)
}
pub fn anchors(line: &Line) -> Vec<Anchor> {
    line.curve.clone().unwrap_or_else(|| {
        line.points
            .iter()
            .map(|&point| Anchor {
                point,
                incoming: None,
                outgoing: None,
            })
            .collect()
    })
}
pub fn upsert(project: &Project, index: Option<usize>, anchors: Vec<Anchor>) -> Result<Project> {
    project.validate()?;
    let points = flatten(&anchors)?;
    let mut next = project.clone();
    next.version = 7;
    if let Some(i) = index {
        let line = next
            .lines
            .get_mut(i)
            .ok_or("Selected line is unavailable")?;
        line.points = points;
        line.curve = Some(anchors);
    } else {
        let layer = next
            .layers
            .iter()
            .find(|l| l.source == "Drawing" && layers::visible(&next, Some(l.id)))
            .map(|l| l.id)
            .unwrap_or(layers::next_id(&next)?);
        if !next.layers.iter().any(|l| l.id == layer) {
            next.layers.push(layers::Layer {
                id: layer,
                name: "Drawn lines".into(),
                source: "Drawing".into(),
                parent: None,
                hidden: false,
                warnings: vec![],
            });
        }
        let mut n = 1;
        while next
            .lines
            .iter()
            .any(|l| l.layer == Some(layer) && l.name == format!("DRAW_{n}"))
        {
            n += 1;
        }
        next.lines.push(Line {
            name: format!("DRAW_{n}"),
            points,
            layer: Some(layer),
            hidden: false,
            curve: Some(anchors),
        });
    }
    next.validate()?;
    Ok(next)
}
pub fn insert(project: &Project, index: usize, target: Point) -> Result<Project> {
    project.validate()?;
    if target.iter().any(|n| !n.is_finite()) {
        return Err("Invalid anchor location".into());
    }
    let line = project
        .lines
        .get(index)
        .ok_or("Selected line is unavailable")?;
    let mut nodes = anchors(line);
    let mut nearest = (f64::INFINITY, 0, [0., 1.]);
    for i in 0..nodes.len() - 1 {
        let c = controls(&nodes[i], &nodes[i + 1]);
        let mut samples = vec![(c[0], 0.)];
        sample(c, [0., 1.], 0, &mut samples)?;
        for pair in samples.windows(2) {
            let d = segment_distance(target, pair[0].0, pair[1].0);
            if d < nearest.0 {
                nearest = (d, i, [pair[0].1, pair[1].1]);
            }
        }
    }
    let (_, i, mut range) = nearest;
    let c = controls(&nodes[i], &nodes[i + 1]);
    for _ in 0..45 {
        let a = range[0] + (range[1] - range[0]) / 3.;
        let b = range[1] - (range[1] - range[0]) / 3.;
        if distance(split(c, a).0[3], target) < distance(split(c, b).0[3], target) {
            range[1] = b;
        } else {
            range[0] = a;
        }
    }
    let t = (range[0] + range[1]) / 2.;
    if t <= 1e-6 || t >= 1. - 1e-6 {
        return Err("Choose a point between existing anchors.".into());
    }
    let (a, b) = split(c, t);
    nodes[i].outgoing = Some(a[1]);
    nodes[i + 1].incoming = Some(b[2]);
    nodes.insert(
        i + 1,
        Anchor {
            point: a[3],
            incoming: Some(a[2]),
            outgoing: Some(b[1]),
        },
    );
    upsert(project, Some(index), nodes)
}
fn reverse(mut line: Line) -> Line {
    line.points.reverse();
    if let Some(nodes) = &mut line.curve {
        nodes.reverse();
        for a in nodes {
            std::mem::swap(&mut a.incoming, &mut a.outgoing);
        }
    }
    line
}
pub fn join(project: &Project, indices: &[usize]) -> Result<Project> {
    project.validate()?;
    if indices.len() != 2 || indices[0] == indices[1] {
        return Err("Select exactly two lines to join.".into());
    }
    let mut ids = indices.to_vec();
    ids.sort_unstable();
    let a = project
        .lines
        .get(ids[0])
        .ok_or("Selected line is unavailable")?;
    let b = project
        .lines
        .get(ids[1])
        .ok_or("Selected line is unavailable")?;
    let choices = [
        (
            false,
            false,
            distance(*a.points.last().unwrap(), b.points[0]),
        ),
        (
            false,
            true,
            distance(*a.points.last().unwrap(), *b.points.last().unwrap()),
        ),
        (true, false, distance(a.points[0], b.points[0])),
        (true, true, distance(a.points[0], *b.points.last().unwrap())),
    ];
    let &(ra, rb, _) = choices.iter().min_by(|a, b| a.2.total_cmp(&b.2)).unwrap();
    let first = if ra { reverse(a.clone()) } else { a.clone() };
    let second = if rb { reverse(b.clone()) } else { b.clone() };
    let mut nodes = anchors(&first);
    let mut tail = anchors(&second);
    if distance(nodes.last().unwrap().point, tail[0].point) < 1e-7 {
        nodes.last_mut().unwrap().outgoing = tail[0].outgoing;
        tail.remove(0);
    }
    nodes.extend(tail);
    let mut next = upsert(project, Some(ids[0]), nodes)?;
    next.lines.remove(ids[1]);
    if a.layer != b.layer {
        next.warnings.push("Joined line retains the first line's layer; inspect the connecting segment before export.".into());
    }
    if (length(&next.lines[ids[0]]) - length(a) - length(b)).abs() > 0.1 {
        next.warnings.push(
            "Join added a connecting segment between endpoints. Inspect it on the map.".into(),
        );
    }
    next.validate()?;
    Ok(next)
}
