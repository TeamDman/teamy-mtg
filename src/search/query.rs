use super::expr::Expr;
use super::lexer::lex;
use super::parser::Parser;
use crate::model::Card;
use eyre::Result;
use eyre::ensure;

#[derive(Debug)]
pub struct Query(Expr);

impl Query {
    pub fn parse(input: &str) -> Result<Self> {
        ensure!(input.len() <= 8192, "Query exceeds 8192 bytes");
        let tokens = lex(input)?;
        ensure!(!tokens.is_empty(), "Search query cannot be empty");
        let mut parser = Parser {
            tokens,
            index: 0,
            depth: 0,
        };
        let expr = parser.or()?;
        ensure!(
            parser.index == parser.tokens.len(),
            "Unexpected token at end of query"
        );
        Ok(Self(expr))
    }
    pub fn matches(&self, card: &Card) -> bool {
        self.0.matches(card)
    }
}
