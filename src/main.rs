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
#![allow(dead_code)]

#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs;
use pest::Parser;
use pest::iterators::Pair;
use colored::*;


#[derive(Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;


// Value = _{ Array | String | Number | Boolean | Null }

enum Value {
	Null,
	Boolean(bool),
	Int(i64),
	Float(f64),
	String(String),
	Array(Vec<Value>)
}

// def interpret(Pair, SymbolTable)
fn interpret(node: Pair<'_, Rule>, indent: usize) -> Value {

	//println!("{:#?}", node.as_rule());
	println!(
		"{}{} -> {}",
		"    ".repeat(indent),
		format!("{:#?}", node.as_rule()).cyan(),
		format!("{}", node.as_str()).green(),
	);

	match node.as_rule() {
		Rule::Function => {
			println!("FOUND A FUNCTION :D");
		}

		Rule::String {
			println!("FOUND A STRING: {}", node.as_str());
		}
		
		_ => {}
	}

	for rule in node.into_inner() {
		interpret(rule, indent + 1);
		/*
		match rule.as_rule() {

			Rule::Function => {
				let mut inner_rules = rule.into_inner();
				println!("Function: \"{}\"", inner_rules.next().unwrap().as_str().green());

				for rule in inner_rules {
					println!("{} -> {}", "here".red(), rule.as_str().red());
					interpret(rule);
				}
			}

			Rule::Number => {
				println!("Number: \"{}\"", rule.as_str().green());
			}

			Rule::LineComment => (),

			Rule::EOI => (),

			//_ => unreachable!()

			_ => {
				println!("99999999999999999 {}", rule.as_str().red());
			}
		}
		*/
	}

	Value::Null
}


fn main() {
	let unparsed_file = fs::read_to_string("hello.lcore").expect("LCORE: Error Reading File");

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	println!("{:#?}", program);

	// for rule in program.into_inner() {
	// 	//println!("+++++++++++++++++++++++++++++++++++++++++++++");
	// 	//println!("{:#?}", rule.as_rule());
	// 	//println!("---------------------------------------------");

	// 	match rule.as_rule() {
	// 		Rule::Function => {
	// 			let mut inner_rules = rule.into_inner();
	// 			println!("Function: \"{}\"", inner_rules.next().unwrap().as_str());
	// 		}

	// 		Rule::LineComment => (),
	// 		Rule::EOI => (),
	// 		_ => unreachable!()
	// 	}
	// }

	//let mut symbol_table: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
	interpret(program, 0);
}
