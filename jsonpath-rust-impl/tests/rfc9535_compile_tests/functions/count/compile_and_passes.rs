// Test case: 00_count_function
// Tags: function, count
#[test]
fn test_00_count_function() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[?count(@..*)>2]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@..*)>2]"#).expect("failed to parse");
	assert_eq!(q_pest, q_ast);
}

// Test case: 01_single_node_arg
// Tags: function, count
#[test]
fn test_01_single_node_arg() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[?count(@.a)>1]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@.a)>1]"#).expect("failed to parse");
	assert_eq!(q_pest, q_ast);
}

// Test case: 02_multiple_selector_arg
// Tags: function, count
#[test]
fn test_02_multiple_selector_arg() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[?count(@["a","d"])>1]);
    let q_pest_double = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@["a","d"])>1]"#).expect("failed to parse");
    let q_pest_single = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@['a','d'])>1]"#).expect("failed to parse");
	assert_eq!(q_pest_double, q_ast);
    assert_eq!(q_pest_single, q_ast);
}
