// Test case: 01_no_leading_whitespace
// Tags: whitespace
#[test]
fn test_01_no_leading_whitespace() {
    // let q_ast = ::jsonpath_rust_impl::json_query!( $);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#" $"#).expect_err("should not parse");
}

// Test case: 02_no_trailing_whitespace
// Tags: whitespace
#[test]
fn test_02_no_trailing_whitespace() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($ );
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$ "#).expect_err("should not parse");
}

// Test case: 06_name_shorthand_symbol
// Tags: No tags
#[test]
fn test_06_name_shorthand_symbol() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($.&);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.&"#).expect_err("should not parse");
}

// Test case: 07_name_shorthand_number
// Tags: No tags
#[test]
fn test_07_name_shorthand_number() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($.1);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.1"#).expect_err("should not parse");
}

// Test case: 16_multiple_selectors_space_instead_of_comma
// Tags: whitespace
#[test]
fn test_16_multiple_selectors_space_instead_of_comma() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[0 2]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[0 2]"#).expect_err("should not parse");
}

// Test case: 17_selector_leading_comma
// Tags: No tags
#[test]
fn test_17_selector_leading_comma() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[,0]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[,0]"#).expect_err("should not parse");
}

// Test case: 18_selector_trailing_comma
// Tags: No tags
#[test]
fn test_18_selector_trailing_comma() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[0,]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[0,]"#).expect_err("should not parse");
}

// Test case: 28_empty_segment
// Tags: No tags
#[test]
fn test_28_empty_segment() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[]"#).expect_err("should not parse");
}

// Test case: 39_bald_descendant_segment
// Tags: No tags
#[test]
fn test_39_bald_descendant_segment() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($..);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.."#).expect_err("should not parse");
}

// Test case: 40_current_node_identifier_without_filter_selector
// Tags: No tags
#[test]
fn test_40_current_node_identifier_without_filter_selector() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[@.a]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[@.a]"#).expect_err("should not parse");
}

// Test case: 41_root_node_identifier_in_brackets_without_filter_selector
// Tags: No tags
#[test]
fn test_41_root_node_identifier_in_brackets_without_filter_selector() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[$.a]);
    let _q_pest =
        ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[$.a]"#).expect_err("should not parse");
}
