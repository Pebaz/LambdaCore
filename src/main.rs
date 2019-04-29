/*
Parsed 78k LOC in ~500ms.

Goals:
[✔] Expose function
[✔] Call function
[✔] Return value
[✔] Transform recursive `interpret` into iteration
[ ] Allow for multiple stack frames via struct
[ ] Quoting https://www.gnu.org/software/emacs/manual/html_node/elisp/Quoting.html

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

#[macro_use]
extern crate clap;

use std::env;
use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::fs;
use std::fmt;
use std::cmp::min;
use std::process::exit;
use std::str::FromStr;
use pest::Parser;
use pest::iterators::Pair;
use colored::*;
use clap::{App, Arg};

static LCORE_DEBUG: bool = false;

#[derive(Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;

type SymTab = HashMap<String, Value>;

#[derive(Clone)]
enum Value {
	Null,
	Identifier(String),
	Boolean(bool),
	Int(i64),
	Float(f64),
	String(String),
	Array(Vec<Value>),
	Func { f: fn(&mut Value, &mut SymTab) -> Value },

	// TODO(pebaz): Delete the old `Quote` variant
	QUOTE(Box<Value>),

	// TODO(pebaz):
	Struct { name: String, fields: Vec<Value> },
	Hash(HashMap<Value, Value>),


	// Lexical Values
	OpenFunc, CloseFunc,
	OpenBrace, CloseBrace,
	Quote, BackTick, Comma
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

	fn as_func(&self)       -> &fn(&mut Value, &mut SymTab) -> Value {
		match self { Value::Func { f } => return f, _ => unreachable!() }
	}

	fn as_value(&self)        -> &Value {
		match self { Value::QUOTE(ref q) => return &(**q), _ => unreachable!() }
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
			Value::OpenFunc       => {  write!(fm, "(")          }
			Value::CloseFunc      => {  write!(fm, ")")          }
			Value::OpenBrace      => {  write!(fm, "[")          }
			Value::CloseBrace     => {  write!(fm, "]")          }
			Value::Quote          => {  write!(fm, "'")          }
			Value::BackTick       => {  write!(fm, "`")          }
			Value::Comma          => {  write!(fm, ",")          }
			Value::Func { f }     => {  write!(fm, "Func")       }

			Value::QUOTE(i) => { write!(fm, "Actual Quote.") }

			Value::Struct { name, fields } => { write!(fm, "Struct") }
			Value::Hash(h)        => { write!(fm, "Hash")        }
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
				//print!(", ");
				print!(" ");
			}
		}
		print!("]");
	}

	fn print_func(v: &fn(&mut Value, &mut SymTab) -> Value, repr: bool) {
		print!("<Func at {:p}>", v);
	}

	fn print_quote(v: &Box<Value>, repr: bool) {

		// TODO(pebaz): Choose which one is better:

		// 1.
		print!("(quote ");
		print_value(v, repr);
		print!(")");

		// 2.
		// print!("'");
		// print_value(v, repr);
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
			Value::Identifier(v) => {
				// Will only get here if value was quoted
				// CHECK ON THIS LATER, not sure any more
				//print!("'{}", v);
				print!("{}", v);
			}
			Value::QUOTE(v) => { print_quote(v, true) }

			Value::OpenFunc => print!("("),
			Value::CloseFunc => print!(")"),
			_ => { }
		}
	}

	let args = args.as_array();

	if args.len() > 1 { crash(format!("Can only print 1 value at a time right now.")); }

	let value = args.iter().next().unwrap();

	print_value(value, false);
}


fn lcore_prin(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	lcore_print_value(args);
	Value::Null
}


fn lcore_print(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	lcore_print_value(args);
	println!("");
	Value::Null
}


fn lcore_add(args: &mut Value, symbol_table: &mut SymTab) -> Value {
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

fn lcore_quit(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	exit(0);
}

fn lcore_set(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	let mut args = args.as_array().iter();

	let var = args.next().expect("Not enough arguments on call to \"set\": 0/2");
	let value = args.next().expect("Not enough arguments on call to \"set\": 1/2");

	/*if let Value::Identifier(v) = var {
		symbol_table.insert(v.clone().to_string(), value.clone());
	}*/

	match var {
		// Identifier
		Value::Identifier(v) => { symbol_table.insert(v.clone().to_string(), value.clone()); }

		// Quoted Identifier
		Value::QUOTE(v) => {
			let mystr = v.as_identifier();
			symbol_table.insert(mystr.clone().to_string(), value.clone());
		}
		_ => ()
	}

	Value::Null
}

