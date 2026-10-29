use skylab_rust::files::Files;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "skylab-files-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn recents_reopen_disk_contents_preserve_order_and_never_cache_surveys() {
    let root = root();
    let path = root.join("flight.mis");
    let original = "L1\n500000 7800000\n500100 7800100\n500200 7800000";
    fs::write(&path, original).unwrap();
    let mut files = Files::at(root.clone());
    files.enqueue(std::slice::from_ref(&path), true).unwrap();
    let id = files.pending().unwrap().id;
    assert!(files.pending().unwrap().new_project);
    assert_eq!(files.read(id).unwrap()["text"], original);
    assert!(files.recent().is_empty());
    files.remember(id).unwrap();
    files.acknowledge(id);
    assert!(files.pending().is_none());
    let database = fs::read_to_string(root.join("recent.json")).unwrap();
    assert!(!database.contains("7800000"));
    assert!(!database.contains("L1\\n"));
    fs::write(&path, "L2\n500000 7800000\n500010 7800010").unwrap();
    let mut reopened = Files::at(root.clone());
    let id = reopened.recent()[0].id;
    assert!(
        reopened.read(id).unwrap()["text"]
            .as_str()
            .unwrap()
            .starts_with("L2")
    );
    fs::remove_file(&path).unwrap();
    assert!(reopened.read(id).unwrap_err().contains("unavailable"));
    assert_eq!(Files::at(root.clone()).recent().len(), 1);
    reopened.forget(id).unwrap();
    assert!(Files::at(root.clone()).recent().is_empty());
    for name in ["recent.json", "recent.new"] {
        let _ = fs::remove_file(root.join(name));
    }
    fs::remove_dir(root).unwrap();
}
#[test]
fn queue_rejects_invalid_inputs_and_recents_are_limited_to_twelve() {
    let root = root();
    let mut files = Files::at(root.clone());
    assert!(
        files
            .enqueue(&[PathBuf::from("relative.mis")], false)
            .is_err()
    );
    let wrong = root.join("script.exe");
    fs::write(&wrong, "MZ").unwrap();
    assert!(files.enqueue(std::slice::from_ref(&wrong), false).is_err());
    assert!(files.read(999).is_err());
    for i in 0..14 {
        let path = root.join(format!("{i}.mis"));
        fs::write(&path, "L1\n0 0\n1 1").unwrap();
        files.enqueue(std::slice::from_ref(&path), false).unwrap();
        let id = files.pending().unwrap().id;
        files.remember(id).unwrap();
        files.remember(id).unwrap();
        files.acknowledge(id);
        fs::remove_file(path).unwrap();
    }
    assert_eq!(files.recent().len(), 12);
    assert_eq!(files.recent()[0].name, "13.mis");
    assert_eq!(Files::at(root.clone()).recent().len(), 12);
    fs::remove_file(wrong).unwrap();
    fs::remove_file(root.join("recent.json")).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn skylab_projects_are_accepted_for_open_and_recent_files_without_copying_their_contents() {
    let root = root();
    let path = root.join("project.skylab");
    fs::write(&path, "{\"version\":6}").unwrap();
    let mut files = Files::at(root.clone());
    files.enqueue(std::slice::from_ref(&path), true).unwrap();
    let id = files.pending().unwrap().id;
    assert_eq!(files.read(id).unwrap()["name"], "project.skylab");
    files.remember(id).unwrap();
    assert_eq!(files.recent().len(), 1);
    fs::remove_file(&path).unwrap();
    files.forget(id).unwrap();
    fs::remove_file(root.join("recent.json")).unwrap();
    fs::remove_dir(root).unwrap();
}
