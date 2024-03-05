use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);

pub mod ast {
  #[derive(Clone, Debug, PartialEq)]
  pub enum Statement {
    Variable { name: String, value: Box<Expression> },
    Print { value: Box<Expression> },
  }

  #[derive(Clone, Debug, PartialEq)]
  pub enum Expression {
    Integer(i64),
    Variable(String),
    BinaryOperation { lhs: Box<Expression>, operator: Operator, rhs: Box<Expression>, left: usize, right: usize },
  }

  #[derive(Clone, Debug, PartialEq)]
  pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
  }

  // to implement the Display trait
}


pub mod tokens{

use core::fmt;
use logos::Logos;


#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error = String)]
pub enum Token {
  #[token("var")]
  KeywordVar,
  #[token("print")]
  KeywordPrint,

  #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().parse().map_err(|_|"foo".to_string()))]
  Identifier(String),
  #[regex(r"\d+", |lex| lex.slice().parse().map_err(|_| "could not parse".to_string()).and_then(|i| if i > 100 {Err("too large numberrr".to_string())} else {Ok(i)}))]
  Integer(i64),

  #[token("(")]
  LParen,
  #[token(")")]
  RParen,
  #[token("=")]
  Assign,
  #[token(";")]
  Semicolon,

  #[token("+")]
  OperatorAdd,
  #[token("-")]
  OperatorSub,
  #[token("*")]
  OperatorMul,
  #[token("/")]
  OperatorDiv,
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}


}


pub mod lexer {

  use logos::Logos;
  use crate::experiment::tokens::Token;
  use logos::{SpannedIter};


  pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

  #[derive(Debug)]
  pub enum LexicalError {
    InvalidToken,
  }

  pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
  }

  impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
      // the Token::lexer() method is provided by the Logos trait
      Self { token_stream: Token::lexer(input).spanned() }
    }
  }

  impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

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
  }