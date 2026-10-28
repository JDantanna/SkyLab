use skylab_rust::{Crs, Line, Project, editing, length};
fn line(name: &str, points: &[[f64; 2]]) -> Line {
    Line {
        layer: None,
        name: name.into(),
        points: points
            .iter()
            .map(|p| [500000. + p[0], 7800000. + p[1]])
            .collect(),
    }
}
fn project(lines: Vec<Line>) -> Project {
    Project {
        layers: vec![],
        version: 1,
        crs: Crs {
            zone: 23,
            south: true,
            authority: None,
        },
        polygons: vec![],
        lines,
        warnings: vec![],
    }
}
fn stroke(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    points
        .iter()
        .map(|p| [500000. + p[0], 7800000. + p[1]])
        .collect()
}
#[test]
fn knife_preserves_every_original_vertex_and_total_length() {
    let p = project(vec![line(
        "Flight",
        &[[0., 0.], [100., 0.], [100., 100.], [200., 100.]],
    )]);
    let r = editing::cut(&p, &stroke(&[[50., -10.], [50., 110.]])).unwrap();
    assert_eq!(r.lines.len(), 2);
    assert!((r.lines.iter().map(length).sum::<f64>() - length(&p.lines[0])).abs() < 1e-7);
    for point in &p.lines[0].points {
        assert!(r.lines.iter().any(|l| l.points.contains(point)));
    }
    assert_eq!(r.lines[0].points.last(), r.lines[1].points.first());
    assert_eq!(p.lines.len(), 1);
}
#[test]
fn one_stroke_splits_several_passes_of_a_flown_turn() {
    let p = project(vec![line(
        "Flight",
        &[
            [0., 0.],
            [100., 0.],
            [100., 20.],
            [0., 20.],
            [0., 40.],
            [100., 40.],
        ],
    )]);
    let r = editing::cut(&p, &stroke(&[[50., -10.], [50., 50.]])).unwrap();
    assert_eq!(r.lines.len(), 4);
    for (piece, expected) in r.lines.iter().zip([50., 120., 120., 50.]) {
        assert!((length(piece) - expected).abs() < 1e-6);
    }
    assert_eq!(
        r.lines.first().unwrap().points.first(),
        p.lines[0].points.first()
    );
    assert_eq!(
        r.lines.last().unwrap().points.last(),
        p.lines[0].points.last()
    );
}
#[test]
fn retraced_knife_and_vertex_intersections_are_deduplicated() {
    let p = project(vec![line("Flight", &[[0., 0.], [50., 0.], [100., 0.]])]);
    let r = editing::cut(&p, &stroke(&[[50., -10.], [50., 10.], [50., -10.]])).unwrap();
    assert_eq!(r.lines.len(), 2);
    assert!(r.lines.iter().all(|l| l.points.len() == 2));
}
#[test]
fn endpoints_and_collinear_strokes_do_not_create_empty_pieces() {
    let p = project(vec![line("Flight", &[[0., 0.], [100., 0.]])]);
    for knife in [
        vec![[0., -10.], [0., 10.]],
        vec![[-10., 0.], [110., 0.]],
        vec![[200., 0.], [200., 10.]],
    ] {
        let r = editing::cut(&p, &stroke(&knife)).unwrap();
        assert_eq!(r.lines.len(), 1);
        assert_eq!(r.lines[0].name, "Flight");
        assert_eq!(r.lines[0].points, p.lines[0].points);
    }
}
#[test]
fn names_are_unique_and_fit_the_export_limit() {
    let long = "X".repeat(64);
    let p = project(vec![
        line(&long, &[[0., 0.], [100., 0.]]),
        line("Flight_C1", &[[0., 100.], [100., 100.]]),
        line("Flight", &[[0., 10.], [100., 10.]]),
    ]);
    let r = editing::cut(&p, &stroke(&[[50., -10.], [50., 20.]])).unwrap();
    r.validate().unwrap();
    assert_eq!(r.lines.len(), 5);
    assert!(r.lines.iter().all(|l| l.name.len() <= 64));
    assert_eq!(r.lines[2].name, "Flight_C1");
    assert_eq!(r.lines[3].name, "Flight_C2");
}
#[test]
fn invalid_strokes_are_rejected_without_mutating_the_project() {
    let p = project(vec![line("Flight", &[[0., 0.], [100., 0.]])]);
    for knife in [
        vec![],
        vec![[f64::NAN, 0.], [1., 1.]],
        vec![[0., 0.], [0., 0.]],
        vec![[0., 0.]; 513],
    ] {
        assert!(editing::cut(&p, &knife).is_err());
    }
    assert_eq!(p.lines[0].points.len(), 2);
}
