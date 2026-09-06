use super::update::DbUpdateArgs;
use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum DbCommand {
    Update(DbUpdateArgs),
}
