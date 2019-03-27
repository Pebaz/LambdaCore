/*
https://pest.rs/
https://docs.rs/pest/2.1.0/pest/
https://docs.rs/pest_derive/2.1.0/pest_derive/
https://pest.rs/book/examples/ini.html

{ Program : [{ print : ["Hello World"] }] }

[ ] Experiment with walking the tree
[ ] Eval Value
[ ] Eval Function

Parsed 78k LOC in ~500ms.
*/

#![allow(unused_imports)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs;
use pest::Parser;


#[derive(Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;


fn main() {
	let unparsed_file = fs::read_to_string("hello.lcore").expect("LCORE: Error Reading File");

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	//println!("{:#?}", program);

	for rule in program.into_inner() {
		//println!("+++++++++++++++++++++++++++++++++++++++++++++");
		//println!("{:#?}", rule.as_rule());
		//println!("---------------------------------------------");

		match rule.as_rule() {
			Rule::Function => {
				let mut inner_rules = rule.into_inner();
				println!("Function: \"{}\"", inner_rules.next().unwrap().as_str());
			}

			Rule::LineComment => (),
			Rule::EOI => (),
			_ => unreachable!()
		}
	}
}
