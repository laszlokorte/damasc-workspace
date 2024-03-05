use crate::lpop::tokens::Token;
use logos::Logos;
use logos::{SpannedIter};


pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
  InvalidToken,
}

pub struct Lexer<'input> {
  // instead of an iterator over characters, we have a token iterator
  token_stream: SpannedIter<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
  pub fn new(input: &'input str) -> Self {
    // the Token::lexer() method is provided by the Logos trait
    Self { token_stream: Token::lexer(input).spanned() }
  }
}

impl<'input> Iterator for Lexer<'input> {
  type Item = Spanned<Token<'input>, usize, LexicalError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.token_stream.next().map(|(token, span)| {
      match token {
        // an invalid token was met
        Ok(t) => Ok((span.start, t, span.end)),
        Err(e) => {dbg!(e);Err(LexicalError::InvalidToken)},
      }
    })
  }
}