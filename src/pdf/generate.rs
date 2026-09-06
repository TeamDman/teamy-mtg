use super::Faces;
use super::Options;
use super::Paper;
use super::PdfSummary;
use crate::model::DeckCard;
use crate::network::Scryfall;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::ensure;
use lopdf::Document;
use lopdf::Object;
use lopdf::Stream;
use lopdf::content::Content;
use lopdf::content::Operation;
use lopdf::dictionary;
use std::collections::HashMap;
use std::path::Path;

pub fn generate(deck: &[DeckCard], options: Options<'_>, api: &mut Scryfall) -> Result<PdfSummary> {
    let (pw, ph) = match options.paper {
        Paper::A4 => (210.0, 297.0),
        Paper::Letter => (215.9, 279.4),
    };
    let (cw, ch, gap) = (63.0, 88.0, options.gap);
    ensure!(
        gap.is_finite() && gap >= 0.0,
        "Gap must be a finite, non-negative number of millimetres"
    );
    ensure!(
        3.0 * cw + 2.0 * gap + 1.6 <= pw && 3.0 * ch + 2.0 * gap + 1.6 <= ph,
        "Gap is too large to fit nine cards and cut marks on the selected paper"
    );
    ensure!(
        !deck.is_empty(),
        "Deck is empty; add a card before generating proxies"
    );
    ensure!(
        options.force || !options.output.exists(),
        "Output {} already exists; use --force to replace it",
        options.output.display()
    );
    let mut slots = Vec::new();
    for item in deck {
        let images = item
            .card
            .images()
            .into_iter()
            .map(|(name, url)| Ok((name, url.with_context(|| format!("No image for '{name}'"))?)))
            .collect::<eyre::Result<Vec<_>>>()?;
        ensure!(
            !images.is_empty(),
            "No printable image for '{}'",
            item.card.name
        );
        if item.card.image_uris.is_empty() {
            ensure!(
                images.len() == item.card.card_faces.len(),
                "Missing face image for '{}'",
                item.card.name
            );
        }
        let selected: Vec<_> = match options.faces {
            Faces::All => images,
            Faces::Front => images.into_iter().take(1).collect(),
            Faces::Back => images.into_iter().skip(1).collect(),
        };
        for _ in 0..item.quantity {
            slots.extend(selected.iter().copied());
        }
        ensure!(
            slots.len() <= 10000,
            "PDF is limited to 10000 card faces; reduce deck quantities"
        );
    }
    ensure!(!slots.is_empty(), "No back faces in this deck");
    let parent = options
        .output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    ensure!(
        parent.is_dir(),
        "Output directory does not exist: {}",
        parent.display()
    );
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut image_ids = HashMap::new();
    for (name, url) in &slots {
        api.check_cancelled()?;
        if image_ids.contains_key(*url) {
            continue;
        }
        tracing::info!(card = name, "Preparing card image for PDF");
        let image = crate::images::load_image(url, options.cache, options.offline, api)
            .with_context(|| format!("Cannot prepare '{name}'"))?;
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95).encode_image(&image)?;
        let id = doc.add_object(Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image", "Width" => image.width(),
                "Height" => image.height(), "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8,
                "Filter" => "DCTDecode",
            },
            jpeg,
        ));
        image_ids.insert(*url, id);
    }
    let mm = |v: f32| v * 72.0 / 25.4;
    let (left, top) = (
        (pw - 3.0 * cw - 2.0 * gap) / 2.0,
        (ph - 3.0 * ch - 2.0 * gap) / 2.0,
    );
    let mut page_ids = Vec::new();
    for page_slots in slots.chunks(9) {
        api.check_cancelled()?;
        let mut objects = lopdf::Dictionary::new();
        let mut ops = Vec::new();
        for (index, (_, url)) in page_slots.iter().enumerate() {
            let name = format!("Im{index}");
            objects.set(name.clone(), image_ids[url]);
            let x = left + (index % 3) as f32 * (cw + gap);
            let y = ph - top - ch - (index / 3) as f32 * (ch + gap);
            ops.push(Operation::new("q", vec![]));
            ops.push(Operation::new(
                "cm",
                vec![
                    mm(cw).into(),
                    0.into(),
                    0.into(),
                    mm(ch).into(),
                    mm(x).into(),
                    mm(y).into(),
                ],
            ));
            ops.push(Operation::new("Do", vec![Object::Name(name.into_bytes())]));
            ops.push(Operation::new("Q", vec![]));
        }
        super::cut_marks::append(&mut ops, [left, ph - top], [cw, ch], gap, page_slots.len());
        let contents = doc.add_object(Stream::new(
            dictionary! {},
            Content { operations: ops }.encode()?,
        ));
        page_ids.push(doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), mm(pw).into(), mm(ph).into()],
            "Resources" => dictionary! { "XObject" => objects }, "Contents" => contents,
        }));
    }
    doc.objects.insert(pages_id, dictionary! {
        "Type" => "Pages", "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
        "Count" => page_ids.len() as i64,
    }.into());
    let root = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    let info = doc.add_object(dictionary! { "Producer" => Object::string_literal("teamy-mtg"), "Title" => Object::string_literal("Proxy cards - print at 100% scale") });
    doc.trailer.set("Root", root);
    doc.trailer.set("Info", info);
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    doc.save_to(&mut staged)?;
    staged.as_file().sync_all()?;
    if options.force {
        staged.persist(options.output)?;
    } else {
        staged.persist_noclobber(options.output)?;
    }
    Ok(PdfSummary {
        output: options.output.display().to_string(),
        pages: page_ids.len(),
        faces: slots.len(),
        card_width_mm: 63,
        card_height_mm: 88,
    })
}
