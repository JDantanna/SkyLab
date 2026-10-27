use skylab_rust::updates;
#[test]
fn github_releases_only_offer_newer_stable_installers_from_this_repository() {
    let release = |tag: &str, url: &str| {
        serde_json::json!({"tag_name":tag,"body":"Release notes","assets":[{"name":"SkyTEM-SkyLab-Setup-0.2.exe","state":"uploaded","browser_download_url":url}]}).to_string()
    };
    let trusted =
        "https://github.com/JDantanna/SkyLab/releases/download/v0.2/SkyTEM-SkyLab-Setup-0.2.exe";
    let check = updates::github_release(&release("v0.2", trusted)).unwrap();
    assert_eq!(check.status, "available");
    assert_eq!(check.download_url.as_deref(), Some(trusted));
    assert_eq!(
        updates::github_release(&release("v0.1.0", trusted))
            .unwrap()
            .status,
        "up_to_date"
    );
    for bad in [
        "https://example.com/setup.exe",
        "https://github.com/JDantanna/SkyLab.evil/releases/download/a.exe",
        "http://github.com/JDantanna/SkyLab/releases/download/a.exe",
    ] {
        let check = updates::github_release(&release("v0.2", bad)).unwrap();
        assert_eq!(check.status, "release_pending");
        assert!(check.download_url.is_none());
    }
    for text in [
        r#"{"no_release":true}"#,
        r#"{"draft":true}"#,
        r#"{"prerelease":true}"#,
    ] {
        assert_eq!(updates::github_release(text).unwrap().status, "no_release");
    }
    assert!(updates::github_release(&release("vbad", trusted)).is_err());
}
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static ID: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "skylab-update-test-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn config(&self, manifest: serde_json::Value) -> PathBuf {
        fs::write(
            self.0.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let p = self.0.join("updates.json");
        fs::write(&p, r#"{"manifest":"manifest.json"}"#).unwrap();
        p
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for name in ["manifest.json", "updates.json"] {
            let _ = fs::remove_file(self.0.join(name));
        }
        let _ = fs::remove_dir(&self.0);
    }
}
#[test]
fn missing_source_is_not_reported_as_up_to_date() {
    let f = Fixture::new();
    let result = updates::check_at(&f.0.join("updates.json")).unwrap();
    assert_eq!(result.status, "not_configured");
    assert_eq!(result.current_version, "0.1");
}
#[test]
fn configured_manifest_compares_numeric_versions() {
    let f = Fixture::new();
    for (v, status) in [
        ("0.1", "up_to_date"),
        ("0.1.0", "up_to_date"),
        ("0.0.9", "up_to_date"),
        ("0.10", "available"),
    ] {
        let result=updates::check_at(&f.config(serde_json::json!({"version":v,"download_url":"https://example.com/skylab.exe","notes":"Teste"}))).unwrap();
        assert_eq!(result.status, status);
    }
}
#[test]
fn rejects_unsafe_download_and_bad_versions() {
    let f = Fixture::new();
    for v in [
        serde_json::json!({"version":"0.2","download_url":"file:///C:/program.exe"}),
        serde_json::json!({"version":"0.2","download_url":"javascript:alert(1)"}),
        serde_json::json!({"version":"not-a-version"}),
    ] {
        assert!(updates::check_at(&f.config(v)).is_err());
    }
}
#[test]
fn remote_source_must_be_https() {
    let f = Fixture::new();
    let path = f.0.join("updates.json");
    fs::write(&path, r#"{"manifest":"http://example.com/manifest.json"}"#).unwrap();
    assert!(updates::check_at(&path).unwrap_err().contains("HTTPS"));
    for url in [
        "https://",
        "https://user:password@example.com/x",
        "https://example.com/\nfile",
    ] {
        assert!(!updates::safe_https(url));
    }
}
