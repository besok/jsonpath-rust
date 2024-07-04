#[cfg(test)]
macro_rules! filter {
   () => {FilterExpression::Atom(op!,FilterSign::new(""),op!())};
   ( $left:expr, $s:literal, $right:expr) => {
      FilterExpression::Atom($left,FilterSign::new($s),$right)
   };
   ( $left:expr,||, $right:expr) => {FilterExpression::Or(Box::new($left),Box::new($right)) };
   ( $left:expr,&&, $right:expr) => {FilterExpression::And(Box::new($left),Box::new($right)) };
}

#[cfg(test)]
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

#[cfg(test)]
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
   ( [$l:literal;$m:literal;$r:literal]) => {JsonPathIndex::Slice($l,$m,$r)};
   ( [$l:literal;$m:literal;]) => {JsonPathIndex::Slice($l,$m,1)};
   ( [$l:literal;;$m:literal]) => {JsonPathIndex::Slice($l,0,$m)};
   ( [;$l:literal;$m:literal]) => {JsonPathIndex::Slice(0,$l,$m)};
   ( [;;$m:literal]) => {JsonPathIndex::Slice(0,0,$m)};
   ( [;$m:literal;]) => {JsonPathIndex::Slice(0,$m,1)};
   ( [$m:literal;;]) => {JsonPathIndex::Slice($m,0,1)};
   ( [;;]) => {JsonPathIndex::Slice(0,0,1)};
}

#[cfg(test)]
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
///
/// let path = JsonPath::from_str(".abc.*").unwrap();
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
