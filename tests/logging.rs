use facet::Facet;
use std::process::Command;

#[derive(Facet, Debug)]
struct ImportReport {
    cards_imported: usize,
}

#[test]
fn import_progress_respects_filters_and_reaches_the_log_file() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cards.jsonl");
    std::fs::write(
        &input,
        "{\"id\":\"fixture\",\"name\":\"Fixture Card\"}\n".repeat(10000),
    )
    .unwrap();

    for filter in ["info", "error"] {
        let log = dir.path().join(format!("{filter}.ndjson"));
        let output = Command::new(env!("CARGO_BIN_EXE_teamy-mtg"))
            .arg("--data-dir")
            .arg(dir.path().join(filter))
            .args(["--json", "--log-filter", filter, "--log-file"])
            .arg(&log)
            .args(["db", "update", "--from"])
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: ImportReport = facet_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report.cards_imported, 10000);
        let stderr = String::from_utf8(output.stderr).unwrap();
        let logged = std::fs::read_to_string(log).unwrap();
        assert_eq!(stderr.contains("Imported cards"), filter == "info");
        assert_eq!(logged.contains("Imported cards"), filter == "info");
        if filter == "info" {
            assert!(logged.contains("\"count\":10000"), "{logged}");
        }
    }
}
