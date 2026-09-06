use super::generate::ProxyPdfGenerateArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum ProxyPdfCommand {
    Generate(ProxyPdfGenerateArgs),
}
