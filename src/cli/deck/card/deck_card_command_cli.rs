use super::add::DeckCardAddArgs;
use super::list::DeckCardListArgs;
use super::remove::DeckCardRemoveArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum DeckCardCommand {
    Add(DeckCardAddArgs),
    List(DeckCardListArgs),
    Remove(DeckCardRemoveArgs),
}
