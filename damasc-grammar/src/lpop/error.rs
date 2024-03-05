#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexError {
	NumberParsing,
	StringParsing,

	#[default] 
	Other
}