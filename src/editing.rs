//! Split surveyed polylines at a drawn knife stroke, preserving all original vertices.
use crate::{Point, Project, Result, distance, length, trim_line};
use std::collections::HashSet;
const TOLERANCE: f64 = 1e-7;
fn cross(a: Point, b: Point) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}
fn bounds(points: &[Point]) -> [f64; 4] {
    points.iter().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |b, p| {
            [
                b[0].min(p[0]),
                b[1].min(p[1]),
                b[2].max(p[0]),
                b[3].max(p[1]),
            ]
        },
    )
}
fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] <= b[2] + TOLERANCE
        && b[0] <= a[2] + TOLERANCE
        && a[1] <= b[3] + TOLERANCE
        && b[1] <= a[3] + TOLERANCE
}
/// Collinear overlaps do not cut: a knife must cross or touch an individual point.
fn intersection(a: Point, b: Point, c: Point, d: Point) -> Option<f64> {
    let r = sub(b, a);
    let s = sub(d, c);
    let denominator = cross(r, s);
    if denominator.abs() <= 1e-12 * distance(a, b) * distance(c, d) {
        return None;
    }
    let q = sub(c, a);
    let t = cross(q, s) / denominator;
    let u = cross(q, r) / denominator;
    if (-TOLERANCE..=1. + TOLERANCE).contains(&t) && (-TOLERANCE..=1. + TOLERANCE).contains(&u) {
        Some(t.clamp(0., 1.))
    } else {
        None
    }
}
pub fn cut(project: &Project, stroke: &[Point]) -> Result<Project> {
    project.validate()?;
    if !(2..=512).contains(&stroke.len()) || stroke.iter().flatten().any(|n| !n.is_finite()) {
        return Err("Trace um corte com 2 a 512 pontos válidos.".into());
    }
    let segments: Vec<_> = stroke
        .windows(2)
        .filter(|s| distance(s[0], s[1]) > TOLERANCE)
        .map(|s| (s[0], s[1], bounds(s)))
        .collect();
    if segments.is_empty() {
        return Err("O traço de corte precisa ter comprimento.".into());
    }
    let box_all = bounds(stroke);
    let mut work = 0usize;
    let mut result = Project {
        name: project.name.clone(),
        version: project.version,
        crs: project.crs.clone(),
        layers: project.layers.clone(),
        polygons: project.polygons.clone(),
        warnings: project.warnings.clone(),
        lines: Vec::with_capacity(project.lines.len()),
    };
    let mut visibility = std::collections::HashMap::new();
    let mut names: HashSet<String> = project.lines.iter().map(|l| l.name.clone()).collect();
    for line in &project.lines {
        if line.hidden
            || !*visibility
                .entry(line.layer)
                .or_insert_with(|| crate::layers::visible(project, line.layer))
        {
            result.lines.push(line.clone());
            continue;
        }
        let total = length(line);
        let mut cuts = vec![0., total];
        let mut cumulative = 0.;
        for pair in line.points.windows(2) {
            let len = distance(pair[0], pair[1]);
            let box_line = bounds(pair);
            if len > TOLERANCE && overlaps(box_line, box_all) {
                work += segments.len();
                if work > 30_000_000 {
                    return Err(
                        "Traço muito complexo para esta área. Faça cortes mais curtos.".into(),
                    );
                }
                for &(a, b, box_cut) in &segments {
                    if overlaps(box_line, box_cut)
                        && let Some(t) = intersection(pair[0], pair[1], a, b)
                    {
                        let along = cumulative + t * len;
                        if along > 0.01 && along < total - 0.01 {
                            cuts.push(along);
                        }
                    }
                }
            }
            cumulative += len;
        }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < 0.01);
        if cuts.len() == 2 {
            result.lines.push(line.clone());
            continue;
        }
        // Pieces below the minimum valid length are merged with a neighbour, never dropped.
        let mut filtered = vec![0.];
        for &cut in cuts.iter().skip(1).take(cuts.len() - 2) {
            if cut - filtered.last().unwrap() >= 0.01 && total - cut >= 0.01 {
                filtered.push(cut);
            }
        }
        filtered.push(total);
        for (index, range) in filtered.windows(2).enumerate() {
            let mut piece = trim_line(line, range[0], range[1])?;
            let mut number = index + 1;
            loop {
                let suffix = format!("_C{number}");
                let base = &line.name[..line.name.len().min(64 - suffix.len())];
                let candidate = format!("{base}{suffix}");
                if names.insert(candidate.clone()) {
                    piece.name = candidate;
                    break;
                }
                number += 1;
            }
            result.lines.push(piece);
        }
    }
    result.validate()?;
    Ok(result)
}
