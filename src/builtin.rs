use std::collections::{HashMap, VecDeque};

use crate::lcore::*;
use std::process::exit;
use std::iter::FromIterator;

pub fn lcore_print_value(args: &mut Value) -> Result<Value, LCoreError> {
	fn print_string(v: &String, repr: bool) {
		if repr {
			//print!("{}", v);
			print!("\"{}\"", v);
		} else {
			//print!("{}", &v[1 .. v.len() - 1]);
			print!("{}", v);
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

	fn print_func(v: &fn(&mut Value, &mut Environment) -> Result<Value, LCoreError>, repr: bool) {
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

	fn print_dict(v: &HashMap<Value, Value>, repr: bool) {
		print!("{{ ");
		//print!("{:?}", v);
		let length = v.len();
		let mut count = 0;

		for (key, value) in v {
			print_value(key, true);
			print!(": ");
			print_value(value, true);

			count += 1;
			if count < length {
				print!(", ");
				//print!(" ");
			}
		}

		print!(" }}");
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
				// TODO
				// Will only get here if value was quoted
				// CHECK ON THIS LATER, not sure any more
				//print!("'{}", v);
				print!("{}", v);
			}
			Value::Quote(v) => { print_quote(v, true) }
			Value::Dict(v) => { print_dict(v, repr) }
			Value::OpenFunc => print!("("),
			Value::CloseFunc => print!(")"),
			_ => { }
		}
	}

	let args = args.as_array();

	if args.len() > 1 {
		//crash(format!("Can only print 1 value at a time right now."));
		return Err(
			LCoreError::ArgumentError(
				format!("Can only print 1 value at a time right now.")));
	}

	let value = args.iter().next().unwrap();

	print_value(value, false);

	Ok(Value::Null)
}


pub fn lcore_prin(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	lcore_print_value(args);
	Ok(Value::Null)
}


pub fn lcore_print(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	lcore_print_value(args);
	println!("");
	Ok(Value::Null)
}


pub fn lcore_add(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	let mut args = args.as_array().iter();
	let a = args.next().expect("Not enough arguments on call to \"add\": 0/2");
	let b = args.next().expect("Not enough arguments on call to \"add\": 1/2");
	match (a, b) {
		(Value::Int(v1), Value::Int(v2)) => {
			return Ok(Value::Int(a.as_int() + b.as_int()));
		}

		(Value::Float(v1), Value::Float(v2)) => {
			return Ok(Value::Float(a.as_float() + b.as_float()));
		}

		_ => unreachable!()  // Handle error
	}
}

pub fn lcore_quit(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	exit(0);
}

pub fn lcore_set(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	let mut args = args.as_array().iter();

	let var = args.next().expect("Not enough arguments on call to \"set\": 0/2");
	let value = args.next().expect("Not enough arguments on call to \"set\": 1/2");

	/*if let Value::Identifier(v) = var {
		symbol_table.insert(v.clone().to_string(), value.clone());
	}*/

	match var {
		// Identifier
		Value::Identifier(v) => {
			symbol_table.insert(v.clone().to_string(), value.clone());
		}

		// Quoted Identifier
		Value::Quote(v) => {
			let mystr = v.as_identifier();
			symbol_table.insert(mystr.clone().to_string(), value.clone());
		}

		_ => ()
	}

	Ok(Value::Null)
}

pub fn lcore_loop(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
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

	Ok(Value::Null)
}


///
/// Stuff the code to run in a list value in the symbol table. Make sure to
/// store the variables to bind at call time.
///
pub fn lcore_defn(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
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
		Value::Identifier(v) => {
			symbol_table.insert(v.clone().to_string(), def);
		}

		// Quoted Identifier
		Value::Quote(v) => {
			let mystr = v.as_identifier();
			symbol_table.insert(mystr.clone().to_string(), def);
		}

		_ => ()
	}

	Ok(Value::Null)
}