fn lcore_loop(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	let mut args = args.as_array().iter();

	let quote = args.next().expect("Not enough arguments on call to \"loop\": 0/3");
	let iters = args.next().expect("Not enough arguments on call to \"loop\": 1/3");
	let body = args.next().expect("Not enough arguments on call to \"loop\": 2/3");

	for i in 0..*iters.as_int() {
		let mut loop_body = match body.as_value().clone() {
			Value::Array(v) => VecDeque::from_iter(v),
			_ => unreachable!()
		};

		if let Value::Identifier(s) = quote.as_value() {
			symbol_table.insert(s.clone().to_string(), Value::Int(i));
		}

		lcore_interpret(&mut loop_body, symbol_table);
	}

	Value::Null
}


///
/// Stuff the code to run in a list value in the symbol table. Make sure to
/// store the variables to bind at call time.
///
fn lcore_defn(args: &mut Value, symbol_table: &mut SymTab) -> Value {
	// Identifier
	// Array<Quoted(Identifier)>
	// Quoted(Array<Value>) (The code to run later)

	let mut args = args.as_array().iter();

	let name = args.next().expect("Not enough arguments on call to \"defn\": 0/3");
	let arguments = args.next().expect("Not enough arguments on call to \"defn\": 1/3");
	let body = args.next().expect("Not enough arguments on call to \"defn\": 2/3");

	let def = Value::Array(vec![
		arguments.clone(), body.as_value().clone()
	]);

	match name {
		// Identifier
		Value::Identifier(v) => { symbol_table.insert(v.clone().to_string(), def); }

		// Quoted Identifier
		Value::QUOTE(v) => {
			let mystr = v.as_identifier();
			symbol_table.insert(mystr.clone().to_string(), def);
		}

		_ => ()
	}

	Value::Null
}


///
/// Turn tokens into intermediate code.
///
/// Returns: The count of the lines of code in the file.
///
fn lcore_parse(
	node: Pair<'_, Rule>,
	//stack: &mut Vec<Value>
	stack: &mut VecDeque<Value>
) -> usize {
	let mut loc = 0;

	match node.as_rule() {
		Rule::Program => {
			for rule in node.into_inner() {
				loc += lcore_parse(rule, stack);
			}
		}

		Rule::Function => {
			stack.push_back(Value::OpenFunc);
			let mut rules = node.into_inner();

			let func = match rules.next() { 
				Some(rule) => { stack.push_back(Value::Identifier(String::from(rule.as_str()))); },
				_ => unreachable!()
			};

			for rule in rules {
				loc += lcore_parse(rule, stack);
			}
			stack.push_back(Value::CloseFunc);
		}

		Rule::Array => {
			//stack.push_back(Value::OpenBrace);

			let mut array_stack = VecDeque::new();

			for rule in node.into_inner() {
				//loc += lcore_parse(rule, stack);
				loc += lcore_parse(rule, &mut array_stack);
			}

			let mut new_array = Vec::new();
			new_array.extend(array_stack);
			stack.push_back(Value::Array(new_array));

			//stack.extend(array_stack);
			//stack.push_back(Value::CloseBrace);
		}

		Rule::Number => {
			if node.as_str().contains(".") {
				stack.push_back(Value::Float(FromStr::from_str(node.as_str()).unwrap()))
			} else {
				stack.push_back(Value::Int(FromStr::from_str(node.as_str()).unwrap()))
			}
		}


		Rule::Quote => {
			//stack.push_back(Value::Quote)

			let mut quote_stack = VecDeque::new();

			// NEED TO NEST ALL OTHER VALUES WITHIN ALL TYPES OF QUOTES :/

			for rule in node.into_inner() {
				loc += lcore_parse(rule, &mut quote_stack);
			}

			assert!(quote_stack.len() == 1);

			stack.push_back(Value::QUOTE(Box::new(quote_stack.pop_back().unwrap())));

			//let mut new_array = Vec::new();
			//new_array.extend(quote_stack);
			//stack.push_back(Value::Array(new_array));
		}


		Rule::BackTick => { stack.push_back(Value::BackTick) }
		Rule::Comma => { stack.push_back(Value::Comma) }


		Rule::Identifier => { stack.push_back(Value::Identifier(String::from(node.as_str()))) }
		Rule::String => { stack.push_back(Value::String(String::from(node.as_str()))) }
		Rule::Boolean => { stack.push_back(Value::Boolean(FromStr::from_str(node.as_str().to_lowercase().as_str()).unwrap())) }
		Rule::Null => { stack.push_back(Value::Null) }
		Rule::NewLine => { loc += 1 }
		Rule::EOI => { }  // May want to use this for module imports :D
		_ => ()
	}

	return loc;
}


