use super::expr::Expr;
use super::term::Term;
use super::token::Token;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
pub(super) struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) index: usize,
    pub(super) depth: usize,
}
impl Parser {
    pub(super) fn or(&mut self) -> Result<Expr> {
        let mut left = self.and()?;
        while self.tokens.get(self.index) == Some(&Token::Or) {
            self.index += 1;
            left = Expr::Or(Box::new(left), Box::new(self.and()?));
        }
        Ok(left)
    }
    fn and(&mut self) -> Result<Expr> {
        let mut left = self.atom()?;
        while self.index < self.tokens.len()
            && !matches!(self.tokens[self.index], Token::Or | Token::Close)
        {
            if self.tokens[self.index] == Token::And {
                self.index += 1;
            }
            left = Expr::And(Box::new(left), Box::new(self.atom()?));
        }
        Ok(left)
    }
    fn atom(&mut self) -> Result<Expr> {
        self.depth += 1;
        ensure!(self.depth <= 64, "Query nesting exceeds 64 levels");
        let result = match self.tokens.get(self.index) {
            Some(Token::Not) => {
                self.index += 1;
                Expr::Not(Box::new(self.atom()?))
            }
            Some(Token::Open) => {
                self.index += 1;
                let result = self.or()?;
                ensure!(
                    self.tokens.get(self.index) == Some(&Token::Close),
                    "Missing closing parenthesis"
                );
                self.index += 1;
                result
            }
            Some(Token::Word(word)) => {
                let term = Term::parse(word)?;
                self.index += 1;
                Expr::Term(term)
            }
            _ => bail!("Expected a search term"),
        };
        self.depth -= 1;
        Ok(result)
    }
}
