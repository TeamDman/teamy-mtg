use super::card::DeckCardArgs;
use super::create::DeckCreateArgs;
use super::list::DeckListArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum DeckCommand {
    List(DeckListArgs),
    Create(DeckCreateArgs),
    Card(DeckCardArgs),
}
