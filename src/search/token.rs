#[derive(Debug, PartialEq)]
pub(super) enum Token {
    Word(String),
    Open,
    Close,
    And,
    Or,
    Not,
}
