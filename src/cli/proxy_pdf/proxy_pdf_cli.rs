use super::ProxyPdfCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ProxyPdfArgs {
    #[facet(args::subcommand)]
    pub command: ProxyPdfCommand,
}
impl ProxyPdfArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            ProxyPdfCommand::Generate(args) => args.invoke(ctx),
        }
    }
}
