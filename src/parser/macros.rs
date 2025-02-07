#[macro_export]
macro_rules! filter {
   () => {FilterExpression::Atom(op!,FilterSign::new(""),op!())};
   ( $left:expr, $s:literal, $right:expr) => {
      FilterExpression::Atom($left,FilterSign::new($s),$right)
   };
   ( $left:expr,||, $right:expr) => {FilterExpression::Or(Box::new($left),Box::new($right)) };
   ( $left:expr,&&, $right:expr) => {FilterExpression::And(Box::new($left),Box::new($right)) };
   ( count $inner:expr  ) => { FilterExpression::Extension(FilterExt::Count, vec![filter!(op!($inner),"exists",op!())])};
   ( length $inner:expr  ) =>  { FilterExpression::Extension(FilterExt::Length, vec![filter!(op!($inner),"exists",op!())])};
   ( value $inner:expr  ) => { FilterExpression::Extension(FilterExt::Value, vec![filter!(op!($inner),"exists",op!())])};
   ( search  $inner1:expr,$inner2:expr  ) => { FilterExpression::Extension(FilterExt::Search,vec![
       filter!(op!($inner1),"exists",op!()),
       filter!(op!($inner2),"exists",op!())
   ])};
   ( match_  $inner1:expr,$inner2:expr  ) => { FilterExpression::Extension(FilterExt::Match,vec![
       filter!(op!($inner1),"exists",op!()),
       filter!(op!($inner2),"exists",op!())
   ])};
}

#[macro_export]
macro_rules! op {
    ( ) => {
        Operand::Dynamic(Box::new(JsonPath::Empty))
    };
    ( $s:literal) => {
        Operand::Static(json!($s))
    };
    ( s $s:expr) => {
        Operand::Static(json!($s))
    };
    ( $s:expr) => {
        Operand::Dynamic(Box::new($s))
    };
}

#[macro_export]
macro_rules! idx {
   ( $s:literal) => {JsonPathIndex::Single(json!($s))};
   ( idx $($ss:literal),+) => {{
       let ss_vec = vec![
           $(
               json!($ss),
           )+
       ];
       JsonPathIndex::UnionIndex(ss_vec)
   }};
   ( $($ss:literal),+) => {{
       let ss_vec = vec![
           $(
               $ss.to_string(),
           )+
       ];
       JsonPathIndex::UnionKeys(ss_vec)
   }};
   ( $s:literal) => {JsonPathIndex::Single(json!($s))};
   ( ? $s:expr) => {JsonPathIndex::Filter($s)};
   ( [$l:literal;$m:literal;$r:literal]) => {JsonPathIndex::Slice(Some($l),Some($m),Some($r))};
   ( [$l:literal;$m:literal;]) => {JsonPathIndex::Slice($l,$m,1)};
   ( [$l:literal;;$m:literal]) => {JsonPathIndex::Slice($l,0,$m)};
   ( [;$l:literal;$m:literal]) => {JsonPathIndex::Slice(0,$l,$m)};
   ( [;;$m:literal]) => {JsonPathIndex::Slice(None,None,Some($m))};
   ( [;$m:literal;]) => {JsonPathIndex::Slice(None,Some($m),None)};
   ( [$m:literal;;]) => {JsonPathIndex::Slice(Some($m),None,None)};
   ( [;;]) => {JsonPathIndex::Slice(None,None,None)};
}

#[macro_export]
macro_rules! chain {
    ($($ss:expr),+) => {{
        let ss_vec = vec![
            $(
                $ss,
            )+
        ];
        JsonPath::Chain(ss_vec)
   }};
}

/// Can be used to Parse a JsonPath with a more native way.
/// e.g.
/// ```rust
/// use jsonpath_rust::{path, JsonPath};
/// use std::str::FromStr;
/// use serde_json::Value;
///
/// let path:JsonPath<Value> = JsonPath::from_str(".abc.*").unwrap();
/// let path2 = JsonPath::Chain(vec![path!("abc"), path!(*)]);
/// assert_eq!(path, path2);
/// ```
#[macro_export]
macro_rules! path {
   ( ) => {JsonPath::Empty};
   (*) => {JsonPath::Wildcard};
   ($) => {JsonPath::Root};
   (@) => {JsonPath::Current(Box::new(JsonPath::Empty))};
   (@$e:expr) => {JsonPath::Current(Box::new($e))};
   (@,$($ss:expr),+) => {{
       let ss_vec = vec![
           $(
               $ss,
           )+
       ];
       let chain = JsonPath::Chain(ss_vec);
       JsonPath::Current(Box::new(chain))
   }};
   (..$e:literal) => {JsonPath::Descent($e.to_string())};
   (..*) => {JsonPath::DescentW};
   ($e:literal) => {JsonPath::Field($e.to_string())};
   ($e:expr) => {JsonPath::Index($e)};
}

#[cfg(test)]
pub(crate) use chain;
#[cfg(test)]
pub(crate) use filter;
#[cfg(test)]
pub(crate) use idx;
#[cfg(test)]
pub(crate) use op;
