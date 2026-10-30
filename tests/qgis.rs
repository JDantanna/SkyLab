#![cfg(windows)]
use skylab_rust::{Crs, Project, files::Files, qgis};
use std::{fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> Fixture {
    let root = std::env::temp_dir().join(format!(
        "skylab-qgis-synthetic-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    Fixture(root)
}
fn geojson() -> serde_json::Value {
    serde_json::json!({"type":"FeatureCollection","features":[{"type":"Feature","properties":{"private":"not imported"},"geometry":{"type":"Polygon","coordinates":[[[9.,55.],[9.02,55.],[9.02,55.02],[9.,55.02],[9.,55.]],[[9.005,55.005],[9.005,55.01],[9.01,55.01],[9.01,55.005],[9.005,55.005]]]}}]})
}
#[test]
fn qgs_preserves_groups_visibility_holes_crs_and_reports_missing_sources() {
    let f = fixture();
    fs::write(f.0.join("area.geojson"), geojson().to_string()).unwrap();
    let xml = r#"<!DOCTYPE qgis PUBLIC 'http://mrcc.com/qgis.dtd' 'SYSTEM'><qgis><title>Synthetic survey</title><projectCrs><spatialrefsys><authid>EPSG:32632</authid></spatialrefsys></projectCrs><layer-tree-group><layer-tree-group name="Survey" checked="Qt::Checked"><layer-tree-layer id="area" checked="Qt::Unchecked"/><layer-tree-layer id="missing" checked="Qt::Checked"/></layer-tree-group></layer-tree-group><projectlayers><maplayer><id>area</id><layername>Boundary</layername><provider>ogr</provider><datasource>./area.geojson</datasource><srs><spatialrefsys><authid>EPSG:4326</authid></spatialrefsys></srs></maplayer><maplayer><id>missing</id><layername>Missing</layername><provider>ogr</provider><datasource>./missing.shp</datasource></maplayer></projectlayers></qgis>"#;
    let path = f.0.join("survey.qgs");
    fs::write(&path, xml).unwrap();
    let before = fs::read(&path).unwrap();
    let bundle = qgis::read(&path).unwrap();
    assert_eq!(bundle.sources.len(), 3);
    assert_eq!(bundle.sources[1].layer.parent, Some(1));
    assert!(bundle.sources[1].layer.hidden);
    assert_eq!(bundle.warnings.len(), 1);
    let info = qgis::inspect(&bundle).unwrap();
    assert_eq!(info["crs"]["authority"], "EPSG:32632");
    let p = qgis::import(
        &bundle,
        serde_json::from_value(info["crs"].clone()).unwrap(),
    )
    .unwrap();
    p.validate().unwrap();
    assert_eq!(p.polygons.len(), 1);
    assert_eq!(p.polygons[0].holes.len(), 1);
    assert_eq!(p.name.as_deref(), Some("Synthetic survey"));
    assert_eq!(p.layers[0].source, "Group");
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!serde_json::to_string(&p).unwrap().contains("private"));
}
#[test]
fn remote_and_entity_sources_are_never_executed() {
    let f = fixture();
    let path = f.0.join("remote.qgs");
    fs::write(&path,r#"<qgis><projectlayers><maplayer><id>x</id><layername>Remote</layername><provider>ogr</provider><datasource>https://example.com/a.geojson</datasource></maplayer></projectlayers></qgis>"#).unwrap();
    assert!(
        qgis::read(&path)
            .unwrap_err()
            .contains("not loaded automatically")
    );
    fs::write(
        path.clone(),
        r#"<!DOCTYPE qgis [<!ENTITY example "a">]><qgis/>"#,
    )
    .unwrap();
    assert!(
        qgis::read(&path)
            .unwrap_err()
            .contains("Entity declarations")
    );
}
#[test]
fn standalone_geojson_and_file_queue_acceptance_keep_projection_explicit() {
    let f = fixture();
    let path = f.0.join("area.geojson");
    fs::write(&path, geojson().to_string()).unwrap();
    let bundle = qgis::read(&path).unwrap();
    assert!(bundle.crs.is_none());
    let info = qgis::inspect(&bundle).unwrap();
    assert_eq!(info["kind"], "suggested");
    let crs: Crs = serde_json::from_value(info["crs"].clone()).unwrap();
    assert_eq!(crs.zone, 32);
    assert!(!crs.south);
    let p = qgis::import(&bundle, crs).unwrap();
    assert_eq!(p.polygons.len(), 1);
    let mut files = Files::at(f.0.join("recents"));
    files.enqueue(&[path], true).unwrap();
    let pending = files.pending().unwrap().id;
    let file = files.read(pending).unwrap();
    assert_eq!(file["qgis"], true);
    assert!(serde_json::from_str::<qgis::Bundle>(file["text"].as_str().unwrap()).is_ok());
}
#[test]
fn read_only_geopackage_reads_named_table_and_qgz_opens_referenced_vectors() {
    let f = fixture();
    let gpkg = f.0.join("synthetic data.gpkg");
    let b64 = f.0.join("fixture.b64");
    fs::write(&b64, include_str!("synthetic-gpkg.b64")).unwrap();
    let script = format!(
        "[IO.File]::WriteAllBytes('{}',[Convert]::FromBase64String([IO.File]::ReadAllText('{}')))",
        gpkg.to_string_lossy().replace('\'', "''"),
        b64.to_string_lossy().replace('\'', "''")
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(script)
        .status()
        .unwrap();
    assert!(status.success());
    let before = fs::read(&gpkg).unwrap();
    assert!(
        qgis::read(&gpkg)
            .unwrap_err()
            .contains("multiple vector layers")
    );
    let qgs = f.0.join("mission.qgs");
    fs::write(&qgs,r#"<qgis><projectCrs><spatialrefsys><authid>EPSG:32632</authid></spatialrefsys></projectCrs><projectlayers><maplayer><id>l</id><layername>Flight</layername><provider>ogr</provider><datasource>./synthetic data.gpkg|layername=flight</datasource></maplayer><maplayer><id>p</id><layername>Blocks</layername><provider>ogr</provider><datasource>./synthetic data.gpkg|layername=blocks</datasource></maplayer></projectlayers></qgis>"#).unwrap();
    let bundle = qgis::read(&qgs).unwrap();
    let info = qgis::inspect(&bundle).unwrap();
    let p: Project = qgis::import(
        &bundle,
        serde_json::from_value(info["crs"].clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(p.lines.len(), 1);
    assert_eq!(p.polygons.len(), 1);
    assert_eq!(
        p.lines[0].points,
        vec![
            [500000., 6100000.],
            [500100., 6100100.],
            [500200., 6100100.]
        ]
    );
    assert_eq!(fs::read(&gpkg).unwrap(), before);
    let zipped = f.0.join("compressed");
    fs::create_dir(&zipped).unwrap();
    fs::copy(&qgs, zipped.join("mission.qgs")).unwrap();
    let qgz = f.0.join("mission.qgz");
    let script = format!(
        "Add-Type -AssemblyName System.IO.Compression.FileSystem;[IO.Compression.ZipFile]::CreateFromDirectory('{}','{}')",
        zipped.to_string_lossy().replace('\'', "''"),
        qgz.to_string_lossy().replace('\'', "''")
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(script)
        .status()
        .unwrap();
    assert!(status.success());
    let compressed = qgis::read(&qgz).unwrap();
    assert_eq!(compressed.sources.len(), 2);
    assert_eq!(compressed.sources[0].shapes[0].rings[0], p.lines[0].points);
}
#[test]
fn shapefile_parts_are_preserved_and_truncation_is_rejected() {
    let f = fixture();
    let path = f.0.join("route.shp");
    let mut record = vec![];
    record.extend(3i32.to_le_bytes());
    for n in [500000f64, 6100000., 500200., 6100100.] {
        record.extend(n.to_le_bytes());
    }
    record.extend(1i32.to_le_bytes());
    record.extend(3i32.to_le_bytes());
    record.extend(0i32.to_le_bytes());
    for p in [
        [500000f64, 6100000.],
        [500100., 6100100.],
        [500200., 6100100.],
    ] {
        for n in p {
            record.extend(n.to_le_bytes());
        }
    }
    let mut bytes = vec![0; 100];
    bytes[..4].copy_from_slice(&9994i32.to_be_bytes());
    bytes[28..32].copy_from_slice(&1000i32.to_le_bytes());
    bytes[32..36].copy_from_slice(&3i32.to_le_bytes());
    bytes.extend(1i32.to_be_bytes());
    bytes.extend((record.len() as i32 / 2).to_be_bytes());
    bytes.extend(record);
    fs::write(&path, &bytes).unwrap();
    fs::write(path.with_extension("prj"), "EPSG:32632").unwrap();
    let bundle = qgis::read(&path).unwrap();
    assert_eq!(bundle.sources[0].shapes[0].rings[0].len(), 3);
    bytes.pop();
    fs::write(&path, bytes).unwrap();
    assert!(qgis::read(&path).is_err());
}
