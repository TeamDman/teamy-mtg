use facet::Facet;

#[derive(Debug, Facet)]
pub struct PdfSummary {
    pub output: String,
    pub pages: usize,
    pub proxy_pages: usize,
    pub faces: usize,
    pub double_faced_cards: usize,
    pub card_width_mm: f32,
    pub card_height_mm: f32,
    pub scale: f32,
}
