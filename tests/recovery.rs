use serde_json::json;
use skylab_rust::recovery::Recovery;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
static ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn crashed_process_keeps_snapshot_and_normal_shutdown_removes_it() {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpStream,
        process::{Command, Stdio},
    };
    let root = std::env::temp_dir().join(format!("skylab-recovery-process-{}", std::process::id()));
    let start = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_skylab-rust"))
            .args(["--no-open", "--port", "0"])
            .env("SKYLAB_RECOVERY_DIR", &root)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        let port = loop {
            line.clear();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if let Some(port) = line.trim().strip_prefix("Abra http://127.0.0.1:") {
                break port.parse::<u16>().unwrap();
            }
        };
        (child, port)
    };
    let call = |port, method: &str, path: &str, body: &str| {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        write!(stream,"{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",body.len()).unwrap();
        let mut out = String::new();
        stream.read_to_string(&mut out).unwrap();
        out
    };
    let (mut first, port) = start();
    let session = json!({"active":1,"documents":[{"id":1,"state":{"project":null,"dirty":true,"title":"Crash recovery"}}]});
    assert!(call(port, "POST", "/recovery", &session.to_string()).contains("\"ok\":true"));
    first.kill().unwrap();
    first.wait().unwrap();
    assert!(root.join("session.json").exists());
    let (mut second, port) = start();
    assert!(call(port, "GET", "/recovery", "").contains("Crash recovery"));
    assert!(call(port, "POST", "/shutdown", "{}").contains("\"ok\":true"));
    assert!(second.wait().unwrap().success());
    assert!(!root.join("session.json").exists());
    fs::remove_dir(root).unwrap();
}
#[test]
fn recovery_replaces_previous_snapshot_and_clears_only_owned_files() {
    let root = std::env::temp_dir().join(format!(
        "skylab-recovery-{}-{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ));
    let store = Recovery::at(root.clone());
    assert!(store.load().unwrap().is_none());
    let mut session = json!({"active":1,"documents":[{"id":1,"state":{"project":null,"dirty":true,"title":"Synthetic"}}]});
    store.save(session.clone()).unwrap();
    assert_eq!(store.load().unwrap().unwrap()["session"], session);
    session["documents"][0]["state"]["title"] = json!("Latest");
    store.save(session.clone()).unwrap();
    assert_eq!(store.load().unwrap().unwrap()["session"], session);
    assert!(!root.join("session.new").exists());
    fs::write(root.join("personal.txt"), "keep").unwrap();
    store.clear().unwrap();
    assert!(store.load().unwrap().is_none());
    assert_eq!(
        fs::read_to_string(root.join("personal.txt")).unwrap(),
        "keep"
    );
    fs::remove_file(root.join("personal.txt")).unwrap();
    fs::remove_dir(root).unwrap();
}
#[test]
fn malformed_recovery_is_reported_and_invalid_save_preserves_previous_snapshot() {
    let root = std::env::temp_dir().join(format!(
        "skylab-recovery-{}-{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ));
    let store = Recovery::at(root.clone());
    let session = json!({"active":1,"documents":[{"id":1,"state":{}}]});
    store.save(session.clone()).unwrap();
    assert!(store.save(json!({"active":5,"documents":[]})).is_err());
    assert_eq!(store.load().unwrap().unwrap()["session"], session);
    fs::write(root.join("session.json"), "broken JSON").unwrap();
    assert!(store.load().is_err());
    assert!(root.join("session.json").exists());
    store.clear().unwrap();
    fs::remove_dir(root).unwrap();
}
