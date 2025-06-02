// Test case: 03_non_query_arg_number
// Tags: function, count
#[test]
fn test_03_non_query_arg_number() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(1)>2]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(1)>2]"#).expect_err("should not parse");
}

// Test case: 04_non_query_arg_string
// Tags: function, count
#[test]
fn test_04_non_query_arg_string() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count('string')>2]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count('string')>2]"#).expect_err("should not parse");
}

// Test case: 05_non_query_arg_true
// Tags: function, count
#[test]
fn test_05_non_query_arg_true() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(true)>2]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(true)>2]"#).expect_err("should not parse");
}

// Test case: 06_non_query_arg_false
// Tags: function, count
#[test]
fn test_06_non_query_arg_false() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(false)>2]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(false)>2]"#).expect_err("should not parse");
}

// Test case: 07_non_query_arg_null
// Tags: function, count
#[test]
fn test_07_non_query_arg_null() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(null)>2]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(null)>2]"#).expect_err("should not parse");
}

// Test case: 08_result_must_be_compared
// Tags: function, count
#[test]
fn test_08_result_must_be_compared() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(@..*)]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@..*)]"#).expect_err("should not parse");
}

// Test case: 09_no_params
// Tags: function, count
#[test]
fn test_09_no_params() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count()==1]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count()==1]"#).expect_err("should not parse");
}

// Test case: 10_too_many_params
// Tags: function, count
#[test]
fn test_10_too_many_params() {
    // let q_ast = ::jsonpath_rust_impl::json_query!($[?count(@.a,@.b)==1]);
    let _q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[?count(@.a,@.b)==1]"#).expect_err("should not parse");
}
