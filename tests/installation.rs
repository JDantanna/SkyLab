#![cfg(windows)]
use skylab_rust::installation::{self, APP, Options, UNINSTALL};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static ID: AtomicUsize = AtomicUsize::new(0);
fn temp() -> (PathBuf, Options) {
    let parent = std::env::temp_dir().join(format!(
        "skylab-install-test-{}-{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&parent).unwrap();
    let o = Options::test(parent.join("SkyLab"));
    (parent, o)
}
#[test]
fn install_upgrade_uninstall_preserves_user_files_and_shortcut_targets() {
    let (parent, o) = temp();
    let app = b"MZ-test-app-v1";
    let setup = b"MZ-test-setup";
    installation::install(&o, app, setup).unwrap();
    assert!(o.installed());
    assert_eq!(fs::read(o.root.join(APP)).unwrap(), app);
    assert_eq!(fs::read(o.root.join(UNINSTALL)).unwrap(), setup);
    assert!(o.root.join("test-shortcuts/SkyLab.lnk").is_file());
    assert!(o.root.join("test-shortcuts/SkyTEM SkyLab.lnk").is_file());
    let metadata: serde_json::Value = serde_json::from_str(
        fs::read_to_string(o.root.join("test-registration.json"))
            .unwrap()
            .trim_start_matches('\u{feff}'),
    )
    .unwrap();
    assert_eq!(
        fs::canonicalize(metadata["InstallLocation"].as_str().unwrap()).unwrap(),
        fs::canonicalize(&o.root).unwrap()
    );
    let shortcut=skylab_rust::desktop::hidden(std::process::Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","$s=New-Object -ComObject WScript.Shell; ($s.CreateShortcut((Join-Path $env:SKYLAB_TEST_ROOT 'test-shortcuts/SkyLab.lnk'))).TargetPath"]).env("SKYLAB_TEST_ROOT",&o.root)).output().unwrap();
    assert!(shortcut.status.success());
    assert_eq!(
        fs::canonicalize(String::from_utf8_lossy(&shortcut.stdout).trim()).unwrap(),
        fs::canonicalize(o.root.join(APP)).unwrap()
    );
    fs::write(o.root.join("personal-project.json"), "keep").unwrap();
    installation::install(&o, b"MZ-test-app-v2", setup).unwrap();
    assert_eq!(fs::read(o.root.join(APP)).unwrap(), b"MZ-test-app-v2");
    assert_eq!(
        fs::read(o.root.join("SkyLab-Rust.previous.exe")).unwrap(),
        app
    );
    installation::uninstall(&o).unwrap();
    assert!(!o.installed());
    assert!(!o.root.join(APP).exists());
    assert!(!o.root.join(UNINSTALL).exists());
    assert!(!o.root.join("test-registration.json").exists());
    assert!(!o.root.join("test-shortcuts").exists());
    assert_eq!(
        fs::read_to_string(o.root.join("personal-project.json")).unwrap(),
        "keep"
    );
    fs::remove_file(o.root.join("personal-project.json")).unwrap();
    fs::remove_dir(o.root).unwrap();
    fs::remove_dir(parent).unwrap();
}
#[test]
fn installer_refuses_unknown_directory() {
    let (parent, o) = temp();
    fs::create_dir(&o.root).unwrap();
    fs::write(o.root.join("keep.txt"), "owned by user").unwrap();
    assert!(installation::install(&o, b"MZ-test", b"MZ-test").is_err());
    assert!(installation::uninstall(&o).is_err());
    assert_eq!(
        fs::read_to_string(o.root.join("keep.txt")).unwrap(),
        "owned by user"
    );
    fs::remove_file(o.root.join("keep.txt")).unwrap();
    fs::remove_dir(o.root).unwrap();
    fs::remove_dir(parent).unwrap();
}
#[test]
fn invalid_payload_does_not_create_installation() {
    let (parent, o) = temp();
    assert!(installation::install(&o, b"invalid", b"MZ-test").is_err());
    assert!(!o.root.exists());
    assert!(installation::install(&o, b"MZ-test", b"invalid").is_err());
    assert!(!o.root.exists());
    fs::remove_dir(parent).unwrap();
}

#[test]
fn internal_update_waits_for_a_transient_image_file_lock_without_killing_processes() {
    use std::os::windows::fs::OpenOptionsExt;
    let (parent, o) = temp();
    installation::install(&o, b"MZ-old", b"MZ-setup").unwrap();
    let source = parent.join("update.exe");
    fs::write(&source, b"MZ-new").unwrap();
    let guard = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(o.root.join(APP))
        .unwrap();
    let started = std::time::Instant::now();
    let unlock = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(600));
        drop(guard);
    });
    installation::replace_application(&o, &source).unwrap();
    assert!(started.elapsed() >= std::time::Duration::from_millis(300));
    unlock.join().unwrap();
    assert_eq!(fs::read(o.root.join(APP)).unwrap(), b"MZ-new");
    installation::uninstall(&o).unwrap();
    fs::remove_file(source).unwrap();
    fs::remove_dir(parent).unwrap();
}

