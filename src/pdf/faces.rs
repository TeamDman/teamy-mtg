use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Debug, Clone, Copy, Facet, Arbitrary, PartialEq)]
#[repr(u8)]
#[facet(rename_all = "lowercase")]
pub enum Faces {
    All,
    Front,
    Back,
}
