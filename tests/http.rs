use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    process::{Child, Command, Stdio},
    time::Duration,
};
struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn call(port: u16, method: &str, path: &str, body: &str, origin: Option<&str>) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let extra = origin
        .map(|o| format!("Origin: {o}\r\n"))
        .unwrap_or_default();
    write!(s,"{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{extra}\r\n{body}",body.len()).unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}
#[test]
fn local_server_serves_ui_and_validates_api() {
    let mut child = Guard(
        Command::new(env!("CARGO_BIN_EXE_skylab-rust"))
            .args(["--no-open", "--port", "0"])
            .env(
                "SKYLAB_UPDATES_CONFIG",
                std::env::temp_dir()
                    .join(format!("skylab-missing-config-{}.json", std::process::id())),
            )
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut reader = BufReader::new(child.0.stdout.take().unwrap());
    let mut line = String::new();
    let port = loop {
        line.clear();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if let Some(url) = line.trim().strip_prefix("Abra http://127.0.0.1:") {
            break url.parse::<u16>().unwrap();
        }
    };
    let ui = call(port, "GET", "/", "", None);
    assert!(ui.starts_with("HTTP/1.1 200"));
    assert!(ui.contains("Content-Security-Policy:"));
    assert!(ui.contains("Gerar linhas no bloco"));
    let updates = call(port, "GET", "/updates/check", "", None);
    assert!(updates.starts_with("HTTP/1.1 200"));
    assert!(updates.contains("not_configured"));
    assert!(updates.contains(skylab_rust::updates::display_version()));
    let js = call(port, "GET", "/app.js", "", None);
    assert!(js.starts_with("HTTP/1.1 200"));
    assert!(js.contains("async function api"));
    assert!(call(port, "GET", "/brand.jpg", "", None).starts_with("HTTP/1.1 404"));
    let brand = call(port, "GET", "/brand.svg", "", None);
    assert!(brand.starts_with("HTTP/1.1 200"));
    assert!(brand.contains("Cache-Control: no-store"));
    assert!(
        call(port, "POST", "/api", "{}", Some("https://example.com")).starts_with("HTTP/1.1 403")
    );
    assert!(call(port, "POST", "/api", "{}", None).starts_with("HTTP/1.1 400"));
    let body=serde_json::json!({"op":"import-mis","crs":{"zone":23,"south":true},"text":"L100100\n500000 7800000\n500100 7800100"}).to_string();
    let imported = call(port, "POST", "/api", &body, None);
    assert!(imported.starts_with("HTTP/1.1 200"));
    let data: serde_json::Value =
        serde_json::from_str(imported.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(
        data["data"]["lines"][0]["points"].as_array().unwrap().len(),
        2
    );
    let exported = call(
        port,
        "POST",
        "/api",
        &serde_json::json!({"op":"export-mis","project":data["data"]}).to_string(),
        None,
    );
    assert!(exported.starts_with("HTTP/1.1 200"));
    assert!(exported.contains("500000.00 7800000.00"));
    let inspect = |text: &str| -> serde_json::Value {
        let response = call(
            port,
            "POST",
            "/api",
            &serde_json::json!({"op":"inspect-crs","text":text}).to_string(),
            None,
        );
        serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
    };
    let kml = inspect(
        "<kml><LineString><coordinates>-45,-20 -44.99,-20</coordinates></LineString></kml>",
    );
    assert_eq!(kml["data"]["kind"], "suggested");
    assert_eq!(kml["data"]["crs"]["zone"], 23);
    assert_eq!(kml["data"]["crs"]["south"], true);
    assert_eq!(
        inspect("L100100\n500000 7800000\n500100 7800100")["data"]["kind"],
        "unknown"
    );
    assert_eq!(
        inspect(&data["data"].to_string())["data"]["kind"],
        "declared"
    );
    let response=call(port,"POST","/api",&serde_json::json!({"op":"reproject","project":data["data"],"crs":{"zone":23,"south":true,"authority":"EPSG:31983"}}).to_string(),None);
    let projected: serde_json::Value =
        serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(projected["ok"], true);
    assert_eq!(projected["data"]["version"], 2);
    assert_eq!(projected["data"]["crs"]["authority"], "EPSG:31983");
    let geographic = call(
        port,
        "POST",
        "/api",
        &serde_json::json!({"op":"geographic","project":projected["data"]}).to_string(),
        None,
    );
    let geographic: serde_json::Value =
        serde_json::from_str(geographic.split_once("\r\n\r\n").unwrap().1).unwrap();
    let lon = geographic["data"]["lines"][0]["points"][0][0]
        .as_f64()
        .unwrap();
    assert!((lon + 45.).abs() < 1e-6);
    let selected = call(
        port,
        "POST",
        "/api",
        &serde_json::json!({"op":"export-selection","project":data["data"],"indices":[0,0]})
            .to_string(),
        None,
    );
    let selected: serde_json::Value =
        serde_json::from_str(selected.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(selected["ok"], true);
    assert!(
        selected["data"]["text"]
            .as_str()
            .unwrap()
            .contains("500000.00 7800000.00")
    );
    for indices in [serde_json::json!([]), serde_json::json!([99])] {
        assert!(call(port,"POST","/api",&serde_json::json!({"op":"export-selection","project":data["data"],"indices":indices}).to_string(),None).starts_with("HTTP/1.1 400"));
    }
    for resource in [
        "panels.js",
        "files.js",
        "panels.css",
        "brand-light.svg",
        "brand-dark.svg",
    ] {
        assert!(call(port, "GET", &format!("/{resource}"), "", None).starts_with("HTTP/1.1 200"));
    }
    assert!(
        call(
            port,
            "POST",
            "/files/enqueue",
            r#"{"paths":[]}"#,
            Some("https://example.com")
        )
        .starts_with("HTTP/1.1 403")
    );
    let file = std::env::temp_dir().join(format!("skylab-open-{}.mis", std::process::id()));
    std::fs::write(&file, "L1\n500000 7800000\n500100 7800100").unwrap();
    let forwarded = Command::new(env!("CARGO_BIN_EXE_skylab-rust"))
        .args(["--no-open", "--port", &port.to_string(), "--open"])
        .arg(&file)
        .status()
        .unwrap();
    assert!(forwarded.success());
    let pending = call(port, "GET", "/files/pending", "", None);
    let pending: serde_json::Value =
        serde_json::from_str(pending.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(pending["data"]["new_project"], true);
    let id = pending["data"]["id"].as_u64().unwrap();
    let read = call(
        port,
        "POST",
        "/files/read",
        &serde_json::json!({"id":id}).to_string(),
        None,
    );
    assert!(read.contains("L1"));
    assert!(
        call(
            port,
            "POST",
            "/files/ack",
            &serde_json::json!({"id":id}).to_string(),
            None
        )
        .starts_with("HTTP/1.1 200")
    );
    assert!(call(port, "GET", "/files/pending", "", None).contains("\"data\":null"));
    std::fs::remove_file(file).unwrap();
    let shape = |layer: u32, x: f64| serde_json::json!({"layer":layer,"outer":[[x,7800000.],[x+1000.,7800000.],[x+1000.,7801000.],[x,7801000.],[x,7800000.]],"holes":[]});
    let areas = serde_json::json!({"version":3,"crs":{"zone":23,"south":true},"lines":[],"polygons":[shape(1,500000.),shape(2,503000.)],"warnings":[],"layers":[{"id":1,"name":"First","source":"KML"},{"id":2,"name":"Second","source":"KML"}]});
    let generated=call(port,"POST","/api",&serde_json::json!({"op":"generate","index":0,"project":areas,"settings":{"spacing":100.,"angle":0.,"offset":50.,"min_length":1.,"prefix":"L","first":1,"increment":1,"alternate":false}}).to_string(),None);
    let generated: serde_json::Value =
        serde_json::from_str(generated.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(generated["ok"], true);
    assert_eq!(generated["data"]["polygons"].as_array().unwrap().len(), 2);
    assert!(
        generated["data"]["layers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["id"] == 2)
    );
    for line in generated["data"]["lines"].as_array().unwrap() {
        for point in line["points"].as_array().unwrap() {
            assert!(point[0].as_f64().unwrap() <= 501000.);
        }
    }
    let curve_points = vec![
        serde_json::json!({"point":[500000.,7800000.],"outgoing":[500500.,7799500.]}),
        serde_json::json!({"point":[501000.,7800000.],"incoming":[501000.,7799500.]}),
        serde_json::json!({"point":[501000.,7801000.]}),
        serde_json::json!({"point":[500000.,7801000.]}),
    ];
    let drawn=call(port,"POST","/api",&serde_json::json!({"op":"upsert-polygon","project":{"version":1,"crs":{"zone":23,"south":true},"polygons":[],"lines":[],"warnings":[]},"anchors":curve_points}).to_string(),None);
    let drawn: serde_json::Value =
        serde_json::from_str(drawn.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(drawn["ok"], true);
    assert_eq!(drawn["data"]["version"], 4);
    let reprojected=call(port,"POST","/api",&serde_json::json!({"op":"reproject","project":drawn["data"],"crs":{"zone":23,"south":true,"authority":"EPSG:31983"}}).to_string(),None);
    let reprojected: serde_json::Value =
        serde_json::from_str(reprojected.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(reprojected["ok"], true);
    assert!(reprojected["data"]["polygons"][0]["curve"].is_null());
    assert_eq!(
        reprojected["data"]["polygons"][0]["outer"]
            .as_array()
            .unwrap()
            .len(),
        drawn["data"]["polygons"][0]["outer"]
            .as_array()
            .unwrap()
            .len()
    );
    assert!(
        reprojected["data"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|text| text.as_str().unwrap_or("").contains("curves"))
    );
    let surface = serde_json::json!({"version":1,"crs":{"zone":23,"south":true},"layers":[],"lines":[],"warnings":[],"polygons":[{"outer":[[500000,7800000],[501000,7800000],[501000,7801000],[500000,7801000],[500000,7800000]],"holes":[]}]});
    let settings = serde_json::json!({"spacing":100,"angle":120,"offset":50,"min_length":1,"prefix":"L","first":100100,"increment":100,"alternate":false});
    let fetch_project = |op: &str| {
        let reply = call(
            port,
            "POST",
            "/api",
            &serde_json::json!({"op":op,"project":surface,"settings":settings}).to_string(),
            None,
        );
        let v: serde_json::Value =
            serde_json::from_str(reply.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(v["ok"], true);
        serde_json::from_value::<skylab_rust::Project>(v["data"].clone()).unwrap()
    };
    let preview = fetch_project("generate-preview");
    let committed = fetch_project("generate");
    assert!(preview.polygons.is_empty());
    assert!(preview.layers.is_empty());
    assert_eq!(preview.lines.len(), committed.lines.len());
    for (a, b) in preview.lines.iter().zip(&committed.lines) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.points, b.points);
    }
    // Split at cumulative distance, including an interior vertex and colliding suffixes.
    let mission = serde_json::json!({"version":1,"crs":{"zone":23,"south":true},"layers":[],"polygons":[],"warnings":[],"lines":[{"name":"L1","points":[[500000,7800000],[500030,7800000],[500030,7800040],[500100,7800040]]},{"name":"L1_A","points":[[500000,7800100],[500100,7800100]]}]});
    for at in [30., 50.] {
        let reply = call(
            port,
            "POST",
            "/api",
            &serde_json::json!({"op":"split","project":mission,"index":0,"start":at}).to_string(),
            None,
        );
        let reply: serde_json::Value =
            serde_json::from_str(reply.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(reply["ok"], true, "{reply}");
        let result: skylab_rust::Project = serde_json::from_value(reply["data"].clone()).unwrap();
        assert_eq!(result.lines.len(), 3);
        let (a, b) = (&result.lines[0], &result.lines[1]);
        assert_eq!(a.points.last(), b.points.first());
        assert!((skylab_rust::length(a) - at).abs() < 1e-7);
        assert!((skylab_rust::length(b) - (140. - at)).abs() < 1e-7);
        assert_eq!(a.name, "L1_A2");
        assert_eq!(b.name, "L1_B");
        assert_eq!(b.points.last(), Some(&[500100., 7800040.]));
    }
    for at in [0., 140., -1.] {
        let reply = call(
            port,
            "POST",
            "/api",
            &serde_json::json!({"op":"split","project":mission,"index":0,"start":at}).to_string(),
            None,
        );
        let reply: serde_json::Value =
            serde_json::from_str(reply.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(reply["ok"], false);
    }
}
