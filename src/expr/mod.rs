mod parser;

include!(concat!(env!("OUT_DIR"), "/expr_generated.rs"));

#[allow(unused_imports)]
pub use expr_generated::*;

use self::parser::Parser;
use crate::tokens::Tokens;

impl TryFrom<Tokens> for Expr {
    type Error = Vec<String>;

    fn try_from(tokens: Tokens) -> Result<Self, Self::Error> {
        let mut parser: Parser = tokens.into();
        parser.parse()
    }
}
