use crate::lpop::error::LexError;
use core::fmt;
use logos::Logos;


#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error = LexError)]
pub enum Token<'s> {
  #[token("for")]
  KeywordFor,
  #[token("if")]
  KeywordIf,
  #[token("else")]
  KeywordElse,
  #[token("match")]
  KeywordMatch,
  #[token("fn")]
  KeywordFn,
  #[token("where")]
  KeywordWhere,
  #[token("into")]
  KeywordInto,
  #[token("limit")]
  KeywordLimit,
  #[token("with")]
  KeywordWith,
  #[token("in")]
  KeywordIn,
  #[token("as")]
  KeywordAs,
  #[token("action")]
  KeywordAction,
  #[token("bag")]
  KeywordBag,

  #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice())]
  Identifier(&'s str),

  #[regex(r"\d+", |lex| lex.slice())]
  Int(&'s str),

  #[token("(")]
  LParen,
  #[token(")")]
  RParen,

  #[token("{")]
  LCurly,
  #[token("}")]
  RCurly,

  #[token("[")]
  LBrack,
  #[token("]")]
  RBrack,


  #[token("=>")]
  Rocket,
  #[token(",")]
  Comma,
  #[token(".")]
  Period,
  #[token("`")]
  Tick,

  #[token("=")]
  Assign,
  #[token(";")]
  Semicolon,
  #[token(":")]
  Colon,

  #[token("+")]
  OperatorAdd,
  #[token("-")]
  OperatorSub,
  #[token("*")]
  OperatorMul,
  #[token("/")]
  OperatorDiv,
  #[token("^")]
  OperatorExp,


  #[token("&&")]
  OperatorLogicAnd,
  #[token("||")]
  OperatorLogicOr,
  #[token("!")]
  OperatorLogicNot,

  #[token("==")]
  OperatorEqual,
  #[token("!=")]
  OperatorNotEqual,
  #[token("<")]
  OperatorLess,
  #[token(">")]
  OperatorGreater,
  #[token("<=")]
  OperatorLessOrEqual,
  #[token(">=")]
  OperatorGreaterOrEqual,
}

impl fmt::Display for Token<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
