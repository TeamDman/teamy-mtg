use lopdf::content::Operation;

/// Put marks outside the grid so shared edges never draw over adjacent artwork.
pub(super) fn append(
    ops: &mut Vec<Operation>,
    origin: [f32; 2],
    card: [f32; 2],
    gap: f32,
    faces: usize,
) {
    let [left, top] = origin;
    let [width, height] = card;
    let columns = faces.min(3);
    let rows = faces.div_ceil(3);
    let right = left + columns as f32 * width + (columns - 1) as f32 * gap;
    let bottom = top - rows as f32 * height - (rows - 1) as f32 * gap;
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
    // At zero gap, each shared edge needs just one mark at each end.
    xs.dedup();
    ys.dedup();
    ops.push(Operation::new("w", vec![0.25.into()]));
    ops.push(Operation::new("G", vec![0.5.into()]));
    let mut line = |start: [f32; 2], end: [f32; 2]| {
        let points = |p: [f32; 2]| p.map(|v| (v * 72.0 / 25.4).into()).to_vec();
        ops.push(Operation::new("m", points(start)));
        ops.push(Operation::new("l", points(end)));
        ops.push(Operation::new("S", vec![]));
    };
    for x in xs {
        line([x, top], [x, top + 0.8]);
        line([x, bottom], [x, bottom - 0.8]);
    }
    for y in ys {
        line([left, y], [left - 0.8, y]);
        line([right, y], [right + 0.8, y]);
    }
}
