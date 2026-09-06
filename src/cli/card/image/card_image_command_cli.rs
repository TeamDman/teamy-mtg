use super::list::CardImageListArgs;
use super::sync::CardImageSyncArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum CardImageCommand {
    Sync(CardImageSyncArgs),
    List(CardImageListArgs),
}
