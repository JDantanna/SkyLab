use skylab_rust::{Crs, Line, Point, Project, curves::Anchor, distance, export_mis, line_paths};
fn anchor(point: Point) -> Anchor {
    Anchor {
        point,
        incoming: None,
        outgoing: None,
    }
}
fn project() -> Project {
    Project::empty(Crs {
        zone: 32,
        south: false,
        authority: None,
    })
}
#[test]
fn curves_roundtrip_flatten_and_export_without_losing_editability() {
    let p = project();
    let nodes = vec![
        Anchor {
            point: [500000., 6100000.],
            incoming: None,
            outgoing: Some([500030., 6100080.]),
        },
        Anchor {
            point: [500100., 6100000.],
            incoming: Some([500070., 6100080.]),
            outgoing: None,
        },
    ];
    let data = line_paths::upsert(&p, None, nodes.clone()).unwrap();
    assert_eq!(data.version, 7);
    assert!(data.lines[0].points.len() > 8);
    assert_eq!(data.lines[0].points.first(), Some(&nodes[0].point));
    assert_eq!(data.lines[0].points.last(), Some(&nodes[1].point));
    assert!(data.lines[0].points.iter().any(|p| p[1] > 6100050.));
    let saved: Project = serde_json::from_str(&serde_json::to_string(&data).unwrap()).unwrap();
    saved.validate().unwrap();
    assert_eq!(
        saved.lines[0].curve.as_ref().unwrap()[0].outgoing,
        nodes[0].outgoing
    );
    assert!(export_mis(&saved).unwrap().contains("DRAW_1"));
    assert!(p.lines.is_empty());
    let mut bad = saved;
    bad.lines[0].points[1][0] += 1.;
    assert!(bad.validate().is_err());
}
#[test]
fn insert_on_straight_and_curved_lines_keeps_the_shape() {
    let data = line_paths::upsert(
        &project(),
        None,
        vec![anchor([500000., 6100000.]), anchor([500100., 6100000.])],
    )
    .unwrap();
    let inserted = line_paths::insert(&data, 0, [500037., 6100000.]).unwrap();
    let nodes = inserted.lines[0].curve.as_ref().unwrap();
    assert_eq!(nodes.len(), 3);
    assert!(distance(nodes[1].point, [500037., 6100000.]) < 1e-5);
    let mut curved = nodes.clone();
    curved[0].outgoing = Some([500010., 6100030.]);
    curved[1].incoming = Some([500025., 6100030.]);
    let curved = line_paths::upsert(&inserted, Some(0), curved).unwrap();
    let inserted = line_paths::insert(&curved, 0, [500020., 6100020.]).unwrap();
    inserted.validate().unwrap();
    assert_eq!(inserted.lines[0].curve.as_ref().unwrap().len(), 4);
    assert_eq!(
        curved.lines[0].points.first(),
        inserted.lines[0].points.first()
    );
    assert_eq!(
        curved.lines[0].points.last(),
        inserted.lines[0].points.last()
    );
}
#[test]
fn join_chooses_nearest_endpoints_preserves_vertices_and_does_not_mutate_inputs() {
    let mut p = project();
    p.lines = vec![
        Line {
            name: "A".into(),
            layer: None,
            hidden: false,
            curve: None,
            points: vec![
                [500000., 6100000.],
                [500010., 6100000.],
                [500020., 6100000.],
            ],
        },
        Line {
            name: "B".into(),
            layer: None,
            hidden: false,
            curve: None,
            points: vec![
                [500050., 6100020.],
                [500040., 6100010.],
                [500030., 6100000.],
            ],
        },
    ];
    let joined = line_paths::join(&p, &[1, 0]).unwrap();
    assert_eq!(joined.lines.len(), 1);
    assert_eq!(joined.lines[0].name, "A");
    assert_eq!(
        joined.lines[0].points,
        vec![
            [500000., 6100000.],
            [500010., 6100000.],
            [500020., 6100000.],
            [500030., 6100000.],
            [500040., 6100010.],
            [500050., 6100020.]
        ]
    );
    assert_eq!(p.lines.len(), 2);
    assert!(!joined.warnings.is_empty());
    assert!(line_paths::join(&p, &[0, 0]).is_err());
    assert!(line_paths::join(&p, &[0, 999]).is_err());
}
#[test]
fn invalid_line_controls_are_rejected_before_any_geometry_changes() {
    assert!(line_paths::upsert(&project(), None, vec![anchor([500000., 6100000.])]).is_err());
    assert!(line_paths::flatten(&[anchor([f64::NAN, 0.]), anchor([1., 1.])]).is_err());
}
