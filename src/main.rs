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


Goals:
[ ]
*/

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs;
use std::fmt;
use pest::Parser;
use pest::iterators::Pair;
use colored::*;


#[derive(Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;


enum Value {
	Null,
	Identifier(String),
	Boolean(bool),
	Int(i64),
	Float(f64),
	String(String),
	Array(Vec<Value>),
	Func { f: fn(&mut Value) }
}

impl fmt::Debug for Value {
	fn fmt(&self, fm: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Value::Null           => {  write!(fm, "Null")       }
			Value::Identifier(i)  => {  write!(fm, "Identifier") }
			Value::Boolean(b)     => {  write!(fm, "Boolean")    }
			Value::Int(i)         => {  write!(fm, "Int")        }
			Value::Float(fl)      => {  write!(fm, "Float")      }
			Value::String(s)      => {  write!(fm, "String")     }
			Value::Array(a)       => {  write!(fm, "Array")      }
			Value::Func { f }     => {  write!(fm, "Func")       }
		}
	}
}


//fn get_args


fn interpret(
	node: Pair<'_, Rule>,
	indent: usize,
	symtab: &mut HashMap<&str, Value>
) -> Value {

	let return_value = Value::Null;

	if format!("{:#?}", node.as_rule()) != "Program" {
		println!(
			"{}{} -> {}",
			if indent > 0 { "    ".repeat(indent) } else { String::from("") },
			format!("{:#?}", node.as_rule()).cyan(),
			format!("{}", node.as_str()).green(),
		);
	}

	match node.as_rule() {
		Rule::Program => {
			for rule in node.into_inner() {
				// NOTE(pebaz): Keep the indent at 0 for the first time
				interpret(rule, indent, symtab);
			}

			// Interpret the program
		}

		Rule::Identifier => {
			// Lookup the value of identifier and return that
			// e.g. Function, String, Array, etc.
		}
		
		Rule::Function => {

			//let mut args = Vec<Value>;

			for rule in node.into_inner() {
				interpret(rule, indent + 1, symtab);
			}

			// Execute function (built-in or otherwise)
		}

		Rule::Array => {
			for rule in node.into_inner() {
				interpret(rule, indent + 1, symtab);
			}

			// Return Value::Array
		}

		Rule::String => {
			let return_value = Value::String(String::from(node.as_str()));
			/*
			println!("FOUND STRING: {}", match return_value {
				Value::String(s) => { s.as_str().to_owned() }
				_ => { "".to_owned() }
			});
			*/

			// Return Value::String
		}
		
		Rule::Number => {
			// Return either Value::Int or Value::Float
		}

		Rule::Boolean => {
			// Return Value::Boolean
		}

		_ => {}
	}

	/*
	for rule in node.into_inner() {
		interpret(rule, indent + 1);
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
	}
	*/

	return_value
}


fn main() {
	let unparsed_file = fs::read_to_string("print.lcore").expect("LCORE: Error Reading File");

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	println!("{:#?}", program);

	fn lcore_print(args: &mut Value) {
		println!("lcore_print()");
	}

	let mut symbol_table: HashMap<&str, Value> = HashMap::new();

	// Fill the symbol table with built-in functions
	symbol_table.insert("print", Value::Func { f: lcore_print });

	// Interpret the Program
	interpret(program, 0, &mut symbol_table);
}