///
/// Interpret a LambdaCore Program.
///
fn lcore_interpret(
	//stack: &mut Vec<Value>,
	stack: &mut VecDeque<Value>,
	symbol_table: &mut SymTab
) -> Value {
	let mut arrays: Vec<Value> = Vec::with_capacity(64);

	// NOTE(pebaz): Since a function can be called in the global scope, we need
	// a top-level array to catch any global function call return values.
	arrays.push(Value::Array(Vec::new()));

	while let Some(node) = stack.pop_front() {

		match node {
			Value::Int(ref v)        => {
				if LCORE_DEBUG { println!("Int: {}", node.as_int()); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Float(ref v)      => {
				if LCORE_DEBUG { println!("Float: {}", node.as_float()); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::String(ref v)     => {
				if LCORE_DEBUG { println!("String: {}", node.as_string()); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Identifier(ref v) => {
				if LCORE_DEBUG { println!("Identifier: {}", node.as_identifier()); }

				let length = arrays.len();

				if let Value::Array(ref mut v) = arrays[length - 1] {

					if let Some(last) = v.last_mut() {
						// Replace the quote with the current node (skipping it)
						//*last = node;
						if let Value::Quote = last {
							if LCORE_DEBUG { println!("Quoted"); }
							*last = node;
						} else {
							// Lookup the current node and push it
							if LCORE_DEBUG { println!("Normal"); }

							let key = node.as_identifier();
							if !symbol_table.contains_key(key.as_str()) {
								crash(format!("Undefined Variable: No variable named \"{}\"", key));
							}
							let length = arrays.len();
							if let Value::Array(ref mut array) = arrays[length - 1] {
								array.push(symbol_table.get(key.as_str()).unwrap().clone())
							}
						}
					} else {
							// Lookup the current node and push it
							if LCORE_DEBUG { println!("Normal"); }

							let key = node.as_identifier();
							if !symbol_table.contains_key(key.as_str()) {
								crash(format!("Undefined Variable: No variable named \"{}\"", key));
							}
							let length = arrays.len();
							if let Value::Array(ref mut array) = arrays[length - 1] {
								array.push(symbol_table.get(key.as_str()).unwrap().clone())
							}
						}
				}

				
				/*
				let key = node.as_identifier();
				if !symbol_table.contains_key(key.as_str()) {
					crash(format!("Undefined Variable: No variable named \"{}\"", key));
				}
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(symbol_table.get(key.as_str()).unwrap().clone())
				}
				*/
			}

			Value::Boolean(ref v)    => {
				if LCORE_DEBUG { println!("Boolean: {}", node.as_string()); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Null              => {
				if LCORE_DEBUG { println!("Null"); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::OpenFunc          => {
				// Call the function & store result in `arrays`
				if LCORE_DEBUG { println!("("); }
				arrays.push(Value::Array(Vec::new()));
			}

			Value::CloseFunc         => {
				if LCORE_DEBUG { println!(")"); }
				
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					let func = v.remove(0);
					let mut args = arrays.pop().unwrap();

					// IMPORTANT(pebaz): Either the func is a native function
					// or a LambdaCore function.
					//let ret = func.as_func()(&mut args, symbol_table);

					let ret = match func {
						Value::Func { f } => f(&mut args, symbol_table),

						Value::Array(a) => {
							// The argument names
							let arg_names = match &a[0] {
								Value::Array(argument_names) => { argument_names }
								_ => unreachable!()
							};

							// Bind all arguments to the given values
							if let Value::Array(ref mut v) = args {
								let mut count = 0;
								while let Some(value) = v.pop() {

									match &arg_names[count] {
										Value::QUOTE(v) => {
											symbol_table.insert(v.as_identifier().to_string(), value);
										}

										_ => unreachable!()
									}

									//symbol_table.insert(arg_names[count].as_identifier().to_string(), value);
									count += 1;
								}
							}

							// Execute the function

							// The function body
							// let def = match &a[1] {
							// 	Value::Array(definition) => { definition }
							// 	_ => unreachable!()
							// };
							//let mut body = VecDeque::from_iter(def);
							//lcore_interpret(&mut body, &mut symbol_table);

							/*if let Value::Array(def) = &a[1] {
								let mut body = VecDeque::from_iter(def.clone());
								lcore_interpret(&mut body, symbol_table)
							}*/

							let ret = match &a[1] {
								Value::Array(def) => {
									let mut body = VecDeque::from_iter(def.clone());
									lcore_interpret(&mut body, symbol_table)
								}
								_ => unreachable!()
							};

							//Value::Null
							ret
						}

						_ => Value::Null
					};
				
					let length = arrays.len();
					if let Value::Array(ref mut v) = arrays[length - 1] {
						v.push(ret)
					}
				}
			}

			Value::OpenBrace         => {
				if LCORE_DEBUG { println!("["); }
				arrays.push(Value::Array(Vec::new()));
			}

			Value::CloseBrace        => {
				if LCORE_DEBUG { println!("]"); }

				let array = arrays.pop().unwrap();

				//arrays.push(Value::Array(Vec::new()));

				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(array)
				}
			}

			Value::Quote | Value::BackTick | Value::Comma => {
				if LCORE_DEBUG { println!("{:?}", node); }
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::Array(ref v) => {
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			Value::QUOTE(ref v) => {
				let length = arrays.len();
				if let Value::Array(ref mut v) = arrays[length - 1] {
					v.push(node)
				}
			}

			// Ignored Values:
			// Value::Func
			_ => ()
		}
	}

	// Return the value from the last function to be called
	let mut last_array = arrays.pop().unwrap();
	match last_array {
		Value::Array(ref mut v) => return v.pop().unwrap(),
		_ => unreachable!()
	}
}


fn count_newlines(s: &str) -> usize {
    s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}

fn main() {
	let matches = App::new("LambdaCore")
		.version(crate_version!())
		.author(crate_authors!())
		.about("Lisp dialect written in Rust")
		.arg(Arg::with_name("file")
			.short("f")
			.long("file")
			.value_name("FILE")
			.help("The script to run")
			.required(false))
		.get_matches();

	// Get other CLI switches (not FILE yet)

	let mut code_file = matches.value_of("file");

	if let Option::None = code_file {
		println!("REPL\n(> ");
		code_file = Some("func.lcore");
	}

	let unparsed_file = fs::read_to_string(code_file.unwrap()).expect("LCORE: Error Reading File");

	let lines_of_code = count_newlines(unparsed_file.as_str()) + 1;

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	//if LCORE_DEBUG { println!("{:#?}", program); }

	let mut symbol_table: SymTab = HashMap::new();

	// Fill the symbol table with built-in functions
	symbol_table.insert(String::from("print"), Value::Func { f: lcore_print });
	symbol_table.insert(String::from("prin"), Value::Func { f: lcore_prin });
	symbol_table.insert(String::from("+"), Value::Func { f: lcore_add });
	symbol_table.insert(String::from("quit"), Value::Func { f: lcore_quit });
	symbol_table.insert(String::from("set"), Value::Func { f: lcore_set });
	symbol_table.insert(String::from("loop"), Value::Func { f: lcore_loop });
	symbol_table.insert(String::from("defn"), Value::Func { f: lcore_defn });

	// Interpret the Program
	//interpret(program, 0, &mut symbol_table);

	let mut stack = VecDeque::with_capacity(lines_of_code);
	let planned = stack.capacity();
	let loc = lcore_parse(program, &mut stack) * 2;

	println!("---------------------------------------------");
	println!("| Code Lines | Planned Stack | Actual Stack |");
	println!("| {: <10} | {: <13} | {: <12} |", lines_of_code, planned, stack.len());
	println!("---------------------------------------------\n");

	if LCORE_DEBUG {
		for item in &stack {
			println!("{:?}", item);
		}
	}

	lcore_interpret(&mut stack, &mut symbol_table);	

	if LCORE_DEBUG {
		for item in &symbol_table {
			println!("{:?}", item);
		}
	}

	// Print Single symbol
	// let a = symbol_table.remove(&mut String::from("hello-world")).unwrap();
	// lcore_print_value(&mut Value::Array(vec![a]));
}
