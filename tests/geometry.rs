use skylab_rust::*;
fn crs() -> Crs {
    Crs {
        authority: None,
        zone: 23,
        south: true,
    }
}
fn pt(x: f64, y: f64) -> Point {
    [500000. + x, 7800000. + y]
}
fn ring(coords: &[(f64, f64)]) -> Vec<Point> {
    coords.iter().map(|&(x, y)| pt(x, y)).collect()
}
fn square() -> Polygon {
    Polygon {
        name: None,
        hidden: false,
        curve: None,
        layer: None,
        outer: ring(&[(0., 0.), (1000., 0.), (1000., 1000.), (0., 1000.), (0., 0.)]),
        holes: vec![],
    }
}
fn project(g: Polygon) -> Project {
    let mut p = Project::empty(crs());
    p.polygons.push(g);
    p
}
fn line(points: Vec<Point>) -> Line {
    Line {
        hidden: false,
        layer: None,
        name: "L100100".into(),
        points,
    }
}
fn settings() -> Settings {
    Settings {
        spacing: 200.,
        angle: 0.,
        offset: 100.,
        min_length: 0.,
        prefix: "L".into(),
        first: 100100,
        increment: 100,
        alternate: false,
    }
}
#[test]
fn projection_matches_published_proj_vectors() {
    // https://proj.org/en/stable/operations/projections/utm.html
    for (c, ll, xy) in [
        (
            Crs {
                authority: None,
                zone: 32,
                south: false,
            },
            [12., 56.],
            [687071.44, 6210141.33],
        ),
        (
            Crs {
                authority: None,
                zone: 59,
                south: true,
            },
            [174., -44.],
            [740526.32, 5123750.87],
        ),
    ] {
        let got = c.forward(ll).unwrap();
        assert!(distance(got, xy) < 0.02, "{got:?}");
        let back = c.inverse(got).unwrap();
        assert!(distance(back, ll) < 0.0000001, "{back:?}");
    }
}
#[test]
fn projection_grid_roundtrip() {
    for zone in [1, 23, 32, 60] {
        for lat in [-79., -20., 0.1, 55., 83.] {
            let c = Crs {
                authority: None,
                zone,
                south: lat < 0.,
            };
            for d in [-2.99, 0., 2.99] {
                let ll = [zone as f64 * 6. - 183. + d, lat];
                assert!(distance(c.inverse(c.forward(ll).unwrap()).unwrap(), ll) < 1e-6)
            }
        }
    }
}
#[test]
fn rejects_wrong_crs_nonfinite_and_out_of_domain() {
    assert!(crs().forward([-43., 20.]).is_err());
    assert!(crs().forward([0., -20.]).is_err());
    assert!(crs().forward([f64::NAN, -20.]).is_err());
    assert!(
        Crs {
            authority: None,
            zone: 0,
            south: true
        }
        .validate()
        .is_err()
    );
    assert!(crs().inverse([1., 2.]).is_err())
}
#[test]
fn north_south_spacing_and_numbering() {
    let lines = generate(&project(square()), &settings()).unwrap();
    assert_eq!(lines.len(), 5);
    for (i, l) in lines.iter().enumerate() {
        assert_eq!(l.name, format!("L{}", 100100 + i * 100));
        assert!((length(l) - 1000.).abs() < 1e-5);
        assert!((l.points[0][0] - 500100. - i as f64 * 200.).abs() < 1e-5);
        assert!(l.points[1][1] > l.points[0][1]);
    }
}
#[test]
fn east_west_and_reciprocal_bearing() {
    for angle in [90., 120., 270., 300.] {
        let mut s = settings();
        s.angle = angle;
        let lines = generate(&project(square()), &s).unwrap();
        for l in lines {
            let a = l.points.first().unwrap();
            let b = l.points.last().unwrap();
            let bearing = (b[0] - a[0])
                .atan2(b[1] - a[1])
                .to_degrees()
                .rem_euclid(360.);
            assert!((bearing - angle).abs() < 1e-6, "{bearing} {angle}");
        }
    }
}
#[test]
fn alternate_direction() {
    let mut s = settings();
    s.alternate = true;
    let lines = generate(&project(square()), &s).unwrap();
    assert!(lines[0].points[1][1] > lines[0].points[0][1]);
    assert!(lines[1].points[1][1] < lines[1].points[0][1]);
}
#[test]
fn hole_splits_without_connecting_exclusion() {
    let mut g = square();
    g.holes.push(ring(&[
        (400., 400.),
        (600., 400.),
        (600., 600.),
        (400., 600.),
        (400., 400.),
    ]));
    let pieces = clip_line(&line(vec![pt(500., -100.), pt(500., 1100.)]), &[g]);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0], vec![pt(500., 0.), pt(500., 400.)]);
    assert_eq!(pieces[1], vec![pt(500., 600.), pt(500., 1000.)]);
}
#[test]
fn concave_polygon_splits_into_two() {
    let g = Polygon {
        name: None,
        hidden: false,
        curve: None,
        layer: None,
        outer: ring(&[
            (0., 0.),
            (1000., 0.),
            (1000., 1000.),
            (700., 1000.),
            (700., 300.),
            (300., 300.),
            (300., 1000.),
            (0., 1000.),
            (0., 0.),
        ]),
        holes: vec![],
    };
    let pieces = clip_line(&line(vec![pt(-100., 500.), pt(1100., 500.)]), &[g]);
    assert_eq!(pieces.len(), 2);
    assert!((length(&line(pieces[0].clone())) - 300.).abs() < 1e-7);
}
#[test]
fn overlapping_blocks_are_union() {
    let a = square();
    let mut b = square();
    for p in &mut b.outer {
        p[0] += 500.
    }
    let pieces = clip_line(&line(vec![pt(-100., 500.), pt(2000., 500.)]), &[a, b]);
    assert_eq!(pieces.len(), 1);
    assert!((length(&line(pieces[0].clone())) - 1500.).abs() < 1e-6);
}
#[test]
fn multi_vertex_line_preserved_and_trimmed() {
    let l = line(vec![
        pt(0., 0.),
        pt(100., 0.),
        pt(100., 100.),
        pt(200., 100.),
    ]);
    let t = trim_line(&l, 50., 250.).unwrap();
    assert_eq!(
        t.points,
        vec![pt(50., 0.), pt(100., 0.), pt(100., 100.), pt(150., 100.)]
    );
    assert_eq!(length(&t), 200.);
    assert!(trim_line(&l, 250., 50.).is_err());
    assert!(trim_line(&l, 0., 301.).is_err());
}
#[test]
fn clipping_keeps_inside_vertices() {
    let l = line(vec![pt(100., 100.), pt(500., 300.), pt(900., 900.)]);
    assert_eq!(clip_line(&l, &[square()]), vec![l.points]);
}
#[test]
fn boundary_and_vertex_tangencies() {
    let g = square();
    let on = clip_line(
        &line(vec![pt(-100., 0.), pt(1100., 0.)]),
        std::slice::from_ref(&g),
    );
    assert_eq!(on.len(), 1);
    assert!((length(&line(on[0].clone())) - 1000.).abs() < EPS_TEST);
    let tangent = clip_line(&line(vec![pt(-100., 100.), pt(100., -100.)]), &[g]);
    assert!(tangent.is_empty());
}
const EPS_TEST: f64 = 1e-6;
#[test]
fn invalid_polygon_and_holes_rejected() {
    let mut g = square();
    g.outer.swap(1, 2);
    assert!(project(g).validate().is_err());
    let mut g = square();
    g.holes.push(ring(&[
        (900., 900.),
        (1100., 900.),
        (1100., 1100.),
        (900., 1100.),
        (900., 900.),
    ]));
    assert!(project(g).validate().is_err());
    let mut g = square();
    g.outer.pop();
    assert!(project(g).validate().is_err());
}
#[test]
fn invalid_settings_rejected() {
    let p = project(square());
    let mut s = settings();
    s.spacing = 0.;
    assert!(generate(&p, &s).is_err());
    s = settings();
    s.offset = 200.;
    assert!(generate(&p, &s).is_err());
    s = settings();
    s.increment = 0;
    assert!(generate(&p, &s).is_err());
    s = settings();
    s.first = u32::MAX;
    assert!(generate(&p, &s).is_err());
    s = settings();
    s.min_length = 1001.;
    assert!(generate(&p, &s).is_err());
}
#[test]
fn mis_roundtrip_and_reject_truncation() {
    let mut p = project(square());
    p.lines
        .push(line(vec![pt(0., 0.), pt(100., 0.), pt(100., 100.)]));
    let text = export_mis(&p).unwrap();
    assert!(text.contains("\r\n"));
    let q = import_mis(&text, crs()).unwrap();
    assert_eq!(p.lines[0].points, q.lines[0].points);
    assert!(import_mis("L1\n500000 7800000", crs()).is_err());
    assert!(import_mis("L1\n500000 NaN\n500001 7800000", crs()).is_err());
    assert!(import_mis("500000 7800000", crs()).is_err());
}
#[test]
fn duplicate_and_unsafe_names_rejected() {
    for name in ["L1\nINJECT", "a b", "área", "123", "NaN", ""] {
        let mut p = Project::empty(crs());
        p.lines.push(Line {
            hidden: false,
            layer: None,
            name: name.into(),
            points: vec![pt(0., 0.), pt(10., 10.)],
        });
        assert!(p.validate().is_err());
    }
    assert!(
        import_mis(
            "L1\n500000 7800000\n500010 7800010\nL1\n500000 7800000\n500010 7800010",
            crs()
        )
        .is_err()
    );
}
#[test]
fn kml_export_reimport_multi_vertex() {
    let mut p = Project::empty(crs());
    p.lines
        .push(line(vec![pt(0., 0.), pt(100., 0.), pt(100., 100.)]));
    let text = export_kml(&p).unwrap();
    let q = import_kml(&text, crs()).unwrap();
    assert_eq!(q.lines[0].points.len(), 3);
    for (a, b) in p.lines[0].points.iter().zip(&q.lines[0].points) {
        assert!(distance(*a, *b) < 0.001)
    }
}
#[test]
fn kml_polygon_hole_namespace_and_lines_only() {
    let k = "<k:kml xmlns:k='http://www.opengis.net/kml/2.2'><k:Placemark><k:Polygon><k:outerBoundaryIs><k:LinearRing><k:coordinates>-45,-20 -44.99,-20 -44.99,-19.99 -45,-19.99 -45,-20</k:coordinates></k:LinearRing></k:outerBoundaryIs><k:innerBoundaryIs><k:LinearRing><k:coordinates>-44.998,-19.998 -44.996,-19.998 -44.996,-19.996 -44.998,-19.996 -44.998,-19.998</k:coordinates></k:LinearRing></k:innerBoundaryIs></k:Polygon></k:Placemark></k:kml>";
    let p = import_kml(k, crs()).unwrap();
    assert_eq!(p.polygons[0].holes.len(), 1);
    let mut p = Project::empty(crs());
    p.lines.push(line(vec![pt(0., 0.), pt(10., 10.)]));
    let q = import_kml(&export_kml(&p).unwrap(), crs()).unwrap();
    assert!(q.polygons.is_empty());
    assert!(generate(&q, &settings()).is_err());
}
#[test]
fn malformed_xml_and_entities_rejected() {
    assert!(import_kml("<kml><Placemark>", crs()).is_err());
    assert!(
        import_kml(
            "<!DOCTYPE kml [<!ENTITY x SYSTEM 'file:///secret'>]><kml>&x;</kml>",
            crs()
        )
        .is_err()
    );
}
#[test]
fn json_persists_projection_and_holes() {
    let p = project(square());
    let data = serde_json::to_string(&p).unwrap();
    let q: Project = serde_json::from_str(&data).unwrap();
    q.validate().unwrap();
    assert_eq!(q.crs.zone, 23);
    assert!(q.crs.south);
    assert_eq!(q.polygons.len(), 1);
}

