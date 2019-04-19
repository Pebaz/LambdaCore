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
use std::process::exit;
use std::str::FromStr;
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

trait GetActualValues {
	fn as_identifier(&self) -> &String;
	fn as_bool(&self)       -> &bool;
	fn as_int(&self)        -> &i64;
	fn as_float(&self)      -> &f64;
	fn as_string(&self)     -> &String;
	fn as_array(&self)      -> &Vec<Value>;
	fn as_func(&self)       -> &fn(&mut Value);
}

impl GetActualValues for Value {
	fn as_identifier(&self) -> &String {
		match self { Value::Identifier(ref i) => return i, _ => unreachable!() }
	}

	fn as_bool(&self) -> &bool {
		match self { Value::Boolean(ref b) => return b, _ => unreachable!() }
	}

	fn as_int(&self)        -> &i64 {
		match self { Value::Int(ref i) => return i, _ => unreachable!() }
	}

	fn as_float(&self)      -> &f64 {
		match self { Value::Float(ref f) => return f, _ => unreachable!() }
	}

	fn as_string(&self)     -> &String {
		match self { Value::String(ref s) => return s, _ => unreachable!() }
	}

	fn as_array(&self)      -> &Vec<Value> {
		match self { Value::Array(ref a) => return a, _ => unreachable!() }
	}

	fn as_func(&self)       -> &fn(&mut Value) {
		match self { Value::Func { f } => return f, _ => unreachable!() }
	}
}


fn crash(msg: String) {
	println!("{}", msg);
	exit(1);
}


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
			// Interpret the program

			for rule in node.into_inner() {
				// NOTE(pebaz): Keep the indent at 0 for the first time
				interpret(rule, indent, symtab);
			}
		}

		Rule::Identifier => {
			// Lookup the value of identifier and return that
			// e.g. Function, String, Array, etc.
			
			let key = node.as_str();

			if !symtab.contains_key(&key) {
				crash(format!("Undefined Variable: No variable named \"{}\"", key));
			}

			let return_value = symtab.get(key);
		}
		
		Rule::Function => {
			// Execute function (built-in or otherwise)

			let mut args = Value::Array(Vec::new());

			match args {
				Value::Array(ref mut arr) => arr.push(Value::Boolean(true)),
				_ => {}
			}

			println!("---------> {}", args.as_array()[0].as_bool());

			for rule in node.into_inner() {
				interpret(rule, indent + 1, symtab);
			}
		}

		Rule::Array => {
			// Return Value::Array

			for rule in node.into_inner() {
				interpret(rule, indent + 1, symtab);
			}
		}

		Rule::String => {
			// Return Value::String

			let return_value = Value::String(String::from(node.as_str()));

			/*
			println!("FOUND STRING: {}", match return_value {
				Value::String(s) => { s.as_str().to_owned() }
				_ => { "".to_owned() }
			});
			*/
		}
		
		Rule::Number => {
			// Return either Value::Int or Value::Float
		}

		Rule::Boolean => {
			// Return Value::Boolean

			let return_value = Value::Boolean(FromStr::from_str(node.as_str()).unwrap());
		}

		_ => {}
	}

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
