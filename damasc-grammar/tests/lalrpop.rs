#![feature(assert_matches)]
use core::assert_matches::assert_matches;
use damasc_grammar::experiment::lexer::Lexer;
use damasc_grammar::experiment::grammar::ExpressionParser;

#[test]
fn foo() {
  let source_code = "500+5";
  let lexer = Lexer::new(&source_code[..]);
  let parser = ExpressionParser::new();
  let ast = parser.parse(lexer);

  assert_matches!(ast, Ok(_));
}