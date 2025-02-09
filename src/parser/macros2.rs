use crate::parser::model2::Literal;

#[macro_export]
macro_rules! lit {
    () => { Literal::Null };
    (b$b:expr ) => { Literal::Bool($b) };
    (s$s:expr) => { Literal::String($s.to_string()) };
    (i$n:expr) => { Literal::Int($n) };
    (f$n:expr) => { Literal::Float($n) };
}