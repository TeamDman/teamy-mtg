use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;

struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(handler: impl Fn(&str, &str) -> (u16, &'static str, Vec<u8>) + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let base = url.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                let Ok((mut stream, _)) = listener.accept() else {
                    thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                };
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buf = [0; 2048];
                while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = stream.read(&mut buf).unwrap();
                    if n == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buf[..n]);
                }
                let request = String::from_utf8(bytes).unwrap();
                assert!(request.to_lowercase().contains("user-agent: teamy-mtg/"));
                assert!(request.to_lowercase().contains("accept:"));
                let path = request.split_whitespace().nth(1).unwrap();
                let (status, content_type, body) = handler(path, &base);
                write!(stream, "HTTP/1.1 {status} Response\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(&body).unwrap();
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}

fn cli(dir: &Path, url: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_teamy-mtg"))
        .arg("--data-dir")
        .arg(dir)
        .args(["--api-url", url, "--json"])
        .args(args)
        .output()
        .unwrap()
}
fn ok<T: facet::Facet<'static>>(dir: &Path, url: &str, args: &[&str]) -> T {
    let output = cli(dir, url, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    facet_json::from_slice(&output.stdout).unwrap()
}
fn card(base: &str, id: &str) -> teamy_mtg::model::Card {
    teamy_mtg::model::Card {
        id: id.into(),
        oracle_id: id.into(),
        name: if id == "a" {
            "Zulaport Cutthroat".into()
        } else {
            "Forest".into()
        },
        image_uris: [("png".into(), format!("{base}/image.png"))].into(),
        ..Default::default()
    }
}

