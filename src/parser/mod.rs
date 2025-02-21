//! The parser for the jsonpath.
//! The module grammar denotes the structure of the parsing grammar

mod errors;
pub(crate) mod macros;
pub(crate) mod model;
#[allow(clippy::module_inception)]
pub(crate) mod parser;
#[allow(clippy::module_inception)]
pub mod parser2;
pub mod model2;
pub mod errors2;
mod macros2;
mod tests;

pub use errors::JsonPathParserError;
pub use model::FilterExpression;
pub use model::JsonPath;
pub use model::JsonPathIndex;
pub use model::Operand;
pub use parser::{parse_json_path, Rule};
