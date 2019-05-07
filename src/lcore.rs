#![macro_use]
extern crate pest_derive;
pub extern crate pest;

use crate::builtin::*;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io::{self, Write};
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, Eq};
use std::iter::FromIterator;
use std::str::FromStr;
use std::process::exit;
use pest::Parser;
use pest::error::Error;
use pest::error::ErrorVariant::ParsingError;
use pest::iterators::Pair;


#[derive(pest_derive::Parser)]
#[grammar = "LambdaCore.pest"]
pub struct LambdaCoreParser;

type SymTab = HashMap<String, Value>;

static LCORE_DEBUG: bool = false;

#[derive(Clone)]
pub enum Value {
	Null,
	Identifier(String),
	Boolean(bool),
	Int(i64),
	Float(f64),
	String(String),
	Array(Vec<Value>),
	Func { f: fn(&mut Value, &mut Environment) -> Result<Value, LCoreError> },
	Quote(Box<Value>),
	Dict(HashMap<Value, Value>),

	// TODO(pebaz):
	Struct { name: String, fields: Vec<Value> },


	// Lexical Values
	OpenFunc, CloseFunc,
	OpenBrace, CloseBrace,
	BackTick, Comma
}

impl Value {
	pub fn as_identifier(&self) -> &String {
		match self { Value::Identifier(ref i) => return i, _ => unreachable!() }
	}

	pub fn as_bool(&self)       -> &bool {
		match self { Value::Boolean(ref b) => return b, _ => unreachable!() }
	}

	pub fn as_int(&self)        -> &i64 {
		match self { Value::Int(ref i) => return i, _ => unreachable!() }
	}

	pub fn as_float(&self)      -> &f64 {
		match self { Value::Float(ref f) => return f, _ => unreachable!() }
	}

	pub fn as_string(&self)     -> &String {
		match self { Value::String(ref s) => return s, _ => unreachable!() }
	}

	pub fn as_array(&self)      -> &Vec<Value> {
		match self { Value::Array(ref a) => return a, _ => unreachable!() }
	}

	pub fn as_func(&self)       -> &fn(&mut Value, &mut Environment) -> Result<Value, LCoreError> {
		match self { Value::Func { f } => return f, _ => unreachable!() }
	}

	pub fn as_value(&self)      -> &Value {
		match self { Value::Quote(ref q) => return &(**q), _ => unreachable!() }
	}
}

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Value::Null => Value::Null.hash(state),

			Value::Int(v) => v.hash(state),

			_ => ()
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Value) -> bool {
		return match (self, other) {

			(Value::String(a), Value::String(b)) => a == b,
			(Value::Int(a), Value::Int(b)) => a == b,
			(Value::Float(a), Value::Float(b)) => a == b,
			(Value::Boolean(a), Value::Boolean(b)) => a == b,
			(Value::Identifier(a), Value::Identifier(b)) => a == b,
			(Value::Null, Value::Null) => true,
			(Value::Quote(a), Value::Quote(b)) => a == b,
			(Value::Func { f: a }, Value::Func { f: b }) => a as *const _ == b as *const _,
			(Value::Array(a), Value::Array(b)) => a == b,

			(Value::Dict(a), Value::Dict(b)) => a == b,
			/*{
				if a.len() != b.len() { return false; }

				for key in a.keys() {
					if !b.contains_key(key) { return false; }

					if a[key] != b[key] { return false; }
				}

				true
			}*/

			_ => false,

		}
	}
}

impl Eq for Value {}

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
			Value::Quote(b)       => {  write!(fm, "'")          }
			Value::BackTick       => {  write!(fm, "`")          }
			Value::Comma          => {  write!(fm, ",")          }
			Value::Func { f }     => {  write!(fm, "Func")       }
			Value::Dict(h)        => {  write!(fm, "Dict")        }

			Value::Struct { name, fields } => { write!(fm, "Struct") }
		}
	}
}


pub struct Environment {
	scopes: Vec<SymTab>
}

