mod parser;

include!(concat!(env!("OUT_DIR"), "/stmt_generated.rs"));

#[allow(unused_imports)]
pub use stmt_generated::*;
