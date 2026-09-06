use super::image::CardImageArgs;
use super::list::CardListArgs;
use super::search::CardSearchArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum CardCommand {
    List(CardListArgs),
    Search(CardSearchArgs),
    Image(CardImageArgs),
}