#[test]
fn internal_update_preserves_uninstaller_shortcuts_and_personal_files_and_rolls_back_failure() {
    let (parent, o) = temp();
    installation::install(&o, b"MZ-original", b"MZ-setup").unwrap();
    let shortcut = fs::read(o.root.join("test-shortcuts/SkyLab.lnk")).unwrap();
    fs::write(o.root.join("personal.json"), "keep").unwrap();
    let source = parent.join("update.exe");
    fs::write(&source, b"MZ-updated").unwrap();
    installation::replace_application(&o, &source).unwrap();
    assert_eq!(fs::read(o.root.join(APP)).unwrap(), b"MZ-updated");
    assert_eq!(fs::read(o.root.join(UNINSTALL)).unwrap(), b"MZ-setup");
    assert_eq!(
        fs::read(o.root.join("test-shortcuts/SkyLab.lnk")).unwrap(),
        shortcut
    );
    assert_eq!(
        fs::read_to_string(o.root.join("personal.json")).unwrap(),
        "keep"
    );
    let manifest = fs::read(o.root.join("installation.json")).unwrap();
    let registration = fs::read(o.root.join("test-registration.json")).unwrap();
    fs::write(o.root.join("test-registration.json"), "invalid JSON").unwrap();
    fs::write(&source, b"MZ-next").unwrap();
    assert!(
        installation::replace_application(&o, &source)
            .unwrap_err()
            .contains("restaurada")
    );
    assert_eq!(fs::read(o.root.join(APP)).unwrap(), b"MZ-updated");
    assert_eq!(
        fs::read(o.root.join("installation.json")).unwrap(),
        manifest
    );
    fs::write(o.root.join("test-registration.json"), registration).unwrap();
    installation::uninstall(&o).unwrap();
    fs::remove_file(o.root.join("personal.json")).unwrap();
    fs::remove_dir(&o.root).unwrap();
    fs::remove_file(source).unwrap();
    fs::remove_dir(parent).unwrap();
}
#[test]
fn install_progress_finishes_only_after_files_and_shortcuts_exist() {
    let (parent, o) = temp();
    let percents = std::cell::RefCell::new(Vec::new());
    installation::install_with_progress(&o, b"MZ-test-app", b"MZ-test-setup", |_, percent| {
        percents.borrow_mut().push(percent);
        if percent == 100 {
            assert!(o.root.join(APP).is_file());
            assert!(o.root.join("test-registration.json").is_file());
            assert!(o.root.join("test-shortcuts/SkyLab.lnk").is_file());
        }
    })
    .unwrap();
    let steps = percents.borrow();
    assert_eq!(steps.last(), Some(&100));
    assert!(steps.windows(2).all(|s| s[0] <= s[1]));
    drop(steps);
    installation::uninstall(&o).unwrap();
    fs::remove_dir(parent).unwrap();
}

#[test]
fn running_application_blocks_update_and_uninstall_before_registration_changes() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    struct Guard(std::process::Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let (parent, o) = temp();
    let app = fs::read(env!("CARGO_BIN_EXE_skylab-rust")).unwrap();
    installation::install(&o, &app, b"MZ-test-setup").unwrap();
    let mut child = Guard(
        Command::new(o.root.join(APP))
            .args(["--no-open", "--port", "0"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut reader = BufReader::new(child.0.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if line.starts_with("Abra http://") {
            break;
        }
    }
    let before = fs::read(o.root.join("test-registration.json")).unwrap();
    assert!(
        installation::install(&o, &app, b"MZ-test-setup")
            .unwrap_err()
            .contains("Feche")
    );
    assert!(installation::uninstall(&o).unwrap_err().contains("Feche"));
    assert_eq!(
        before,
        fs::read(o.root.join("test-registration.json")).unwrap()
    );
    assert!(o.installed());
    drop(child);
    installation::uninstall(&o).unwrap();
    assert!(!o.root.exists());
    fs::remove_dir(parent).unwrap();
}

#[test]
fn legacy_update_migrates_mis_registration_and_uninstaller_without_touching_user_data() {
    let (parent, o) = temp();
    installation::install(&o, b"MZ-original", b"MZ-legacy-setup").unwrap();
    let path = o.root.join("installation.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["version"] = serde_json::json!("0.1.0");
    fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    fs::write(o.root.join("flight.mis"), "personal survey").unwrap();
    let source = parent.join("new.exe");
    fs::write(&source, b"MZ-new-app-with-uninstall-mode").unwrap();
    installation::replace_application(&o, &source).unwrap();
    assert_eq!(
        fs::read(o.root.join(UNINSTALL)).unwrap(),
        fs::read(&source).unwrap()
    );
    assert!(!o.root.join("Uninstall.previous.exe").exists());
    let association: serde_json::Value =
        serde_json::from_slice(&fs::read(o.root.join("test-association.json")).unwrap()).unwrap();
    assert_eq!(association["Extension"], ".mis");
    assert_eq!(association["ProjectExtension"], ".skylab");
    assert_eq!(association["ProjectProgId"], "SkyTEM.SkyLab.Project");
    assert!(
        association["ProjectIcon"]
            .as_str()
            .unwrap()
            .ends_with("SkyLab-Rust.exe\",0")
    );
    assert!(
        association["Command"]
            .as_str()
            .unwrap()
            .ends_with(" --open \"%1\"")
    );
    installation::uninstall(&o).unwrap();
    assert!(!o.root.join("test-association.json").exists());
    assert_eq!(
        fs::read_to_string(o.root.join("flight.mis")).unwrap(),
        "personal survey"
    );
    fs::remove_file(o.root.join("flight.mis")).unwrap();
    fs::remove_dir(&o.root).unwrap();
    fs::remove_file(source).unwrap();
    fs::remove_dir(parent).unwrap();
}