pub fn lcore_get(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	let mut args = args.as_array().iter();

	let obj = args.next().expect(
		"Not enough arguments on call to \"get\": 0/2");
	let mut key = args.next().expect(
		"Not enough arguments on call to \"get\": 1/2");

	if let Value::Quote(q) = key {
		key = q;
	}

	match obj {
		Value::Array(v) => {
			if let Value::Int(index) = key {
				if *index > v.len() as i64 {
					/*crash(format!(
						"Index out of bounds: got {} but len is {}", index,
						v.len()));*/

					return Err(
						LCoreError::ArgumentError(
							format!(
								"Index out of bounds: got {} but len is {}",
								index,
								v.len())));

				} else {
					let len = v.len() as i64;
					let mut idx = *index % len;
					if idx < 0 { idx += len }
					return Ok(v[idx as usize].clone());
				}
			} else {
				//crash(format!("Cannot index Array with {:?}", key));
				return Err(
					LCoreError::ArgumentError(
						format!("Cannot index Array with {:?}", key)));
			}
		}

		Value::Dict(v) => {
			match key {
				Value::Identifier(a) => {
					return Ok(v.get(&Value::String(a.to_string()))
						.expect(&format!("No identifier key named: \"{}\"", a))
						.clone());
				}

				Value::String(a) => {
					return Ok(v.get(key)
						.expect(&format!("No string key named: \"{}\"", a))
						.clone());
				}

				Value::Int(a) => {
					return Ok(v.get(key)
						.expect(&format!("No int key named: {}", a))
						.clone());
				}

				Value::Float(a) => {
					return Ok(v.get(key)
						.expect(&format!("No float key named: {}", a))
						.clone());
				}

				Value::Boolean(a) => {
					return Ok(v.get(key)
						.expect(&format!("No boolean key named: {}", a))
						.clone());
				}

				_ => unreachable!()
			}
		}

		Value::String(v) => {
			match key {
				Value::Int(a) => {
					println!("*******************************************");
				}

				_ => unreachable!()
			}
		}

		_ => ()
	}

	Ok(Value::Null)
}


pub fn lcore_dict(args: &mut Value, symbol_table: &mut Environment) -> Result<Value, LCoreError> {
	let args = args.as_array();
	let mut args_iter = args.iter();

	if args.len() % 2 != 0 {
		//crash(format!("Odd number of arguments passed to \"dict\""));
		return Err(LCoreError::ArgumentError(
			format!("Odd number of arguments passed to \"dict\"")));
	}

	let mut dict: HashMap<Value, Value> = HashMap::new();

	for i in 0..args.len() / 2 {
		let key = args_iter.next().expect(&format!("NO KEY {}", i));
		let value = args_iter.next().expect(&format!("NO VALUE {}", i));

		 if let Value::Quote(q) = key {
			 if let Value::Identifier(s) = *q.clone() {
		 		dict.insert(Value::String(s), value.clone());
			 }
		} else {
			dict.insert(key.clone(), value.clone());
		}
	}

	//dict.insert(Value::String(String::from("first name")), Value::Int(24));
	//dict.insert(Value::String(String::from("last name")), Value::String(String::from("Wallace")));

	Ok(Value::Dict(dict))
}


pub fn import_builtins(symbol_table: &mut Environment) {
	symbol_table.insert(String::from("print"), Value::Func { f: lcore_print });
	symbol_table.insert(String::from("prin"), Value::Func  { f: lcore_prin });
	symbol_table.insert(String::from("+"), Value::Func     { f: lcore_add });
	symbol_table.insert(String::from("quit"), Value::Func  { f: lcore_quit });
	symbol_table.insert(String::from("exit"), Value::Func  { f: lcore_quit });
	symbol_table.insert(String::from("set"), Value::Func   { f: lcore_set });
	symbol_table.insert(String::from("loop"), Value::Func  { f: lcore_loop });
	symbol_table.insert(String::from("defn"), Value::Func  { f: lcore_defn });
	symbol_table.insert(String::from("get"), Value::Func  { f: lcore_get });
	symbol_table.insert(String::from("dict"), Value::Func  { f: lcore_dict });
}
