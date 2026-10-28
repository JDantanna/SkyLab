#![cfg(windows)]
use skylab_rust::desktop;
use std::{fs, process::Command};
#[test]
fn native_updater_verifies_hash_and_rejects_zip_traversal() {
    let parent = std::env::temp_dir().join(format!("skylab-package-test-{}", std::process::id()));
    fs::create_dir_all(&parent).unwrap();
    for (index, evil, valid_hash) in [(0, false, true), (1, false, false), (2, true, true)] {
        let root = parent.join(index.to_string());
        fs::create_dir(&root).unwrap();
        let output = desktop::hidden(
            Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    include_str!("package-fixture.ps1"),
                ])
                .env("SKYLAB_PACKAGE_TEST_ROOT", &root)
                .env("SKYLAB_PACKAGE_TEST_EVIL", if evil { "1" } else { "0" }),
        )
        .output()
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let hash = String::from_utf8(output.stdout).unwrap();
        let hash = if valid_hash {
            hash.trim().to_owned()
        } else {
            "0".repeat(64)
        };
        let stage = root.join("stage");
        fs::create_dir(&stage).unwrap();
        let output = desktop::hidden(
            desktop::host_command()
                .unwrap()
                .arg("--unpack-update")
                .arg(root.join("package.zip"))
                .arg(hash)
                .arg(&stage),
        )
        .output()
        .unwrap();
        assert_eq!(
            output.status.success(),
            !evil && valid_hash,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(stage.join("SkyLab-Rust.exe").exists(), !evil && valid_hash);
        assert!(!root.join("SkyLab-Rust.exe").exists());
        for name in ["package.zip", "SkyLab-Rust.exe"] {
            let file = stage.join(name);
            if file.exists() {
                fs::remove_file(file).unwrap();
            }
        }
        fs::remove_dir(stage).unwrap();
        fs::remove_file(root.join("package.zip")).unwrap();
        fs::remove_dir(root).unwrap();
    }
    fs::remove_dir(parent).unwrap();
}
