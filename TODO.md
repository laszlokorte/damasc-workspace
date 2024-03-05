# Damasc TODOs

* native pattern matching in lambda expression
* n-arity of lambdas
* n-arity in native functions
* extract native functions into modules
* syntax to define colored petri net (new package: damasc-flow)
* split damasc-lang into damasc-value and damasc-term
* pretty error reporting for syntax errors
* implement a proper declarative grammar
* add location information to expression and pattern identifiers
* refactor move damasc Value into own crate
* refactor move evaluation and matcher into own crate, separate from expression and pattern
* refactor move assignments and topology into own crate
* refactor move assignment evaluation 
* IMPORTANT: fix parsing for array/object splatting and comprehension separators
* IMPORTANT: Fix lifetimes in new grammar implementation to get rid of deep cloning
* Clean up parser error API to no expose parser library internals across crates
* Use new grammar crate for wasm
* Reimplement the grammar another time: LaLrPop
* add Dropdown/Mode to simply Repl Parsing