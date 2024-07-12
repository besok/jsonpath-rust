//! The parser for the jsonpath.
//! The module grammar denotes the structure of the parsing grammar

mod errors;
pub(crate) mod macros;
pub(crate) mod model;
#[allow(clippy::module_inception)]
pub(crate) mod parser;

pub use errors::JsonPathParserError;
pub use model::JsonPath;
pub use parser::{parse_json_path, Rule};
