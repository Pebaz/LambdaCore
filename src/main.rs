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
[ ] Call function
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

static LCORE_DEBUG: bool = false;

#[derive(Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;

#[derive(Clone)]
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

	let mut return_value = Value::Null;

	if LCORE_DEBUG {
		if format!("{:#?}", node.as_rule()) != "Program" {
			println!(
				"{}{} -> {}",
				if indent > 0 { "    ".repeat(indent) } else { String::from("") },
				format!("{:#?}", node.as_rule()).cyan(),
				format!("{}", node.as_str()).green(),
			);
		}
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

			return_value = symtab.get(key).unwrap().clone();
		}
		
		Rule::Function => {
			// Execute function (built-in or otherwise)


			let mut args = Value::Array(Vec::new());

			/*
			match args {
				Value::Array(ref mut arr) => arr.push(Value::Boolean(true)),
				_ => {}
			}
			*/

			//println!("---------> {}", args.as_array()[0].as_bool());

			let mut rules = node.into_inner();

			let func = match rules.next() { 
				Some(rule) => { interpret(rule, indent + 1, symtab) },
				_ => unreachable!()
			};

			//for rule in node.into_inner() {
			for rule in rules {
				match args {
					Value::Array(ref mut arr) => arr.push(interpret(rule, indent + 1, symtab)),
					_ => {}
				}
			}

			// println!("{:?}", args.as_array());
			// let arguments = args.as_array();
			// println!("COUNT: {}", arguments.len());

			func.as_func()(&mut args);

			/*
			match args {
				Value::Array(ref mut arr) => println!("---------> {}", arr[0].as_identifier()),
				_ => {}
			}
			*/
		}

		Rule::Array => {
			// Return Value::Array

			// for rule in node.into_inner() {
			// 	interpret(rule, indent + 1, symtab);
			// }


			let mut values = Value::Array(Vec::new());
			for rule in node.into_inner() {
				match values {
					Value::Array(ref mut arr) => arr.push(interpret(rule, indent + 1, symtab)),
					_ => {}
				}
			}

			return_value = values;
		}

		Rule::String => {
			// Return Value::String
			return_value = Value::String(String::from(node.as_str()));
		}
		
		Rule::Number => {
			// Return either Value::Int or Value::Float
			if node.as_str().contains(".") {
				return_value = Value::Float(FromStr::from_str(node.as_str()).unwrap())
			} else {
				return_value = Value::Int(FromStr::from_str(node.as_str()).unwrap())
			}
		}

		Rule::Boolean => {
			// Return Value::Boolean

			return_value = Value::Boolean(FromStr::from_str(node.as_str().to_lowercase().as_str()).unwrap());
		}

		Rule::Null => {
			return_value = Value::Null;
		}

		_ => {}
	}

	return_value
}


fn lcore_print(args: &mut Value) {
	fn print_string(v: &String, repr: bool) {
		if repr {
			print!("{}", v);
		} else {
			print!("{}", &v[1 .. v.len() - 1]);
		}
	}

	fn print_boolean(v: &bool, repr: bool) {
 		print!("{}", if *v { "True" } else { "False" });
	}

	fn print_int(v: &i64, repr: bool) {
		print!("{}", v);
	}

	fn print_float(v: &f64, repr: bool) {
		print!("{}", v);
	}

	fn print_null() {
		print!("Null");
	}

	fn print_array(v: &Vec<Value>, repr: bool) {
		let length = v.len();
		let mut count = 0;
		print!("[");
		for value in v {

			print_value(value, true);

			count += 1;
			if count < length {
				print!(", ");
			}
		}
		print!("]");
	}

	fn print_value(value: &Value, repr: bool) {
		match value {
			// Print, stripping out first and last double quotes `"`
			Value::String(v) => print_string(v, repr),
			Value::Boolean(v) => print_boolean(v, repr),
			Value::Int(v) => print_int(v, repr),
			Value::Float(v) => print_float(v, repr),
			Value::Array(v) => print_array(v, repr),
			Value::Null => print_null(),
			_ => { }
		}
	}


	let args = args.as_array();

	if args.len() > 1 { crash(format!("Can only print 1 value at a time right now.")); }

	let mut value = args.iter().next().unwrap();

	print_value(value, false);
	println!("");
}


fn lcore_prin(args: &mut Value) {
	fn print_string(v: &String, repr: bool) {
		if repr {
			print!("{}", v);
		} else {
			print!("{}", &v[1 .. v.len() - 1]);
		}
	}

	fn print_boolean(v: &bool, repr: bool) {
 		print!("{}", if *v { "True" } else { "False" });
	}

	fn print_int(v: &i64, repr: bool) {
		print!("{}", v);
	}

	fn print_float(v: &f64, repr: bool) {
		print!("{}", v);
	}

	fn print_null() {
		print!("Null");
	}

	fn print_array(v: &Vec<Value>, repr: bool) {
		let length = v.len();
		let mut count = 0;
		print!("[");
		for value in v {

			print_value(value, true);

			count += 1;
			if count < length {
				print!(", ");
			}
		}
		print!("]");
	}

	fn print_value(value: &Value, repr: bool) {
		match value {
			// Print, stripping out first and last double quotes `"`
			Value::String(v) => print_string(v, repr),
			Value::Boolean(v) => print_boolean(v, repr),
			Value::Int(v) => print_int(v, repr),
			Value::Float(v) => print_float(v, repr),
			Value::Array(v) => print_array(v, repr),
			Value::Null => print_null(),
			_ => { }
		}
	}


	let args = args.as_array();

	if args.len() > 1 { crash(format!("Can only print 1 value at a time right now.")); }

	let mut value = args.iter().next().unwrap();

	print_value(value, false);
}

fn main() {
	let unparsed_file = fs::read_to_string("print.lcore").expect("LCORE: Error Reading File");

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	if LCORE_DEBUG { println!("{:#?}", program); }

	let mut symbol_table: HashMap<&str, Value> = HashMap::new();

	// Fill the symbol table with built-in functions
	symbol_table.insert("print", Value::Func { f: lcore_print });
	symbol_table.insert("prin", Value::Func { f: lcore_prin });

	// Interpret the Program
	interpret(program, 0, &mut symbol_table);
}
