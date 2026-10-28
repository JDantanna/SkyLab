#![cfg(windows)]
use skylab_rust::{Crs, desktop, import_kml, kmz};
use std::process::Command;
#[test]
fn kmz_import_reads_doc_kml_without_extracting_files_and_rejects_ambiguous_or_invalid_archives() {
    for ambiguous in [false, true] {
        let result = desktop::hidden(
            Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    include_str!("kmz-fixture.ps1"),
                ])
                .env("SKYLAB_KMZ_AMBIGUOUS", if ambiguous { "1" } else { "0" }),
        )
        .output()
        .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let base64 = String::from_utf8(result.stdout).unwrap();
        let kml = kmz::read(base64.trim());
        if ambiguous {
            assert!(kml.is_err());
        } else {
            let p = import_kml(
                &kml.unwrap(),
                Crs {
                    zone: 23,
                    south: true,
                    authority: None,
                },
            )
            .unwrap();
            assert_eq!(p.lines.len(), 1);
            assert_eq!(p.lines[0].name, "L1");
        }
    }
    assert!(kmz::read("invalid base64").is_err());
    assert!(kmz::read("YWJj").is_err());
}
