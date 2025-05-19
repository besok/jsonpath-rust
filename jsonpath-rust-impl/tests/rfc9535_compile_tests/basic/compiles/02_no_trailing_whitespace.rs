//! Test case: 2_no_trailing_whitespace
//! Tags: whitespace

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    let _pest = Main::try_from_pest_parse("$ ").expect_err("Pest cares about whitespace");
    let _syn = json_query! ( $  );
}
