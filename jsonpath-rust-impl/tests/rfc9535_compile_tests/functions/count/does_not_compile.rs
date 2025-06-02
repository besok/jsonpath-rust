// Test case: 03_non_query_arg_number
// Tags: function, count
fn test_03_non_query_arg_number() {
    ::jsonpath_rust_impl::json_query!($[?count(1)>2]);
}

// Test case: 04_non_query_arg_string
// Tags: function, count
fn test_04_non_query_arg_string() {
    ::jsonpath_rust_impl::json_query!($[?count('string')>2]);
}

// Test case: 05_non_query_arg_true
// Tags: function, count
fn test_05_non_query_arg_true() {
    ::jsonpath_rust_impl::json_query!($[?count(true)>2]);
}

// Test case: 06_non_query_arg_false
// Tags: function, count
fn test_06_non_query_arg_false() {
    ::jsonpath_rust_impl::json_query!($[?count(false)>2]);
}

// Test case: 07_non_query_arg_null
// Tags: function, count
fn test_07_non_query_arg_null() {
    ::jsonpath_rust_impl::json_query!($[?count(null)>2]);
}

// Test case: 08_result_must_be_compared
// Tags: function, count
fn test_08_result_must_be_compared() {
    ::jsonpath_rust_impl::json_query!($[?count(@..*)]);
}

// Test case: 09_no_params
// Tags: function, count
fn test_09_no_params() {
    ::jsonpath_rust_impl::json_query!($[?count()==1]);
}

// Test case: 10_too_many_params
// Tags: function, count
fn test_10_too_many_params() {
    ::jsonpath_rust_impl::json_query!($[?count(@.a,@.b)==1]);
}
