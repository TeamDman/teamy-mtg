use super::token::Token;
use eyre::ContextCompat;
use eyre::Result;
use eyre::ensure;
pub(super) fn lex(input: &str) -> Result<Vec<Token>> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        match c {
            '(' => {
                chars.next();
                tokens.push(Token::Open);
            }
            ')' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Not);
            }
            _ => {
                let mut word = String::new();
                let mut quote = false;
                let mut regex = false;
                while let Some(&ch) = chars.peek() {
                    if !quote && !regex && (ch.is_whitespace() || ch == '(' || ch == ')') {
                        break;
                    }
                    chars.next();
                    if ch == '"' && !regex {
                        if word.is_empty() {
                            word.push_str("name:");
                        }
                        quote = !quote;
                        continue;
                    }
                    if ch == '\\' && (quote || regex) {
                        let next = chars.next().context("Trailing escape in query")?;
                        if regex {
                            word.push('\\');
                        }
                        word.push(next);
                        continue;
                    }
                    if ch == '/' && !quote && (regex || word.ends_with(':')) {
                        regex = !regex;
                    }
                    word.push(ch);
                }
                ensure!(!quote && !regex, "Unclosed quote or regular expression");
                ensure!(!word.is_empty(), "Empty query term");
                tokens.push(match word.as_str() {
                    "AND" | "and" => Token::And,
                    "OR" | "or" => Token::Or,
                    "NOT" | "not" => Token::Not,
                    _ => Token::Word(word),
                });
            }
        }
        ensure!(tokens.len() <= 512, "Too many query terms");
    }
    Ok(tokens)
}