#[test]
fn kml_attribute_ids_and_duplicate_names_are_audited() {
    let k = "<kml><Document><Placemark><ExtendedData><SchemaData><SimpleData name='ID'>306950</SimpleData></SchemaData></ExtendedData><LineString><coordinates>-45,-20 -44.999,-20</coordinates></LineString></Placemark><Placemark><name>L306950</name><LineString><coordinates>-45,-20 -44.999,-20</coordinates></LineString></Placemark></Document></kml>";
    let p = import_kml(k, crs()).unwrap();
    assert_eq!(p.lines[0].name, "L306950");
    assert_eq!(p.lines[1].name, "L306950_2");
    assert!(p.warnings.iter().any(|s| s.contains("repetido")));
}

#[test]
fn project_json_roundtrips_full_float_precision() {
    let mut p = project(square());
    p.lines.push(line(vec![
        [604286.6339126186, 7778574.7499720985],
        [606016.564830988, 7759571.0782162985],
    ]));
    let q: Project = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(p.lines[0].points, q.lines[0].points);
}
#[test]
fn mis_rounding_must_not_collapse_line() {
    let mut p = Project::empty(crs());
    p.lines
        .push(line(vec![pt(-0.0049, -0.0049), pt(0.0049, 0.0049)]));
    p.validate().unwrap();
    assert!(export_mis(&p).is_err());
}
