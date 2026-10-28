use skylab_rust::{
    curves::{self, Anchor},
    *,
};
fn empty() -> Project {
    Project::empty(Crs {
        zone: 23,
        south: true,
        authority: None,
    })
}
fn anchors() -> Vec<Anchor> {
    [
        [500000., 7800000.],
        [501000., 7800000.],
        [501000., 7801000.],
        [500000., 7801000.],
    ]
    .into_iter()
    .map(|point| Anchor {
        point,
        incoming: None,
        outgoing: None,
    })
    .collect()
}
fn bowed() -> Vec<Anchor> {
    let mut a = anchors();
    a[0].outgoing = Some([500500., 7799500.]);
    a[1].incoming = Some([501000., 7799500.]);
    a
}
fn distance_to_ring(point: Point, ring: &[Point]) -> f64 {
    ring.windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            let t = (((point[0] - w[0][0]) * dx + (point[1] - w[0][1]) * dy) / (dx * dx + dy * dy))
                .clamp(0., 1.);
            distance(point, [w[0][0] + dx * t, w[0][1] + dy * t])
        })
        .fold(f64::INFINITY, f64::min)
}
#[test]
fn editable_polygon_roundtrips_control_points_and_generates_mission_lines() {
    let p = curves::upsert(&empty(), None, bowed()).unwrap();
    assert_eq!(p.version, 4);
    assert_eq!(p.layers[0].source, "Drawing");
    assert_eq!(p.polygons[0].layer, Some(p.layers[0].id));
    assert!(p.polygons[0].outer.len() > 5);
    let saved = serde_json::to_string(&p).unwrap();
    let restored: Project = serde_json::from_str(&saved).unwrap();
    restored.validate().unwrap();
    assert_eq!(
        restored.polygons[0].curve.as_ref().unwrap()[0].outgoing,
        Some([500500., 7799500.])
    );
    let lines = generate(
        &restored,
        &Settings {
            spacing: 100.,
            angle: 0.,
            offset: 50.,
            min_length: 1.,
            prefix: "L".into(),
            first: 1,
            increment: 1,
            alternate: false,
        },
    )
    .unwrap();
    assert!(!lines.is_empty());
}
#[test]
fn inserting_an_anchor_on_a_straight_cubic_places_it_at_the_click_not_at_linear_parameter_time() {
    let p = curves::upsert(&empty(), None, anchors()).unwrap();
    let result = curves::insert(&p, 0, [500250., 7800000.]).unwrap();
    let anchors = result.polygons[0].curve.as_ref().unwrap();
    assert_eq!(anchors.len(), 5);
    assert!(distance(anchors[1].point, [500250., 7800000.]) < 0.001);
    assert_eq!(p.polygons[0].curve.as_ref().unwrap().len(), 4);
    assert!(
        result.polygons[0]
            .outer
            .iter()
            .all(|p| distance_to_ring(*p, &curves::flatten(&crate::anchors()).unwrap()) < 0.001)
    );
}
#[test]
fn curve_anchor_insertion_preserves_shape_and_keeps_both_tangent_handles() {
    let p = curves::upsert(&empty(), None, bowed()).unwrap();
    let result = curves::insert(&p, 0, [500687.5, 7799625.]).unwrap();
    let a = result.polygons[0].curve.as_ref().unwrap();
    assert!(distance(a[1].point, [500687.5, 7799625.]) < 0.001);
    assert!(a[1].incoming.is_some() && a[1].outgoing.is_some());
    for point in &result.polygons[0].outer {
        assert!(distance_to_ring(*point, &p.polygons[0].outer) <= 0.2);
    }
    for point in &p.polygons[0].outer {
        assert!(distance_to_ring(*point, &result.polygons[0].outer) <= 0.2);
    }
}
#[test]
fn invalid_paths_and_stale_curve_vertices_are_refused_without_changing_the_project() {
    let base = empty();
    let mut crossed = anchors();
    crossed.swap(1, 2);
    assert!(curves::upsert(&base, None, crossed).is_err());
    assert!(base.polygons.is_empty());
    let mut invalid = anchors();
    invalid[0].outgoing = Some([f64::NAN, 0.]);
    assert!(curves::flatten(&invalid).is_err());
    let mut p = curves::upsert(&base, None, bowed()).unwrap();
    p.polygons[0].outer[1][0] += 1.;
    assert!(p.validate().is_err());
    let mut p = curves::upsert(&base, None, anchors()).unwrap();
    p.version = 3;
    assert!(p.validate().is_err());
    assert!(curves::insert(&base, 0, [500000., 7800000.]).is_err());
}

#[test]
fn curve_layer_exports_preserve_editability_in_json_and_include_its_shape_in_kml() {
    let mut p = curves::upsert(&empty(), None, bowed()).unwrap();
    p.polygons[0].holes.push(vec![
        [500200., 7800200.],
        [500400., 7800200.],
        [500400., 7800400.],
        [500200., 7800400.],
        [500200., 7800200.],
    ]);
    let subset = layers::subset(&p, p.layers[0].id).unwrap();
    assert_eq!(subset.version, 4);
    subset.validate().unwrap();
    assert!(subset.polygons[0].curve.is_some());
    let text = export_kml(&subset).unwrap();
    assert!(text.contains("<Polygon>"));
    let imported = import_kml(&text, p.crs.clone()).unwrap();
    assert_eq!(imported.polygons.len(), 1);
    assert_eq!(imported.polygons[0].holes.len(), 1);
    assert_eq!(imported.polygons[0].outer.len(), p.polygons[0].outer.len());
    for point in &imported.polygons[0].outer {
        assert!(distance_to_ring(*point, &p.polygons[0].outer) < 0.01);
    }
}
