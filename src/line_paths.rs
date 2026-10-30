//! Editable open cubic paths. MIS exports the validated flattened vertices.
use crate::bezier::{controls, sample, segment_distance, split};
use crate::{Line, Point, Project, Result, curves::Anchor, distance, layers, length};
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
        sample(controls(&pair[0], &pair[1]), [0., 1.], 0, &mut out, 200000)?;
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
    upsert_validated(project, index, anchors)
}
fn upsert_validated(
    project: &Project,
    index: Option<usize>,
    anchors: Vec<Anchor>,
) -> Result<Project> {
    let points = flatten(&anchors)?;
    let mut next = if let Some(i) = index {
        let line = project.lines.get(i).ok_or("Selected line is unavailable")?;
        let mut lines = Vec::with_capacity(project.lines.len());
        lines.extend(project.lines[..i].iter().cloned());
        lines.push(Line {
            name: line.name.clone(),
            layer: line.layer,
            hidden: line.hidden,
            points,
            curve: Some(anchors),
        });
        lines.extend(project.lines[i + 1..].iter().cloned());
        project.with_lines(lines)
    } else {
        let mut next = project.clone();
        let layer = next
            .layers
            .iter()
            .find(|l| l.source == "Drawing" && layers::visible(&next, Some(l.id)))
            .map(|l| l.id)
            .map_or_else(|| layers::next_id(&next), Ok)?;
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
        let used: std::collections::HashSet<_> = next
            .lines
            .iter()
            .filter(|l| l.layer == Some(layer))
            .map(|l| l.name.as_str())
            .collect();
        let mut n = 1;
        while used.contains(format!("DRAW_{n}").as_str()) {
            n += 1;
        }
        next.lines.push(Line {
            name: format!("DRAW_{n}"),
            points,
            layer: Some(layer),
            hidden: false,
            curve: Some(anchors),
        });
        next
    };
    next.version = 7;
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
        sample(c, [0., 1.], 0, &mut samples, 200000)?;
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
    upsert_validated(project, Some(index), nodes)
}
fn oriented_anchors(line: &Line, reverse: bool) -> Vec<Anchor> {
    let mut nodes = anchors(line);
    if reverse {
        nodes.reverse();
        for a in &mut nodes {
            std::mem::swap(&mut a.incoming, &mut a.outgoing);
        }
    }
    nodes
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
    let mut nodes = oriented_anchors(a, ra);
    let mut tail = oriented_anchors(b, rb);
    if distance(nodes.last().unwrap().point, tail[0].point) < 1e-7 {
        nodes.last_mut().unwrap().outgoing = tail[0].outgoing;
        tail.remove(0);
    }
    nodes.extend(tail);
    let mut replacement = Some(Line {
        name: a.name.clone(),
        layer: a.layer,
        hidden: a.hidden,
        points: flatten(&nodes)?,
        curve: Some(nodes),
    });
    let lines = project
        .lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if i == ids[1] {
                None
            } else if i == ids[0] {
                replacement.take()
            } else {
                Some(line.clone())
            }
        })
        .collect();
    let mut next = project.with_lines(lines);
    next.version = 7;
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