impl Environment {
	pub fn new() -> Environment {
		Environment { scopes: Vec::new() }
	}

	/*fn get_iter(&mut self) -> i32 {

	}*/

	pub fn push(&mut self) {
		self.scopes.push(SymTab::new());
	}

	pub fn pop(&mut self) -> SymTab {
		self.scopes.pop().unwrap()
	}

	pub fn insert(&mut self, key: String, value: Value) {
		let scope = self.scopes.last_mut().unwrap();
		scope.insert(key, value);
	}

	pub fn contains_key(&self, name: String) -> bool {
		for scope in self.scopes.iter().rev() {
			if let Some(value) = scope.get(&name) {
				return true;
			}
		}
		false
	}

	pub fn get(&mut self, name: String) -> Option<&mut Value> {
		for scope in &mut self.scopes.iter_mut().rev() {
			if let Some(value) = scope.get_mut(&name) {
				return Some(value);
			}
		}
		None
	}
}



pub enum LCoreError {
	LambdaCoreError(String),
	IndexError(String),
	ArgumentError(String),
	NameError(String)
}



pub fn crash(msg: String) {
	println!("\n{}", msg);
	exit(1);
}



///
/// Turn tokens into intermediate code.
///
/// Returns: The count of the lines of code in the file.
///
pub fn lcore_parse(
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

			stack.push_back(Value::Quote(Box::new(quote_stack.pop_back().unwrap())));

			//let mut new_array = Vec::new();
			//new_array.extend(quote_stack);
			//stack.push_back(Value::Array(new_array));
		}

		Rule::String => {
			let mut string = String::from(node.as_str());
			string = string[1 .. string.len() - 1].to_string();
			stack.push_back(Value::String(string))
		}

		Rule::BackTick => { stack.push_back(Value::BackTick) }
		Rule::Comma => { stack.push_back(Value::Comma) }
		Rule::Identifier => { stack.push_back(Value::Identifier(String::from(node.as_str()))) }
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
pub fn lcore_interpret(
	//stack: &mut Vec<Value>,
	stack: &mut VecDeque<Value>,
	symbol_table: &mut Environment
) -> Result<Value, LCoreError> {
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

						/*if let Value::Quote = last {
							if LCORE_DEBUG { println!("Quoted"); }
							*last = node;*/

						if let Value::Quote(b) = last {
							println!("Quoted");
							*last = node;

						} else {
							// Lookup the current node and push it
							if LCORE_DEBUG { println!("Normal"); }

							let key = node.as_identifier();
							if !symbol_table.contains_key(key.as_str().to_string()) {
								//crash(format!("NameError: Cannot lookup name: \"{}\"", key));

								return Err(LCoreError::NameError(format!("NameError: Cannot lookup name: \"{}\"", key)));
							}
							let length = arrays.len();
							if let Value::Array(ref mut array) = arrays[length - 1] {
								array.push(symbol_table.get(key.as_str().to_string()).unwrap().clone())
							}
						}
					} else {
							// Lookup the current node and push it
							if LCORE_DEBUG { println!("Normal"); }

							let key = node.as_identifier();
							if !symbol_table.contains_key(key.as_str().to_string()) {
								//crash(format!("NameError: Cannot lookup name: \"{}\"", key));

								return Err(LCoreError::NameError(format!("NameError: Cannot lookup name: \"{}\"", key)));
							}
							let length = arrays.len();
							if let Value::Array(ref mut array) = arrays[length - 1] {
								array.push(symbol_table.get(key.as_str().to_string()).unwrap().clone())
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

					let ret = match func {
						Value::Func { f } => f(&mut args, symbol_table),

						Value::Array(a) => {
							// The argument names
							let arg_names = match &a[0] {
								Value::Array(argument_names) => { argument_names }
								_ => unreachable!()
							};

							// TODO(pebaz): In order to do Tail-Call Optimization,
							// it is necessary to remove the next code line.
							// This will allow the function to reuse names
							// (and therefore storage) from previous call.

							// Push a new scope
							symbol_table.push();

							// Bind all arguments to the given values
							if let Value::Array(ref mut v) = args {
								let mut count = 0;
								while let Some(value) = v.pop() {

									match &arg_names[count] {
										Value::Quote(v) => {
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

							// Reclaim all old variables
							symbol_table.pop();

							//Value::Null
							ret
						}

						_ => Ok(Value::Null)
					};
				
					let length = arrays.len();
					if let Value::Array(ref mut v) = arrays[length - 1] {
						//v.push(ret.ok().unwrap())

						v.push(match ret {
							Ok(i) => i,
							Err(err) => {
								match err {
									LCoreError::LambdaCoreError(s) => println!("{}", s),
									LCoreError::IndexError(s) => println!("{}", s),
									LCoreError::ArgumentError(s) => println!("{}", s),
									LCoreError::NameError(s) => println!("{}", s)
								}
								exit(1);
							}
						});
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

			Value::BackTick | Value::Comma => {
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

			Value::Quote(ref v) => {
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
		Value::Array(ref mut v) => {
			match v.pop() {
				Some(e) => return Ok(e),
				None => return Ok(Value::Null)
			}
		}
		_ => unreachable!()
	}
}


pub fn count_newlines(s: &str) -> usize {
    s.as_bytes().iter().filter(|&&c| c == b'\n').count()
}


/*
pub fn lcore_repl222() {
	let mut symbol_table = Environment::new();
	symbol_table.push();
	import_builtins(&mut symbol_table);

	let mut again = false;
	let mut input = String::new();

	while again {
		print!("(> ");
		std::io::stdout().flush().unwrap();

		let mut line = String::new();

		io::stdin().read_line(&mut line).unwrap();
		input.push_str(&line);

		match LambdaCoreParser::parse(Rule::Program, &input) {
			Ok(i) => {
				let mut stack = VecDeque::new();
				lcore_parse(i.next().unwrap(), &mut stack);
				lcore_interpret(&mut stack, &mut symbol_table);
				again = false;
			}

			Err(err) => {
				again = true;
			}
		}
	}
}
*/

pub fn lcore_repl() {
	print!("LambdaCore Programming Language v");
	println!(env!("CARGO_PKG_VERSION"));
	println!("Type CTRL+Z or (quit) to exit.");

	let mut symbol_table = Environment::new();
	symbol_table.push();
	import_builtins(&mut symbol_table);

	loop {
		print!("(> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		let mut add_space = false;

		loop {
			let mut line = String::new();
			io::stdin().read_line(&mut line).unwrap();
			if add_space {
				input.push_str(" ");
				add_space = false;
			}
			input.push_str(&line.trim());

			match LambdaCoreParser::parse(Rule::Program, &input) {
				Ok(mut i) => {
					let mut stack = VecDeque::new();
					lcore_parse(i.next().unwrap(), &mut stack);

					// lcore_interpret(&mut stack, &mut symbol_table);

					if let Err(err) = lcore_interpret(&mut stack, &mut symbol_table) {
						match err {
							LCoreError::LambdaCoreError(s) => println!("{}", s),
							LCoreError::IndexError(s) => println!("{}", s),
							LCoreError::ArgumentError(s) => println!("{}", s),
							LCoreError::NameError(s) => println!("{}", s)
						}
					}

					break;
				}

				Err(err) => {
					match err.variant {
						ParsingError {positives, negatives} => {

							// NOTE(pebaz): Something has gone wrong and Pest
							// cannot parse the input.
							if positives.contains(&Rule::EOI) {
								println!("Enter only one form per line");
								break;
							}

							// NOTE(pebaz): This is needed to not read
							// additional lines from stdin if the user just
							// presses the `enter` key.
							else if positives.contains(&Rule::Program) {
								break;
							}

							// Add whitespace because lines are concatenated
							else {
								add_space = true;
							}
						}

						_ => ()
					}

					print!(" > ");
					std::io::stdout().flush().unwrap();
				}
			}
		}
	}
}









#[test]
pub fn test_tests() {
	assert_eq!(4, 4);
}
