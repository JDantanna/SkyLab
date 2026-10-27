use skylab_rust::{Crs, projection};
#[test]
fn catalog_includes_qgis_common_systems_and_guards_nonmetric_workspaces() {
    assert!(projection::catalog().len() > 13000);
    for id in ["EPSG:31983", "EPSG:25832", "EPSG:32723", "EPSG:26915"] {
        projection::validate(id).unwrap();
    }
    assert!(projection::validate("EPSG:4326").is_err());
    assert!(projection::validate("EPSG:2227").is_err());
    assert!(projection::validate("EPSG:999999").is_err());
}
#[cfg(windows)]
#[test]
fn offline_proj_transforms_multiple_datums_and_preserves_project_authority() {
    for (id, point) in [
        ("EPSG:31983", [-45., -20.]),
        ("EPSG:25832", [9., 56.]),
        ("EPSG:26915", [-93., 45.]),
    ] {
        let crs = Crs {
            zone: 23,
            south: true,
            authority: Some(id.into()),
        };
        let xy = crs.forward(point).unwrap();
        assert!((xy[0] - 500000.).abs() < 100.);
        let ll = crs.inverse(xy).unwrap();
        assert!((ll[0] - point[0]).abs() < 1e-7);
        assert!((ll[1] - point[1]).abs() < 1e-7);
        let restored: Crs = serde_json::from_str(&serde_json::to_string(&crs).unwrap()).unwrap();
        assert_eq!(restored.authority.as_deref(), Some(id));
    }
}
#[cfg(windows)]
#[test]
fn missing_datum_grid_fails_instead_of_using_an_approximation() {
    assert!(projection::transform("EPSG:4326", "EPSG:26706", [-147., 64.]).is_err());
}
#[test]
fn authority_requires_new_project_schema_to_protect_older_readers() {
    let mut p = skylab_rust::Project::empty(Crs {
        zone: 23,
        south: true,
        authority: Some("EPSG:31983".into()),
    });
    assert_eq!(p.version, 2);
    p.validate().unwrap();
    p.version = 1;
    assert!(p.validate().is_err());
}
