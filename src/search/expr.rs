use super::term::Term;
use crate::model::Card;
#[derive(Debug)]
pub(super) enum Expr {
    Term(Term),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
impl Expr {
    pub(super) fn matches(&self, card: &Card) -> bool {
        match self {
            Self::Term(term) => term.matches(card),
            Self::And(a, b) => a.matches(card) && b.matches(card),
            Self::Or(a, b) => a.matches(card) || b.matches(card),
            Self::Not(a) => !a.matches(card),
        }
    }
}