fn assert_pdf_grid(path: &Path, gap_mm: f32, scale: f32, guidelines: bool) {
    let doc = lopdf::Document::load(path).unwrap();
    let page = *doc.get_pages().values().nth(1).unwrap();
    let content = lopdf::content::Content::decode(&doc.get_page_content(page).unwrap()).unwrap();
    let placements: Vec<_> = content
        .operations
        .iter()
        .filter(|op| op.operator == "cm")
        .collect();
    assert_eq!(placements.len(), 9);
    let x = |i: usize| placements[i].operands[4].as_float().unwrap();
    let y = |i: usize| placements[i].operands[5].as_float().unwrap();
    let mm = |v: f32| v * 72.0 / 25.4;
    assert!((x(1) - x(0) - mm(63.0 * scale + gap_mm)).abs() < 0.001);
    assert!((y(0) - y(3) - mm(88.0 * scale + gap_mm)).abs() < 0.001);
    let cut_mark_lines = if gap_mm == 0.0 { 16 } else { 24 };
    let guideline_lines = if guidelines {
        if gap_mm == 0.0 { 8 } else { 12 }
    } else {
        0
    };
    assert_eq!(
        content
            .operations
            .iter()
            .filter(|op| op.operator == "S")
            .count(),
        cut_mark_lines + guideline_lines
    );
    // Every cut mark stays outside the entire card grid, including shared seams.
    for op in content
        .operations
        .iter()
        .filter(|op| op.operator == "m" || op.operator == "l")
    {
        let px = op.operands[0].as_float().unwrap();
        let py = op.operands[1].as_float().unwrap();
        assert!(
            px <= x(0) + 0.001
                || px >= x(2) + mm(63.0 * scale) - 0.001
                || py <= y(6) + 0.001
                || py >= y(0) + mm(88.0 * scale) - 0.001
        );
    }
}
#[test]
fn preplan_workflow_json_decks_image_sync_and_offline_pdf() {
    use std::collections::BTreeMap;
    use teamy_mtg::decks::Deck;
    use teamy_mtg::decks::DeckEntry;
    use teamy_mtg::decks::DeckSummary;
    use teamy_mtg::model::Card;
    use teamy_mtg::pdf::PdfSummary;
    let server = Server::new(|path, base| {
        let body = match path {
            "/bulk-data/oracle-cards" => {
                format!(r#"{{"updated_at":"fixture","jsonl_download_uri":"{base}/bulk.gz"}}"#)
                    .into_bytes()
            }
            "/bulk.gz" => {
                let mut gz =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                writeln!(gz, "{}", facet_json::to_string(&card(base, "a")).unwrap()).unwrap();
                writeln!(gz, "{}", facet_json::to_string(&card(base, "b")).unwrap()).unwrap();
                return (200, "application/gzip", gz.finish().unwrap());
            }
            "/image.png" => {
                let img = image::RgbImage::from_fn(63, 88, |x, y| {
                    image::Rgb([x as u8 * 3, y as u8 * 2, 100])
                });
                let mut bytes = std::io::Cursor::new(Vec::new());
                img.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
                return (200, "image/png", bytes.into_inner());
            }
            _ => panic!("unexpected route {path}; search must never use network"),
        };
        (200, "application/json", body)
    });
    let dir = tempfile::tempdir().unwrap();
    let base = &server.url;
    let output = cli(dir.path(), base, &["db", "update"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        ok::<Vec<Card>>(dir.path(), base, &["card", "list"]).len(),
        2
    );
    assert_eq!(
        ok::<Vec<Card>>(dir.path(), base, &["card", "search", "zulaport"]).len(),
        1
    );
    ok::<Deck>(dir.path(), base, &["deck", "create", "my-deck-1"]);
    ok::<Deck>(
        dir.path(),
        base,
        &[
            "deck",
            "card",
            "add",
            "--deck",
            "my-deck-1",
            "Zulaport Cutthroat",
            "--quantity",
            "10",
        ],
    );
    assert_eq!(
        ok::<Vec<DeckEntry>>(
            dir.path(),
            base,
            &["deck", "card", "list", "--deck", "my-deck-1"]
        )[0]
        .quantity,
        10
    );
    assert_eq!(
        ok::<Vec<DeckSummary>>(dir.path(), base, &["deck", "list"])[0].total_cards,
        10
    );
    let missing = ok::<BTreeMap<String, String>>(
        dir.path(),
        base,
        &["card", "image", "list", "--deck", "my-deck-1"],
    );
    assert!(missing.values().all(|p| p == "missing"));
    let paths = ok::<BTreeMap<String, String>>(
        dir.path(),
        base,
        &["card", "image", "sync", "--deck", "my-deck-1"],
    );
    assert!(
        paths
            .values()
            .all(|p| Path::new(p).is_absolute() && Path::new(p).is_file())
    );
    let pdf = dir.path().join("a.pdf");
    let args = [
        "proxy-pdf",
        "generate",
        "--deck",
        "my-deck-1",
        pdf.to_str().unwrap(),
        "--offline",
    ];
    let summary = ok::<PdfSummary>(dir.path(), base, &args);
    assert_eq!(summary.pages, 3);
    assert_eq!(summary.proxy_pages, 2);
    assert_eq!(summary.faces, 10);
    assert_eq!(summary.double_faced_cards, 0);
    assert_pdf_grid(&pdf, 0.0, 1.0, true);
    let doc = lopdf::Document::load(&pdf).unwrap();
    assert_eq!(doc.get_pages().len(), 3);
    let catalog = doc
        .trailer
        .get(b"Root")
        .unwrap()
        .as_reference()
        .and_then(|id| doc.get_object(id))
        .unwrap()
        .as_dict()
        .unwrap();
    let viewer_preferences = catalog
        .get(b"ViewerPreferences")
        .unwrap()
        .as_dict()
        .unwrap();
    assert_eq!(
        viewer_preferences
            .get(b"PrintScaling")
            .unwrap()
            .as_name()
            .unwrap(),
        b"None"
    );
    let reminder = *doc.get_pages().values().next().unwrap();
    let reminder_content =
        lopdf::content::Content::decode(&doc.get_page_content(reminder).unwrap()).unwrap();
    let reminder_text = reminder_content
        .operations
        .iter()
        .filter(|op| op.operator == "Tj")
        .map(|op| String::from_utf8_lossy(op.operands[0].as_str().unwrap()))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(reminder_text.contains("Double-faced cards: 0"));
    assert!(reminder_text.contains("exclude this reminder page"));
    let page = doc
        .get_object(*doc.get_pages().values().nth(1).unwrap())
        .unwrap()
        .as_dict()
        .unwrap();
    let media = page.get(b"MediaBox").unwrap().as_array().unwrap();
    assert!((media[2].as_float().unwrap() - 595.2756).abs() < 0.01);
    let contents = doc
        .get_page_content(*doc.get_pages().values().nth(1).unwrap())
        .unwrap();
    let content = lopdf::content::Content::decode(&contents).unwrap();
    let placement = content
        .operations
        .iter()
        .find(|o| o.operator == "cm")
        .unwrap();
    assert!((placement.operands[0].as_float().unwrap() - 63.0 * 72.0 / 25.4).abs() < 0.01);
    assert!((placement.operands[3].as_float().unwrap() - 88.0 * 72.0 / 25.4).abs() < 0.01);
    let previous = std::fs::read(&pdf).unwrap();
    assert!(!cli(dir.path(), base, &args).status.success());
    assert_eq!(std::fs::read(&pdf).unwrap(), previous);
    let image_path = paths.values().next().unwrap();
    std::fs::write(image_path, b"damaged image").unwrap();
    assert!(
        ok::<BTreeMap<String, String>>(
            dir.path(),
            base,
            &["card", "image", "list", "--deck", "my-deck-1"]
        )
        .values()
        .all(|p| p == "missing")
    );
    let mut replace_args = args.to_vec();
    replace_args.push("--force");
    assert!(!cli(dir.path(), base, &replace_args).status.success());
    assert_eq!(std::fs::read(&pdf).unwrap(), previous);
    assert_eq!(
        ok::<BTreeMap<String, String>>(
            dir.path(),
            base,
            &["card", "image", "sync", "--deck", "my-deck-1"]
        ),
        paths
    );
    drop(server);
    let offline = "http://127.0.0.1:1";
    // Sync a second time after shutting down the server: all images must be reused.
    assert_eq!(
        ok::<BTreeMap<String, String>>(
            dir.path(),
            offline,
            &["card", "image", "sync", "--deck", "my-deck-1"]
        ),
        paths
    );
    let mut letter_args = args.to_vec();
    letter_args.extend(["--force", "--paper", "letter"]);
    assert_eq!(
        ok::<PdfSummary>(dir.path(), offline, &letter_args).faces,
        10
    );
    assert_pdf_grid(&pdf, 0.0, 1.0, true);
    letter_args.extend(["--gap", "1.5"]);
    assert_eq!(
        ok::<PdfSummary>(dir.path(), offline, &letter_args).faces,
        10
    );
    assert_pdf_grid(&pdf, 1.5, 1.0, true);
    let spaced_pdf = std::fs::read(&pdf).unwrap();
    for gap in ["-1", "NaN", "inf", "100"] {
        *letter_args.last_mut().unwrap() = gap;
        assert!(
            !cli(dir.path(), offline, &letter_args).status.success(),
            "gap {gap} must fail"
        );
        assert_eq!(std::fs::read(&pdf).unwrap(), spaced_pdf);
    }
    let mut scaled_args = args.to_vec();
    scaled_args.extend(["--force", "--scale", "1.1"]);
    let scaled = ok::<PdfSummary>(dir.path(), offline, &scaled_args);
    assert_eq!(scaled.scale, 1.1);
    assert!((scaled.card_width_mm - 69.3).abs() < 0.001);
    assert_pdf_grid(&pdf, 0.0, 1.1, true);
    let mut no_guidelines_args = args.to_vec();
    no_guidelines_args.extend(["--force", "--no-guidelines"]);
    ok::<PdfSummary>(dir.path(), offline, &no_guidelines_args);
    assert_pdf_grid(&pdf, 0.0, 1.0, false);
    let unscaled_pdf = std::fs::read(&pdf).unwrap();
    for scale in ["0", "-1", "NaN", "inf", "2"] {
        let mut invalid_scale_args = args.to_vec();
        invalid_scale_args.extend(["--force", "--scale", scale]);
        assert!(
            !cli(dir.path(), offline, &invalid_scale_args)
                .status
                .success(),
            "scale {scale} must fail"
        );
        assert_eq!(std::fs::read(&pdf).unwrap(), unscaled_pdf);
    }
    assert_eq!(
        ok::<Vec<Card>>(dir.path(), offline, &["card", "search", "zulaport"])[0].name,
        "Zulaport Cutthroat"
    );
    assert!(
        ok::<Vec<Card>>(
            dir.path(),
            offline,
            &["card", "search", "id:g is:commander"]
        )
        .is_empty()
    );
    assert!(
        !cli(
            dir.path(),
            offline,
            &["card", "search", "unsupported:thing"]
        )
        .status
        .success()
    );
    assert!(
        !cli(
            dir.path(),
            offline,
            &[
                "deck",
                "card",
                "add",
                "--deck",
                "my-deck-1",
                "Forest",
                "--quantity",
                "0"
            ]
        )
        .status
        .success()
    );
    ok::<Deck>(
        dir.path(),
        offline,
        &[
            "deck",
            "card",
            "remove",
            "--deck",
            "my-deck-1",
            "Zulaport Cutthroat",
            "--quantity",
            "9",
        ],
    );
    assert_eq!(
        ok::<Vec<DeckSummary>>(dir.path(), offline, &["deck", "list"])[0].total_cards,
        1
    );
    // Direct JSON editing works without involving the CLI.
    let path = dir.path().join("decks/my-deck-1.json");
    let mut deck: Deck = facet_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    deck.cards[0].quantity = 4;
    std::fs::write(&path, facet_json::to_string_pretty(&deck).unwrap()).unwrap();
    assert_eq!(
        ok::<Vec<DeckSummary>>(dir.path(), offline, &["deck", "list"])[0].total_cards,
        4
    );
    let cache = dir.path().join("images");
    assert_eq!(
        ok::<BTreeMap<String, String>>(dir.path(), offline, &["cache", "show"])["path"],
        cache.display().to_string()
    );
    std::fs::write(cache.join("keep.txt"), "not a cached image").unwrap();
    assert_eq!(
        ok::<teamy_mtg::paths::CleanResult>(dir.path(), offline, &["cache", "clean"])
            .entries_removed,
        1
    );
    assert!(cache.join("keep.txt").exists());
    assert!(path.exists());
}
