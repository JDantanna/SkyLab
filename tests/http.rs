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
    let js = call(port, "GET", "/app.js", "", None);
    assert!(js.starts_with("HTTP/1.1 200"));
    assert!(js.contains("function commit"));
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
}
