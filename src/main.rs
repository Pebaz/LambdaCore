/*
Parsed 78k LOC in ~500ms.

Goals:
[✔] Expose function
[✔] Call function
[✔] Return value
[ ] Transform recursive `interpret` into iteration
[ ] Allow for multiple stack frames via struct

Problem: `stress.lcore` takes 13 seconds on performant hardware.

Hypothesis: Parsing and interpreting happen within the same nested recursive
function.

To allow for an iterative approach, some limitations must be accepted:

 1. A program at its highest level consists only of a set of function calls.
 2. A function's arguments may contain values or other function calls.
 3. The parser could construct tokens like: CallFunction, EndFunction,
    OpenArray, CloseArray and push them onto a stack.
 4. The interpreter could have a massive while loop that could pop each token
    off the stack one at a time and construct nested values if needed (arrays,
	structs, etc.).
 5. Once values have been saved up, a CallFunction token will be encountered.
    This is when the saved up values will be put into an array and passed to
	the function.

This method will be advantageous for these reasons:

 1. Right now, variables cannot be set. (e.g.: `(set name "Pebaz")` will crash)
 2. Performance should be better since values should be popped off of a total
    stack rather than deeply nested within recursive function calls.
 3. Prevents a stack overflow resulting from too many iterations by keeping all
    tokens on the heap in a stack container.

Questions:

 1. How will this work when importing code?
 2. Is there really no way to allow identifiers to be passed as-is to functions
    that need them (like set, def, struct, etc.)?
 3. If so, would moving to the much less elegant stack-based approach really be
    better for this particular project?
 4. How will user-defined functions and data types work?
*/


#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate pest_derive;

use std::env;
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
	Func { f: fn(&mut Value) -> Value },
	Struct { name: String, fields: Vec<Value> },

	// Lexical Values
	OpenFunc, CloseFunc,
	OpenBrace, CloseBrace
}

impl Value {
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

	fn as_func(&self)       -> &fn(&mut Value) -> Value {
		match self { Value::Func { f } => return f, _ => unreachable!() }
	}
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
			Value::OpenFunc       => {  write!(fm, "(")          }
			Value::CloseFunc      => {  write!(fm, ")")          }
			Value::OpenBrace      => {  write!(fm, "[")          }
			Value::CloseBrace     => {  write!(fm, "]")          }

			Value::Struct { name, fields } => { write!(fm, "Struct") }
		}
	}
}



