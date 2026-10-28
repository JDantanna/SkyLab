use skylab_rust::{
    Crs, Line, Polygon, Project, editing, export_mis,
    layers::{self, Layer},
    length, trim_line,
};
fn fixture() -> Project {
    let mut p = Project::empty(Crs {
        zone: 23,
        south: true,
        authority: None,
    });
    p.version = 3;
    p.layers = vec![
        Layer {
            id: 1,
            name: "flight.mis".into(),
            source: "MIS".into(),
            warnings: vec![],
        },
        Layer {
            id: 2,
            name: "other.mis".into(),
            source: "MIS".into(),
            warnings: vec![],
        },
        Layer {
            id: 3,
            name: "area.kmz".into(),
            source: "KMZ".into(),
            warnings: vec![],
        },
    ];
    p.lines = vec![
        Line {
            layer: Some(1),
            name: "L100".into(),
            points: vec![
                [500000., 7800000.],
                [500100., 7800000.],
                [500100., 7800100.],
            ],
        },
        Line {
            layer: Some(2),
            name: "L100".into(),
            points: vec![[500000., 7800200.], [500100., 7800200.]],
        },
    ];
    p.polygons = vec![Polygon {
        layer: Some(3),
        outer: vec![
            [499900., 7799900.],
            [500200., 7799900.],
            [500200., 7800300.],
            [499900., 7800300.],
            [499900., 7799900.],
        ],
        holes: vec![],
    }];
    p
}
#[test]
fn exports_only_the_chosen_layer_even_when_other_files_have_the_same_line_name() {
    let p = fixture();
    p.validate().unwrap();
    let subset = layers::subset(&p, 1).unwrap();
    assert_eq!(subset.lines.len(), 1);
    assert!(subset.polygons.is_empty());
    assert_eq!(subset.layers.len(), 1);
    assert_eq!(subset.lines[0].points, p.lines[0].points);
    assert!(export_mis(&subset).is_ok());
    assert!(export_mis(&p).unwrap_err().contains("camadas"));
    let polygon = layers::subset(&p, 3).unwrap();
    assert_eq!(polygon.polygons.len(), 1);
    assert!(polygon.lines.is_empty());
    assert!(layers::subset(&p, 99).is_err());
}
#[test]
fn trim_knife_and_json_roundtrip_preserve_file_provenance_and_vertices() {
    let p = fixture();
    let trimmed = trim_line(&p.lines[0], 20., 180.).unwrap();
    assert_eq!(trimmed.layer, Some(1));
    assert!(trimmed.points.contains(&[500100., 7800000.]));
    let cut = editing::cut(&p, &[[500050., 7799990.], [500050., 7800010.]]).unwrap();
    assert_eq!(cut.lines.len(), 3);
    assert_eq!(cut.lines[0].layer, Some(1));
    assert_eq!(cut.lines[1].layer, Some(1));
    assert_eq!(cut.lines[2].layer, Some(2));
    assert!((cut.lines.iter().map(length).sum::<f64>() - 300.).abs() < 1e-7);
    let restored: Project = serde_json::from_str(&serde_json::to_string(&cut).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.layers[2].source, "KMZ");
    assert_eq!(restored.lines[1].points, cut.lines[1].points);
}
#[test]
fn rejects_dangling_layer_ids_and_duplicate_layer_definitions_and_reads_legacy_projects() {
    let mut p = fixture();
    p.lines[0].layer = Some(99);
    assert!(p.validate().is_err());
    p = fixture();
    p.layers[1].id = 1;
    assert!(p.validate().is_err());
    p = fixture();
    p.version = 2;
    assert!(p.validate().is_err());
    let legacy:Project=serde_json::from_str(r#"{"version":1,"crs":{"zone":23,"south":true},"polygons":[],"lines":[{"name":"L1","points":[[500000,7800000],[500100,7800000]]}],"warnings":[]}"#).unwrap();
    legacy.validate().unwrap();
    assert!(legacy.layers.is_empty());
    assert_eq!(legacy.lines[0].layer, None);
}
