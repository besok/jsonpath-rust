use crate::parser::model::{Comparable, Filter, FilterAtom, FnArg, Literal, Segment, SingularQuery, Test};

#[macro_export]
macro_rules! lit {
    () => {
        Literal::Null
    };
    (b$b:expr ) => {
        Literal::Bool($b)
    };
    (s$s:expr) => {
        Literal::String($s.to_string())
    };
    (i$n:expr) => {
        Literal::Int($n)
    };
    (f$n:expr) => {
        Literal::Float($n)
    };
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
    ($name:ident) => {
        SingularQuerySegment::Name(stringify!($name).to_string())
    };
    ([$name:ident]) => {
        SingularQuerySegment::Name(format!("\"{}\"", stringify!($name)))
    };
    ([$index:expr]) => {
        SingularQuerySegment::Index($index)
    };
}
#[macro_export]
macro_rules! singular_query {
    (@ $($segment:tt)*) => {
        SingularQuery::Current(q_segments!($($segment)*))
    };
    ($($segment:tt)*) => {
        SingularQuery::Root(q_segments!($($segment)*))
    };
}

#[macro_export]
macro_rules! slice {
    () => {
        (None, None, None)
    };
    ($start:expr) => {
        (Some($start), None, None)
    };
    ($start:expr, $end:expr) => {
        (Some($start), Some($end), None)
    };
    ($start:expr,, $step:expr) => {
        (Some($start), None, Some($step))
    };
    (,, $step:expr) => {
        (None, None, Some($step))
    };
    (, $end:expr) => {
        (None, Some($end), None)
    };
    (, $end:expr,$step:expr ) => {
        (None, Some($end), Some($step))
    };
    ($start:expr, $end:expr, $step:expr) => {
        (Some($start), Some($end), Some($step))
    };
}

#[macro_export]
macro_rules! test_fn {
    ($name:ident $arg:expr) => {
        TestFunction::try_new(stringify!($name), vec![$arg]).unwrap()
    };

    ($name:ident $arg1:expr, $arg2:expr ) => {
        TestFunction::try_new(stringify!($name), vec![$arg1, $arg2]).unwrap()
    };
}

#[macro_export]
macro_rules! arg {
    ($arg:expr) => {
        FnArg::Literal($arg)
    };
    (t $arg:expr) => {
        FnArg::Test(Box::new($arg))
    };
    (f $arg:expr) => {
        FnArg::Filter($arg)
    }
}

#[macro_export]
macro_rules! test {
  (@ $($segments:expr)*) => { Test::RelQuery(vec![$($segments),*]) };
  (S $jq:expr) => { Test::AbsQuery($jq) };
  ($tf:expr) => { Test::Function(Box::new($tf)) };

}

#[macro_export]
macro_rules! or {
    ($($items:expr),*) => {
       crate::parser::model::Filter::Or(vec![ $($items),* ])
    };
}

#[macro_export]
macro_rules! and {
    ($($items:expr),*) => {
        crate::parser::model::Filter::And(vec![ $($items),* ])
    };
}

#[macro_export]
macro_rules! filter_ {
    ($item:expr) => {
        crate::parser::model::Filter::Atom($item)
    };

    (or $($items:expr),*) => {
       crate::parser::model::Filter::Or(vec![ $($items),* ])
    };

    (and $($items:expr),*) => {
        crate::parser::model::Filter::And(vec![ $($items),* ])
    };
}

#[macro_export]
macro_rules! atom {
    (! $filter:expr) => {
        FilterAtom::filter($filter, true)
    };
    ($filter:expr) => {
        FilterAtom::filter($filter, false)
    };
    (t! $filter:expr) => {
        FilterAtom::test($filter, true)
    };
    (t $filter:expr) => {
        FilterAtom::filter($filter, false)
    };
    ($lhs:expr, $s:expr, $rhs:expr) => {
        FilterAtom::Comparison(Box::new(cmp!($lhs, $s, $rhs)))
    };
}

#[macro_export]
macro_rules! cmp {
  ($lhs:expr, $op:expr , $rhs:expr) => {
      Comparison::try_new($op, $lhs, $rhs).unwrap()
  }
}

#[macro_export]
macro_rules! comparable {
    ($lit:expr) => {
        Comparable::Literal($lit)
    };
    (f $func:expr) => {
        Comparable::Function($func)
    };
    (> $sq:expr) => {
        Comparable::SingularQuery($sq)
    };
}

#[macro_export]
macro_rules! selector {
    (*) => {
        Selector::Wildcard
    };
    (?$filter:expr) => {
        Selector::Filter($filter)
    };
    ($name:ident) => {
        Selector::Name(stringify!($name).to_string())
    };
    ([$name:ident]) => {
        Selector::Name(format!("\"{}\"", stringify!($name)))
    };
    ([$index:expr]) => {
        Selector::Index($index)
    };
}

#[macro_export]
macro_rules! segment {
    (..$segment:expr) => {
        Segment::Descendant(Box::new($segment))
    };
    ($selector:expr) => {
        Segment::Selector($selector)
    };
    ($($selectors:expr),*) => {
        Segment::Selectors(vec![$($selectors),*])
    };
}

#[macro_export]
macro_rules! jq {
    ($($segment:expr),*) => {
        JpQuery::new(vec![$($segment),*])
    };
}