// scopes.push()
// scopes.pop()
struct Environment<'a> {
	scopes: Vec<HashMap<&'a str, Value>>
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
			// TODO(pebaz): Support user-defined functions

			let mut args = Value::Array(Vec::new());

			let mut rules = node.into_inner();

			let func = match rules.next() { 
				Some(rule) => { interpret(rule, indent + 1, symtab) },
				_ => unreachable!()
			};

			for rule in rules {
				if let Value::Array(ref mut arr) = args {
					arr.push(interpret(rule, indent + 1, symtab));
				}
			}

			return_value = func.as_func()(&mut args);
		}

		Rule::Array => {
			// Return Value::Array

			let mut values = Value::Array(Vec::new());
			for rule in node.into_inner() {
				if let Value::Array(ref mut arr) = values {
					arr.push(interpret(rule, indent + 1, symtab));
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
				return_value = Value::Float(FromStr::from_str(node.as_str()).unwrap());
			} else {
				return_value = Value::Int(FromStr::from_str(node.as_str()).unwrap());
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


fn lcore_print_value(args: &mut Value) {
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

	fn print_func(v: &fn(&mut Value) -> Value, repr: bool) {
		print!("<Func at {:p}>", v);
	}

	fn print_value(value: &Value, repr: bool) {
		match value {
			// Print, stripping out first and last double quotes `"`
			Value::String(v) => print_string(v, repr),
			Value::Boolean(v) => print_boolean(v, repr),
			Value::Int(v) => print_int(v, repr),
			Value::Float(v) => print_float(v, repr),
			Value::Array(v) => print_array(v, repr),
			Value::Func { f: v } => print_func(v, repr),
			Value::Null => print_null(),
			_ => { }
		}
	}

	let args = args.as_array();

	if args.len() > 1 { crash(format!("Can only print 1 value at a time right now.")); }

	let value = args.iter().next().unwrap();

	print_value(value, false);
}


fn lcore_prin(args: &mut Value) -> Value {
	lcore_print_value(args);
	Value::Null
}


fn lcore_print(args: &mut Value) -> Value {
	lcore_print_value(args);
	println!("");
	Value::Null
}


fn lcore_add(args: &mut Value) -> Value {
	let mut args = args.as_array().iter();
	let a = args.next().expect("Not enough arguments on call to \"add\": 0/2");
	let b = args.next().expect("Not enough arguments on call to \"add\": 1/2");
	match (a, b) {
		(Value::Int(v1), Value::Int(v2)) => {
			return Value::Int(a.as_int() + b.as_int());
		}

		(Value::Float(v1), Value::Float(v2)) => {
			return Value::Float(a.as_float() + b.as_float());
		}

		_ => unreachable!()  // Handle error
	}
}


///
/// Turn tokens into intermediate code.
///
fn lcore_parse(node: Pair<'_, Rule>, stack: &mut Vec<Value>) {
	match node.as_rule() {
		Rule::Program => {
			for rule in node.into_inner().rev() {
				lcore_parse(rule, stack);
			}
		}

		Rule::Function => {
			stack.push(Value::OpenFunc);
			let mut rules = node.into_inner();

			let func = match rules.next() { 
				Some(rule) => { stack.push(Value::Identifier(String::from(rule.as_str()))); },
				_ => unreachable!()
			};

			for rule in rules { lcore_parse(rule, stack); }
			stack.push(Value::CloseFunc);
		}

		Rule::Array => {
			stack.push(Value::OpenBrace);
			for rule in node.into_inner() { lcore_parse(rule, stack); }
			stack.push(Value::CloseBrace);
		}

		Rule::Number => {
			if node.as_str().contains(".") {
				stack.push(Value::Float(FromStr::from_str(node.as_str()).unwrap()))
			} else {
				stack.push(Value::Int(FromStr::from_str(node.as_str()).unwrap()))
			}
		}

		Rule::Identifier => { stack.push(Value::Identifier(String::from(node.as_str()))) }
		Rule::String => { stack.push(Value::String(String::from(node.as_str()))) }
		Rule::Boolean => { stack.push(Value::Boolean(FromStr::from_str(node.as_str().to_lowercase().as_str()).unwrap())) }
		Rule::Null => { stack.push(Value::Null) }
		Rule::EOI => { }  // May want to use this for module imports :D
		_ => ()
	}
}


///
/// Interpret a LambdaCore Program.
///
fn lcore_interpret(
	stack: &mut Vec<Value>,
	symbol_table: &mut HashMap<&str, Value>
) {
	let mut arrays: Vec<Value> = Vec::with_capacity(64);

	// NOTE(pebaz): Since a function can be called in the global scope, we need
	// a top-level array to catch any global function call return values.
	arrays.push(Value::Array(Vec::new()));

	while let Some(node) = stack.pop() {
		match node {
			Value::Int(ref v)        => {
				//println!("Int: {}", node.as_int());
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Float(ref v)      => {
				//println!("Float: {}", node.as_float());
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::String(ref v)     => {
				//println!("String: {}", node.as_string());
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Identifier(ref v) => {
				//println!("Identifier: {}", node.as_identifier());
				let key = node.as_identifier();
				if !symbol_table.contains_key(key.as_str()) {
					crash(format!("Undefined Variable: No variable named \"{}\"", key));
				}
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(symbol_table.get(key.as_str()).unwrap().clone())
				}
			}

			Value::Boolean(ref v)    => {
				//println!("Boolean: {}", node.as_string());
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Null              => {
				//println!("Null");
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::OpenFunc          => {
				// Call the function & store result in `arrays`
				//println!("(");

				// arrays: Vec<Value::Array>
				// func: Value::Array.pop()
				// arrays[-1].push(func(arrays.pop()))

				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					let func = v.pop().unwrap();
					let mut args = arrays.pop().unwrap();
					let ret = func.as_func()(&mut args);
					
					let length = arrays.len();
					if let Value::Array(ref mut v) = arrays[length - 1] {
						v.push(ret)
					}
				}
			}

			Value::CloseFunc         => {
				//println!(")");
				arrays.push(Value::Array(Vec::new()));
			}

			Value::OpenBrace         => {
				//println!("[");
				let array = arrays.pop().unwrap();
				arrays.push(Value::Array(Vec::new()));
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(array)
				}
			}

			Value::CloseBrace        => {
				//println!("]");
				arrays.push(Value::Array(Vec::new()));
			}


			// Ignored Values:
			// Value::Func
			// Value::Array
			_ => ()
		}
	}
}


fn main() {
	let args: Vec<String> = env::args().collect();
	let code_file = if args.len() == 1 {
		// TODO(pebaz): Implement REPL and launch it here.
		"print.lcore"
	} else {
		args[1].as_str()
	};

	let unparsed_file = fs::read_to_string(code_file).expect("LCORE: Error Reading File");

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	if LCORE_DEBUG { println!("{:#?}", program); }

	let mut symbol_table: HashMap<&str, Value> = HashMap::new();

	// Fill the symbol table with built-in functions
	symbol_table.insert("print", Value::Func { f: lcore_print });
	symbol_table.insert("prin", Value::Func { f: lcore_prin });
	symbol_table.insert("+", Value::Func { f: lcore_add });

	// Interpret the Program
	//interpret(program, 0, &mut symbol_table);

	// TODO(pebaz): Find out a good starting capacity
	let mut stack = Vec::with_capacity(512);

	lcore_parse(program, &mut stack);

	println!("-----------------");
	println!("Stack Count: {}", stack.len());

	lcore_interpret(&mut stack, &mut symbol_table);	
}
