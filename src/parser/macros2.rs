use crate::parser::model2::{Literal, SingularQuery};

#[macro_export]
macro_rules! lit {
    () => { Literal::Null };
    (b$b:expr ) => { Literal::Bool($b) };
    (s$s:expr) => { Literal::String($s.to_string()) };
    (i$n:expr) => { Literal::Int($n) };
    (f$n:expr) => { Literal::Float($n) };
}

#[macro_export]
macro_rules! q_segments {
    ($segment:tt) => {
        vec![q_segment!($segment)]
    };
    // Recursive case: multiple segments
    ($segment:tt $($rest:tt)*) => {{
        let mut segments = q_segments!($($rest)*);
        segments.insert(0, q_segment!($segment));
        segments
    }};
}

#[macro_export]
macro_rules! q_segment {
    ($name:ident) => { SingularQuerySegment::Name(stringify!($name).to_string()) };
    ([$name:ident]) => { SingularQuerySegment::Name(format!("\"{}\"", stringify!($name))) };
    ([$index:expr]) => { SingularQuerySegment::Index($index) };
}
#[macro_export]
macro_rules! singular_query {
    (@$($segment:tt)*) => {
        SingularQuery::Current(q_segments!($($segment)*))
    };
    ($($segment:tt)*) => {
        SingularQuery::Root(q_segments!($($segment)*))
    };
}