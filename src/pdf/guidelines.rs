use lopdf::content::Operation;

/// Draw every card boundary through the page margins before card images are placed.
/// The images cover the lines, leaving continuous cutting guides only where paper is exposed.
pub(super) fn append(
    ops: &mut Vec<Operation>,
    page: [f32; 2],
    origin: [f32; 2],
    card: [f32; 2],
    gap: f32,
    faces: usize,
) {
    let [page_width, page_height] = page;
    let [left, top] = origin;
    let [width, height] = card;
    let columns = faces.min(3);
    let rows = faces.div_ceil(3);
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for column in 0..columns {
        let x = left + column as f32 * (width + gap);
        xs.extend([x, x + width]);
    }
    for row in 0..rows {
        let y = top - row as f32 * (height + gap);
        ys.extend([y, y - height]);
    }
    xs.dedup();
    ys.dedup();
    ops.push(Operation::new("w", vec![0.25.into()]));
    ops.push(Operation::new("G", vec![0.65.into()]));
    let points = |p: [f32; 2]| p.map(|v| (v * 72.0 / 25.4).into()).to_vec();
    for x in xs {
        ops.push(Operation::new("m", points([x, 0.0])));
        ops.push(Operation::new("l", points([x, page_height])));
        ops.push(Operation::new("S", vec![]));
    }
    for y in ys {
        ops.push(Operation::new("m", points([0.0, y])));
        ops.push(Operation::new("l", points([page_width, y])));
        ops.push(Operation::new("S", vec![]));
    }
}
