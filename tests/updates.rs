use skylab_rust::updates;
#[test]
fn github_detects_new_builds_without_changing_the_approved_version() {
    let fixture = |tag: &str| {
        serde_json::json!({"tag_name":tag,"assets":[{"name":"SkyTEM-SkyLab-Setup-0.1-r2.exe","state":"uploaded","browser_download_url":"https://github.com/JDantanna/SkyLab/releases/download/v0.1-r2/SkyTEM-SkyLab-Setup-0.1-r2.exe"}]}).to_string()
    };
    let revision = updates::build_revision();
    let check = updates::github_release(&fixture(&format!("v0.1-r{}", revision + 1))).unwrap();
    assert_eq!(check.status, "available");
    assert_eq!(check.latest_version.as_deref(), Some("0.1"));
    assert_eq!(check.latest_revision, Some(revision + 1));
    assert_eq!(check.current_revision, revision);
    assert_eq!(
        updates::github_release(&fixture(&format!("v0.1-r{revision}")))
            .unwrap()
            .status,
        "up_to_date"
    );
    assert_eq!(
        updates::github_release(&fixture("v0.1-r0")).unwrap().status,
        "up_to_date"
    );
    for bad in ["v0.1-r", "v0.1-r-1", "v0.1-rx", "v0.1-r4294967296"] {
        assert!(updates::github_release(&fixture(bad)).is_err());
    }
    let pending = updates::github_release(&format!(
        r#"{{"tag_name":"v0.1-r{}","assets":[]}}"#,
        revision + 1
    ))
    .unwrap();
    assert_eq!(pending.status, "release_pending");
    assert!(pending.download_url.is_none());
}
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

#[test]
fn internal_package_requires_a_trusted_uploaded_asset_and_sha256_digest() {
    let url =
        "https://github.com/JDantanna/SkyLab/releases/download/v0.2-r1/SkyLab-Update-0.2-r1.zip";
    let hash = "a".repeat(64);
    let mut release = serde_json::json!({"tag_name":"v0.2-r1","assets":[
        {"name":"SkyTEM-SkyLab-Setup-0.2-r1.exe","state":"uploaded","browser_download_url":"https://github.com/JDantanna/SkyLab/releases/download/v0.2-r1/setup.exe"},
        {"name":"SkyLab-Update-0.2-r1.zip","state":"uploaded","browser_download_url":url,"digest":format!("sha256:{hash}")}
    ]});
    let check = updates::github_release(&release.to_string()).unwrap();
    assert_eq!(check.package_url.as_deref(), Some(url));
    assert_eq!(check.package_sha256.as_deref(), Some(hash.as_str()));
    for digest in [
        serde_json::Value::Null,
        serde_json::json!("sha256:invalid"),
        serde_json::json!("md5:abc"),
    ] {
        release["assets"][1]["digest"] = digest;
        assert!(
            updates::github_release(&release.to_string())
                .unwrap()
                .package_sha256
                .is_none()
        );
    }
    release["assets"][1]["digest"] = serde_json::json!(format!("sha256:{hash}"));
    release["assets"][1]["browser_download_url"] =
        serde_json::json!("https://example.com/update.zip");
    assert!(
        updates::github_release(&release.to_string())
            .unwrap()
            .package_url
            .is_none()
    );
}
