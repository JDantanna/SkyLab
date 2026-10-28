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
}
