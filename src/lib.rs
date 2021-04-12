mod json;
mod parser;
mod path;

#[macro_use]
extern crate pest_derive;
extern crate pest;
use crate::parser::parser::JsonPathParser;
use pest::Parser;

fn main() {
    let successful_parse = JsonPathParser::parse(Rule::field, "-273.15");
    println!("{:?}", successful_parse);

    let unsuccessful_parse = JsonPathParser::parse(Rule::field, "this is not a number");
    println!("{:?}", unsuccessful_parse);
}