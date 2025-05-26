//! Test case: 1_no_leading_whitespace
//! Tags: whitespace

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    let _pest = Main::try_from_pest_parse(" $").expect_err("Pest cares about whitespace");
    let _syn = json_query!(  $ );
}
