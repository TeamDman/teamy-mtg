use facet::Facet;

#[derive(Debug, Facet)]
pub struct PdfSummary {
    pub output: String,
    pub pages: usize,
    pub faces: usize,
    pub card_width_mm: u32,
    pub card_height_mm: u32,
}
