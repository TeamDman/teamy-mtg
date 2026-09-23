use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Generate card cutouts in a 3 x 3 grid. Print at 100% scale and exclude page 1.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct ProxyPdfGenerateArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
    /// Output PDF path.
    #[facet(args::positional)]
    pub output: String,
    /// Paper size: a4 or letter (default a4).
    #[facet(args::named)]
    pub paper: Option<crate::pdf::Paper>,
    /// all, front, or back (default all, separate cutouts, not duplex).
    #[facet(args::named)]
    pub faces: Option<crate::pdf::Faces>,
    /// Gap between cards in millimetres (default 0, shared cut edges).
    #[facet(args::named)]
    #[arbitrary(default)]
    pub gap: Option<f32>,
    /// Card-size multiplier (default 1.0). Must fit a 3 x 3 grid on the selected paper.
    #[facet(args::named)]
    #[arbitrary(default)]
    pub scale: Option<f32>,
    /// Draw full-page cutting guidelines behind the cards (default true).
    #[facet(args::named, default = true)]
    pub guidelines: bool,
    /// Use cached images only.
    #[facet(args::named, default)]
    pub offline: bool,
    /// Replace an existing PDF.
    #[facet(args::named, default)]
    pub force: bool,
}
impl ProxyPdfGenerateArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let cards = ctx.decks.resolve(&self.deck, &ctx.db)?;
        let mut api = ctx.api()?;
        let summary = crate::pdf::generate(
            &cards,
            crate::pdf::Options {
                output: std::path::Path::new(&self.output),
                cache: &ctx.image_dir,
                paper: self.paper.unwrap_or(crate::pdf::Paper::A4),
                faces: self.faces.unwrap_or(crate::pdf::Faces::All),
                gap: self.gap.unwrap_or(0.0),
                scale: self.scale.unwrap_or(1.0),
                guidelines: self.guidelines,
                offline: self.offline,
                force: self.force,
            },
            &mut api,
        )?;
        Ok(CliOutput::facet(summary))
    }
}
