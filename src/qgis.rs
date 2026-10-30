//! QGIS project/vector import. Read-only local sources; no plugins, scripts or remote providers run.
use crate::{Crs, Line, Point, Polygon, Project, Result, layers::Layer, projection};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub kind: String,
    pub rings: Vec<Vec<Point>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub layer: Layer,
    pub crs: String,
    pub shapes: Vec<Shape>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub title: String,
    pub crs: Option<String>,
    pub sources: Vec<Source>,
    pub warnings: Vec<String>,
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn text(node: roxmltree::Node<'_, '_>, tag: &str) -> String {
    node.descendants()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .unwrap_or("")
        .trim()
        .to_owned()
}
fn child_text(node: roxmltree::Node<'_, '_>, tag: &str) -> String {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .unwrap_or("")
        .trim()
        .to_owned()
}
fn small_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(error)?;
    if !file.metadata().map_err(error)?.is_file() {
        return Err("Not a regular file".into());
    }
    let mut data = vec![];
    file.take(limit + 1).read_to_end(&mut data).map_err(error)?;
    if data.len() as u64 > limit {
        return Err("Vector file exceeds the import size limit".into());
    }
    Ok(data)
}
fn small_text(path: &Path) -> Result<String> {
    String::from_utf8(small_read(path, 32 * 1024 * 1024)?).map_err(error)
}
fn source_crs(node: roxmltree::Node<'_, '_>) -> String {
    let id = text(node, "authid");
    if id.is_empty() { text(node, "wkt") } else { id }
}
fn qgis_xml(path: &Path) -> Result<String> {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("qgz"))
    {
        crate::kmz::read_qgz(&crate::files::base64(&small_read(path, 8 * 1024 * 1024)?))
    } else {
        small_text(path)
    }
}
pub fn read(path: &Path) -> Result<Bundle> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !["qgs", "qgz", "qlr"].contains(&ext.as_str()) {
        let (crs, shapes) = read_vector(path, None, None)?;
        let bundle = Bundle {
            title: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            crs: if projection::validate(&crs).is_ok() {
                Some(crs.clone())
            } else {
                None
            },
            sources: vec![Source {
                layer: Layer {
                    id: 1,
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    source: "QGIS".into(),
                    parent: None,
                    hidden: false,
                    warnings: vec![],
                },
                crs,
                shapes,
            }],
            warnings: vec![],
        };
        check_bundle(&bundle)?;
        return Ok(bundle);
    }
    let xml = qgis_xml(path)?;
    if xml.contains("<!ENTITY") {
        return Err("Entity declarations are not supported in QGIS projects".into());
    }
    let doc = roxmltree::Document::parse_with_options(
        &xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .map_err(error)?;
    let root = doc.root_element();
    if !["qgis", "qlr"].contains(&root.tag_name().name()) {
        return Err("Not a QGIS project or layer definition".into());
    }
    let crs = root
        .children()
        .find(|n| n.has_tag_name("projectCrs"))
        .map(source_crs)
        .filter(|s| !s.is_empty());
    let mut bundle = Bundle {
        title: child_text(root, "title"),
        crs,
        sources: vec![],
        warnings: vec![],
    };
    if bundle.title.is_empty() {
        bundle.title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
    }
    let map: HashMap<_, _> = root
        .descendants()
        .filter(|n| n.has_tag_name("maplayer"))
        .map(|n| (child_text(n, "id"), n))
        .collect();
    let base = path.parent().ok_or("Project directory is missing")?;
    fn add(
        node: roxmltree::Node<'_, '_>,
        parent: Option<u32>,
        base: &Path,
        map: &HashMap<String, roxmltree::Node<'_, '_>>,
        bundle: &mut Bundle,
        depth: usize,
    ) -> Result<()> {
        if depth > 31 || bundle.sources.len() >= 1000 {
            return Err("QGIS project exceeds the layer hierarchy limit".into());
        }
        let id = bundle.sources.len() as u32 + 1;
        let hidden = node.attribute("checked") == Some("Qt::Unchecked")
            || node.attribute("checked") == Some("0");
        if node.has_tag_name("layer-tree-group") {
            bundle.sources.push(Source {
                layer: Layer {
                    id,
                    name: node
                        .attribute("name")
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Group")
                        .into(),
                    source: "Group".into(),
                    parent,
                    hidden,
                    warnings: vec![],
                },
                crs: String::new(),
                shapes: vec![],
            });
            for child in node.children().filter(|n| {
                n.has_tag_name("layer-tree-group") || n.has_tag_name("layer-tree-layer")
            }) {
                add(child, Some(id), base, map, bundle, depth + 1)?;
            }
            return Ok(());
        }
        let original = if node.has_tag_name("maplayer") {
            node
        } else {
            *map.get(node.attribute("id").unwrap_or(""))
                .ok_or("QGIS layer definition is missing")?
        };
        let name = child_text(original, "layername");
        let mut source = Source {
            layer: Layer {
                id,
                name: if name.is_empty() {
                    format!("Layer {id}")
                } else {
                    name
                },
                source: "QGIS".into(),
                parent,
                hidden,
                warnings: vec![],
            },
            crs: source_crs(original),
            shapes: vec![],
        };
        let provider = child_text(original, "provider");
        let ds = child_text(original, "datasource");
        let result = (|| {
            if !["ogr", ""].contains(&provider.as_str()) {
                return Err(format!(
                    "Provider {provider} is not imported; only local vector layers are supported."
                ));
            }
            let mut parts = ds.split('|');
            let file = parts.next().unwrap_or("");
            if file.is_empty()
                || file.contains("://")
                || file.starts_with("\\\\")
                || file.starts_with("/vsi")
                || file.contains("dbname=")
            {
                return Err(
                    "Remote, database and virtual sources are not loaded automatically.".into(),
                );
            }
            let mut table = None;
            for part in parts {
                if let Some(value) = part.strip_prefix("layername=") {
                    table = Some(value.to_owned());
                } else if let Some(value) = part.strip_prefix("layerid=") {
                    table = Some(format!("#{value}"));
                } else if !part.is_empty() {
                    return Err(format!("Unsupported QGIS source option: {part}"));
                }
            }
            let file = PathBuf::from(file);
            let file = if file.is_absolute() {
                file
            } else {
                base.join(file)
            };
            let (crs, shapes) = read_vector(
                &file,
                table.as_deref(),
                if source.crs.is_empty() {
                    None
                } else {
                    Some(&source.crs)
                },
            )?;
            source.crs = crs;
            source.shapes = shapes;
            Ok(())
        })();
        if let Err(e) = result {
            let message = format!("{}: {}", source.layer.name, e);
            source.layer.warnings.push(message.clone());
            bundle.warnings.push(message);
        }
        bundle.sources.push(source);
        check_limits(bundle)?;
        Ok(())
    }
    if let Some(tree) = root.children().find(|n| n.has_tag_name("layer-tree-group")) {
        for node in tree
            .children()
            .filter(|n| n.has_tag_name("layer-tree-group") || n.has_tag_name("layer-tree-layer"))
        {
            add(node, None, base, &map, &mut bundle, 0)?;
        }
    } else {
        for node in root.descendants().filter(|n| n.has_tag_name("maplayer")) {
            add(node, None, base, &map, &mut bundle, 0)?;
        }
    }
    check_bundle(&bundle)?;
    Ok(bundle)
}
fn check_limits(bundle: &Bundle) -> Result<()> {
    if bundle.sources.len() > 1000 {
        return Err("QGIS project has too many layers".into());
    }
    let mut count = 0usize;
    let mut vertices = 0usize;
    for s in &bundle.sources {
        count += s.shapes.len();
        for g in &s.shapes {
            for ring in &g.rings {
                vertices += ring.len();
            }
        }
    }
    if count > 20000 || vertices > 200000 {
        return Err("QGIS vectors exceed 20,000 features or 200,000 vertices".into());
    }
    Ok(())
}
fn check_bundle(bundle: &Bundle) -> Result<()> {
    check_limits(bundle)?;
    if bundle.sources.iter().all(|s| s.shapes.is_empty()) {
        return Err(format!(
            "No supported line or polygon data could be read. {}",
            bundle.warnings.join(" · ")
        ));
    }
    Ok(())
}
pub fn inspect(bundle: &Bundle) -> Result<serde_json::Value> {
    check_bundle(bundle)?;
    let mut center = [0., 0.];
    let mut n = 0.;
    for source in &bundle.sources {
        for shape in &source.shapes {
            if let Some(p) = shape.rings.first().and_then(|r| r.first()) {
                let point = projection::transform(&source.crs, "EPSG:4326", *p)?;
                center[0] += point[0];
                center[1] += point[1];
                n += 1.;
            }
        }
    }
    if n == 0. {
        return Err("The vector source has no coordinates".into());
    }
    let zone = (((center[0] / n + 180.) / 6.).floor() as u8 + 1).clamp(1, 60);
    let south = center[1] / n < 0.;
    let declared = bundle
        .crs
        .as_ref()
        .filter(|id| projection::validate(id).is_ok());
    let crs = if let Some(id) = declared {
        Crs {
            zone,
            south,
            authority: Some(id.clone()),
        }
    } else {
        Crs {
            zone,
            south,
            authority: None,
        }
    };
    Ok(
        serde_json::json!({"crs":crs,"kind":if declared.is_some(){"declared"}else{"suggested"},"warnings":bundle.warnings,"qgis":true}),
    )
}
pub fn import(bundle: &Bundle, crs: Crs) -> Result<Project> {
    check_bundle(bundle)?;
    crs.validate()?;
    let target = crs.authority.clone().unwrap_or_else(|| {
        format!(
            "EPSG:{}",
            if crs.south { 32700 } else { 32600 } + u32::from(crs.zone)
        )
    });
    let mut project = Project::empty(crs);
    project.version = 6;
    project.name = Some(bundle.title.clone());
    project.warnings = bundle.warnings.clone();
    for source in &bundle.sources {
        project.layers.push(source.layer.clone());
        for (i, shape) in source.shapes.iter().enumerate() {
            let mut rings = vec![];
            for ring in &shape.rings {
                let mut out = Vec::with_capacity(ring.len());
                for &p in ring {
                    out.push(projection::transform(&source.crs, &target, p)?);
                }
                rings.push(out);
            }
            match shape.kind.as_str() {
                "line" => {
                    let points = rings.into_iter().next().ok_or("Empty QGIS line")?;
                    project.lines.push(Line {
                        name: format!("Q{}_{}", source.layer.id, i + 1),
                        points,
                        layer: Some(source.layer.id),
                        hidden: false,
                        curve: None,
                    });
                }
                "polygon" => {
                    if rings.is_empty() {
                        return Err("Empty QGIS polygon".into());
                    }
                    let outer = rings.remove(0);
                    project.polygons.push(Polygon {
                        name: Some(format!("{} {}", source.layer.name, i + 1)),
                        outer,
                        holes: rings,
                        layer: Some(source.layer.id),
                        hidden: false,
                        curve: None,
                    });
                }
                _ => return Err("Unsupported QGIS geometry".into()),
            }
        }
    }
    project.validate()?;
    Ok(project)
}
fn read_vector(
    path: &Path,
    table: Option<&str>,
    declared: Option<&str>,
) -> Result<(String, Vec<Shape>)> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "geojson" | "json" => {
            let value: serde_json::Value =
                serde_json::from_str(&small_text(path)?).map_err(error)?;
            let crs = declared
                .map(str::to_owned)
                .or_else(|| {
                    value
                        .pointer("/crs/properties/name")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned)
                })
                .unwrap_or("EPSG:4326".into());
            let mut shapes = vec![];
            if value["type"] == "FeatureCollection" {
                for feature in value["features"]
                    .as_array()
                    .ok_or("Invalid GeoJSON features")?
                {
                    json_geometry(&feature["geometry"], &mut shapes, 0)?;
                }
            } else if value["type"] == "Feature" {
                json_geometry(&value["geometry"], &mut shapes, 0)?;
            } else {
                json_geometry(&value, &mut shapes, 0)?;
            }
            Ok((crs, shapes))
        }
        "gpkg" => read_gpkg(path, table, declared),
        "shp" => {
            let crs = if let Some(crs) = declared {
                crs.to_owned()
            } else {
                small_text(&path.with_extension("prj"))
                    .map_err(|_| "Shapefile CRS is missing; keep its .prj beside the .shp")?
            };
            Ok((crs, read_shp(&small_read(path, 32 * 1024 * 1024)?)?))
        }
        "kml" | "kmz" => {
            let xml = if ext == "kmz" {
                crate::kmz::read(&crate::files::base64(&small_read(path, 8 * 1024 * 1024)?))?
            } else {
                small_text(path)?
            };
            Ok(("EPSG:4326".into(), read_kml(&xml)?))
        }
        _ => Err(format!(
            "Unsupported local vector format .{ext}; use GeoPackage, Shapefile, GeoJSON or KML."
        )),
    }
}
fn json_geometry(value: &serde_json::Value, out: &mut Vec<Shape>, depth: usize) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    if depth > 16 || out.len() > 20000 {
        return Err("Vector collection is too complex".into());
    }
    let coords = &value["coordinates"];
    let kind = value["type"].as_str().ok_or("Invalid vector geometry")?;
    fn points(v: &serde_json::Value) -> Result<Vec<Point>> {
        v.as_array()
            .ok_or("Invalid coordinate array")?
            .iter()
            .map(|p| {
                Ok([
                    p[0].as_f64().ok_or("Invalid X coordinate")?,
                    p[1].as_f64().ok_or("Invalid Y coordinate")?,
                ])
            })
            .collect()
    }
    match kind {
        "LineString" => out.push(Shape {
            kind: "line".into(),
            rings: vec![points(coords)?],
        }),
        "Polygon" => out.push(Shape {
            kind: "polygon".into(),
            rings: coords
                .as_array()
                .ok_or("Invalid polygon rings")?
                .iter()
                .map(points)
                .collect::<Result<_>>()?,
        }),
        "MultiLineString" | "MultiPolygon" => {
            let single = if kind == "MultiLineString" {
                "LineString"
            } else {
                "Polygon"
            };
            for coordinates in coords.as_array().ok_or("Invalid multi geometry")? {
                json_geometry(
                    &serde_json::json!({"type":single,"coordinates":coordinates}),
                    out,
                    depth + 1,
                )?;
            }
        }
        "GeometryCollection" => {
            for g in value["geometries"]
                .as_array()
                .ok_or("Invalid geometry collection")?
            {
                json_geometry(g, out, depth + 1)?;
            }
        }
        "Point" | "MultiPoint" => {
            return Err("Point layers are not supported in the line workspace".into());
        }
        _ => return Err(format!("Unsupported geometry {kind}")),
    }
    Ok(())
}
fn read_kml(xml: &str) -> Result<Vec<Shape>> {
    let doc = roxmltree::Document::parse(xml).map_err(error)?;
    fn coords(n: roxmltree::Node<'_, '_>) -> Result<Vec<Point>> {
        let data = text(n, "coordinates");
        data.split_whitespace()
            .map(|tuple| {
                let mut p = tuple.split(',');
                Ok([
                    p.next()
                        .ok_or("Missing longitude")?
                        .parse()
                        .map_err(error)?,
                    p.next().ok_or("Missing latitude")?.parse().map_err(error)?,
                ])
            })
            .collect()
    }
    let mut out = vec![];
    for n in doc.descendants() {
        if n.has_tag_name("LineString") {
            out.push(Shape {
                kind: "line".into(),
                rings: vec![coords(n)?],
            });
        }
        if n.has_tag_name("Polygon") {
            let outer = n
                .children()
                .find(|n| n.has_tag_name("outerBoundaryIs"))
                .ok_or("KML polygon has no outer ring")?;
            let mut rings = vec![coords(outer)?];
            for hole in n.children().filter(|n| n.has_tag_name("innerBoundaryIs")) {
                rings.push(coords(hole)?);
            }
            out.push(Shape {
                kind: "polygon".into(),
                rings,
            });
        }
    }
    Ok(out)
}
fn i32le(bytes: &[u8], at: usize) -> Result<i32> {
    Ok(i32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or("Truncated vector record")?
            .try_into()
            .map_err(error)?,
    ))
}
fn f64le(bytes: &[u8], at: usize) -> Result<f64> {
    Ok(f64::from_le_bytes(
        bytes
            .get(at..at + 8)
            .ok_or("Truncated vector coordinate")?
            .try_into()
            .map_err(error)?,
    ))
}
fn read_shp(bytes: &[u8]) -> Result<Vec<Shape>> {
    if bytes.len() < 100 || i32::from_be_bytes(bytes[..4].try_into().map_err(error)?) != 9994 {
        return Err("Invalid Shapefile header".into());
    }
    let mut at = 100;
    let mut out = vec![];
    while at < bytes.len() {
        let header = bytes.get(at..at + 8).ok_or("Truncated Shapefile record")?;
        let words = i32::from_be_bytes(header[4..8].try_into().map_err(error)?);
        if words < 0 {
            return Err("Invalid Shapefile record length".into());
        }
        let end = at
            .checked_add(8)
            .and_then(|n| n.checked_add(words as usize * 2))
            .ok_or("Shapefile size overflow")?;
        let record = bytes.get(at + 8..end).ok_or("Truncated Shapefile record")?;
        at = end;
        let kind = i32le(record, 0)?;
        if kind == 0 {
            continue;
        }
        if ![3, 5, 13, 15, 23, 25].contains(&kind) {
            return Err("Only line and polygon Shapefiles are supported".into());
        }
        let parts = i32le(record, 36)?;
        let count = i32le(record, 40)?;
        if !(1..=20000).contains(&parts) || !(2..=200000).contains(&count) {
            return Err("Shapefile vertex limit exceeded".into());
        }
        let start = 44 + parts as usize * 4;
        let mut ranges = vec![];
        for i in 0..parts as usize {
            let n = i32le(record, 44 + i * 4)?;
            if n < 0 || n >= count || ranges.last().is_some_and(|&v| v >= n as usize) {
                return Err("Invalid Shapefile part offsets".into());
            }
            ranges.push(n as usize);
        }
        if ranges[0] != 0 {
            return Err("Invalid first Shapefile part".into());
        }
        ranges.push(count as usize);
        let mut rings = vec![];
        for part in ranges.windows(2) {
            let mut points = vec![];
            for i in part[0]..part[1] {
                points.push([
                    f64le(record, start + i * 16)?,
                    f64le(record, start + i * 16 + 8)?,
                ]);
            }
            rings.push(points);
        }
        if [3, 13, 23].contains(&kind) {
            for ring in rings {
                out.push(Shape {
                    kind: "line".into(),
                    rings: vec![ring],
                });
            }
        } else {
            out.extend(polygon_rings(rings)?);
        }
        if out.len() > 20000 {
            return Err("Shapefile feature limit exceeded".into());
        }
    }
    Ok(out)
}
fn polygon_rings(rings: Vec<Vec<Point>>) -> Result<Vec<Shape>> {
    fn inside(p: Point, ring: &[Point]) -> bool {
        let mut inside = false;
        for pair in ring.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
        }
        inside
    }
    if rings.iter().any(|r| r.len() < 4 || r.first() != r.last()) {
        return Err("Invalid Shapefile polygon ring".into());
    }
    let depths: Vec<_> = rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            rings
                .iter()
                .enumerate()
                .filter(|(j, other)| *j != i && inside(r[0], other))
                .count()
        })
        .collect();
    let mut output = vec![];
    for (i, ring) in rings.iter().enumerate() {
        if depths[i] % 2 == 0 {
            let mut parts = vec![ring.clone()];
            for (j, hole) in rings.iter().enumerate() {
                if depths[j] == depths[i] + 1 && inside(hole[0], ring) {
                    parts.push(hole.clone());
                }
            }
            output.push(Shape {
                kind: "polygon".into(),
                rings: parts,
            });
        }
    }
    Ok(output)
}
fn read_gpkg(
    path: &Path,
    table: Option<&str>,
    declared: Option<&str>,
) -> Result<(String, Vec<Shape>)> {
    use crate::geopackage::{Database, quote};
    let db = Database::open(path)?;
    let tables = db.query(
        "SELECT table_name,column_name,srs_id FROM gpkg_geometry_columns ORDER BY table_name",
    )?;
    let chosen=if let Some(id)=table.and_then(|s|s.strip_prefix('#')){tables.get(id.parse::<usize>().map_err(error)?)}else if let Some(name)=table{tables.iter().find(|r|r[0].text().is_ok_and(|s|s==name))}else if tables.len()==1{tables.first()}else{return Err("GeoPackage contains multiple vector layers; open its QGIS project to select named layers".into());}.ok_or("GeoPackage vector layer is missing")?;
    let name = chosen[0].text()?;
    let geom = chosen[1].text()?;
    let srs = chosen[2].text()?.parse::<i32>().map_err(error)?;
    let crs = if let Some(id) = declared {
        id.to_owned()
    } else {
        let row=db.query(&format!("SELECT organization,organization_coordsys_id,definition FROM gpkg_spatial_ref_sys WHERE srs_id={srs}"))?;
        let row = row.first().ok_or("GeoPackage CRS is missing")?;
        if row[0].text()?.eq_ignore_ascii_case("EPSG") {
            format!("EPSG:{}", row[1].text()?)
        } else {
            row[2].text()?.to_owned()
        }
    };
    let rows = db.query(&format!(
        "SELECT {} FROM {} WHERE {} IS NOT NULL LIMIT 20001",
        quote(geom),
        quote(name),
        quote(geom)
    ))?;
    let mut out = vec![];
    let mut total = 0usize;
    for row in rows {
        let blob = row[0].blob()?;
        if blob.len() < 8 || &blob[..2] != b"GP" || blob[2] != 0 {
            return Err("Invalid GeoPackage geometry header".into());
        }
        let flags = blob[3];
        let header: [u8; 4] = blob[4..8].try_into().map_err(error)?;
        let geometry_srs = if flags & 1 != 0 {
            i32::from_le_bytes(header)
        } else {
            i32::from_be_bytes(header)
        };
        if geometry_srs != srs {
            return Err("GeoPackage geometry CRS differs from its layer metadata".into());
        }

        if flags & 16 != 0 {
            continue;
        }
        if flags & 32 != 0 {
            return Err("Extended GeoPackage geometries are not supported".into());
        }
        let envelope = match (flags >> 1) & 7 {
            0 => 0,
            1 => 32,
            2 | 3 => 48,
            4 => 64,
            _ => return Err("Invalid GeoPackage envelope".into()),
        };
        let mut reader = Wkb {
            bytes: blob
                .get(8 + envelope..)
                .ok_or("Truncated GeoPackage envelope")?,
            at: 0,
            vertices: &mut total,
        };
        reader.geometry(&mut out, 0)?;
        if reader.at != reader.bytes.len() {
            return Err("Unexpected trailing WKB data".into());
        }
    }
    Ok((crs, out))
}
struct Wkb<'a> {
    bytes: &'a [u8],
    at: usize,
    vertices: &'a mut usize,
}
impl Wkb<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let result = self
            .bytes
            .get(self.at..self.at + N)
            .ok_or("Truncated WKB geometry")?
            .try_into()
            .map_err(error)?;
        self.at += N;
        Ok(result)
    }
    fn number(&mut self, le: bool) -> Result<u32> {
        let b = self.take()?;
        Ok(if le {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    }
    fn point(&mut self, le: bool, dim: usize) -> Result<Point> {
        *self.vertices += 1;
        if *self.vertices > 200000 {
            return Err("GeoPackage vertex limit exceeded".into());
        }
        let mut p = [0.; 2];
        for target in p.iter_mut().map(Some).chain((2..dim).map(|_| None)) {
            let b = self.take()?;
            let n = if le {
                f64::from_le_bytes(b)
            } else {
                f64::from_be_bytes(b)
            };
            if !n.is_finite() {
                return Err("Invalid WKB coordinate".into());
            }
            if let Some(out) = target {
                *out = n;
            }
        }
        Ok(p)
    }
    fn points(&mut self, le: bool, dim: usize) -> Result<Vec<Point>> {
        let n = self.number(le)?;
        if n > 200000 {
            return Err("WKB point limit exceeded".into());
        }
        (0..n).map(|_| self.point(le, dim)).collect()
    }
    fn geometry(&mut self, out: &mut Vec<Shape>, depth: usize) -> Result<()> {
        if depth > 16 || out.len() > 20000 {
            return Err("WKB collection limit exceeded".into());
        }
        let le = match self.take::<1>()?[0] {
            0 => false,
            1 => true,
            _ => return Err("Invalid WKB endian flag".into()),
        };
        let raw = self.number(le)?;
        let base = raw & 0x1fff_ffff;
        let dimensional = base / 1000;
        let kind = base % 1000;
        let dim = 2
            + usize::from(raw & 0x8000_0000 != 0 || dimensional == 1 || dimensional == 3)
            + usize::from(raw & 0x4000_0000 != 0 || dimensional == 2 || dimensional == 3);
        if raw & 0x2000_0000 != 0 {
            self.number(le)?;
        }
        match kind {
            2 => {
                let points = self.points(le, dim)?;
                out.push(Shape {
                    kind: "line".into(),
                    rings: vec![points],
                });
            }
            3 => {
                let n = self.number(le)?;
                if n > 20000 {
                    return Err("WKB ring limit exceeded".into());
                }
                let mut rings = vec![];
                for _ in 0..n {
                    rings.push(self.points(le, dim)?);
                }
                out.push(Shape {
                    kind: "polygon".into(),
                    rings,
                });
            }
            5..=7 => {
                let n = self.number(le)?;
                if n > 20000 {
                    return Err("WKB collection limit exceeded".into());
                }
                for _ in 0..n {
                    self.geometry(out, depth + 1)?;
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported WKB geometry type {kind}; only linear lines and polygons are imported"
                ));
            }
        }
        Ok(())
    }
}
