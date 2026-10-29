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
            parent: None,
            hidden: false,
            id: 1,
            name: "flight.mis".into(),
            source: "MIS".into(),
            warnings: vec![],
        },
        Layer {
            parent: None,
            hidden: false,
            id: 2,
            name: "other.mis".into(),
            source: "MIS".into(),
            warnings: vec![],
        },
        Layer {
            parent: None,
            hidden: false,
            id: 3,
            name: "area.kmz".into(),
            source: "KMZ".into(),
            warnings: vec![],
        },
    ];
    p.lines = vec![
        Line {
            hidden: false,
            layer: Some(1),
            name: "L100".into(),
            points: vec![
                [500000., 7800000.],
                [500100., 7800000.],
                [500100., 7800100.],
            ],
        },
        Line {
            hidden: false,
            layer: Some(2),
            name: "L100".into(),
            points: vec![[500000., 7800200.], [500100., 7800200.]],
        },
    ];
    p.polygons = vec![Polygon {
        curve: None,
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

#[test]
fn folders_validate_cycles_and_export_only_descendants_without_losing_metadata() {
    let mut p = fixture();
    p.version = 5;
    p.layers.push(Layer {
        id: 10,
        name: "Survey".into(),
        source: "Group".into(),
        parent: None,
        hidden: false,
        warnings: vec![],
    });
    p.layers.push(Layer {
        id: 11,
        name: "Subset".into(),
        source: "Group".into(),
        parent: Some(10),
        hidden: false,
        warnings: vec![],
    });
    p.layers[0].parent = Some(11);
    p.lines[0].hidden = true;
    p.validate().unwrap();
    let saved = serde_json::to_string(&p).unwrap();
    let roundtrip: Project = serde_json::from_str(&saved).unwrap();
    assert!(roundtrip.lines[0].hidden);
    assert_eq!(roundtrip.layers[0].parent, Some(11));
    let subset = layers::subset(&p, 10).unwrap();
    assert_eq!(subset.lines.len(), 1);
    assert_eq!(subset.layers.len(), 3);
    assert_eq!(subset.version, 5);
    subset.validate().unwrap();
    assert!(export_mis(&subset).unwrap().contains("L100"));
    let mut bad = p.clone();
    bad.layers[3].parent = Some(11);
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.layers[0].parent = Some(2);
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.layers[0].parent = Some(999);
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.version = 4;
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.lines[0].layer = Some(10);
    assert!(bad.validate().is_err());
}
#[test]
fn hidden_lines_are_not_cut_and_group_visibility_is_inherited() {
    let mut p = fixture();
    p.version = 5;
    p.layers[0].hidden = true;
    assert!(!layers::visible(&p, Some(1)));
    let cut = editing::cut(&p, &[[500050., 7799900.], [500050., 7800300.]]).unwrap();
    assert_eq!(cut.lines.iter().filter(|l| l.layer == Some(1)).count(), 1);
    assert_eq!(cut.lines[0].points, p.lines[0].points);
    p.layers[0].hidden = false;
    p.lines[0].hidden = true;
    let cut = editing::cut(&p, &[[500050., 7799900.], [500050., 7800300.]]).unwrap();
    assert_eq!(cut.lines[0].points, p.lines[0].points);
}
