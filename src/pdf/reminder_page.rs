use lopdf::Object;
use lopdf::content::Operation;

pub(super) struct Reminder<'a> {
    pub paper: &'a str,
    pub deck_cards: usize,
    pub faces: usize,
    pub double_faced_cards: usize,
    pub proxy_pages: usize,
    pub card_width_mm: f32,
    pub card_height_mm: f32,
    pub scale: f32,
}

pub(super) fn operations(
    page_width_mm: f32,
    page_height_mm: f32,
    reminder: &Reminder<'_>,
) -> Vec<Operation> {
    let pt = |mm: f32| mm * 72.0 / 25.4;
    let mut ops = Vec::new();
    let mut text = |font: &str, size: f32, x: f32, y: f32, value: String| {
        ops.push(Operation::new("BT", vec![]));
        ops.push(Operation::new(
            "Tf",
            vec![Object::Name(font.as_bytes().to_vec()), size.into()],
        ));
        ops.push(Operation::new(
            "Tm",
            vec![
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                pt(x).into(),
                pt(y).into(),
            ],
        ));
        ops.push(Operation::new("Tj", vec![Object::string_literal(value)]));
        ops.push(Operation::new("ET", vec![]));
    };
    let left = 22.0;
    text(
        "F2",
        22.0,
        left,
        page_height_mm - 28.0,
        "Proxy printing instructions".into(),
    );
    text(
        "F1",
        11.0,
        left,
        page_height_mm - 39.0,
        "Exclude this first reminder page from printing.".into(),
    );
    text(
        "F2",
        14.0,
        left,
        page_height_mm - 57.0,
        "Deck summary".into(),
    );
    for (index, line) in [
        format!("Deck cards: {}", reminder.deck_cards),
        format!("Printable faces: {}", reminder.faces),
        format!("Double-faced cards: {}", reminder.double_faced_cards),
        format!("Proxy pages: {}", reminder.proxy_pages),
    ]
    .into_iter()
    .enumerate()
    {
        text(
            "F1",
            11.0,
            left,
            page_height_mm - 67.0 - index as f32 * 7.0,
            line,
        );
    }
    text(
        "F2",
        14.0,
        left,
        page_height_mm - 103.0,
        "Required print settings".into(),
    );
    for (index, line) in [
        format!("- Paper size: {}", reminder.paper),
        "- Scaling: 100% or Actual size".into(),
        "- Disable Fit, Shrink, and Scale to printable area".into(),
        "- Margins: none; do not add document margins".into(),
        "- Print pages 2 onward; exclude this reminder page".into(),
    ]
    .into_iter()
    .enumerate()
    {
        text(
            "F1",
            11.0,
            left,
            page_height_mm - 113.0 - index as f32 * 8.0,
            line,
        );
    }
    text(
        "F2",
        14.0,
        left,
        page_height_mm - 163.0,
        "Measurement check".into(),
    );
    text(
        "F1",
        11.0,
        left,
        page_height_mm - 174.0,
        format!(
            "At scale {:.3}, each printed cutout must measure {:.2} x {:.2} mm.",
            reminder.scale, reminder.card_width_mm, reminder.card_height_mm
        ),
    );
    text(
        "F1",
        10.0,
        left,
        18.0,
        format!("Page size: {:.1} x {:.1} mm", page_width_mm, page_height_mm),
    );
    ops
}
