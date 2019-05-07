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
extern crate clap;

mod lcore;
mod builtin;

use crate::lcore::*;
use crate::builtin::*;
use crate::lcore::pest::Parser;
use std::env;
use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::fs;
use std::fmt;
use std::cmp::min;
use std::str::FromStr;
use colored::*;
use clap::{App, Arg};


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
		lcore_repl();
	}

	lcore_import_file(code_file.unwrap().to_string());

	/*
	let unparsed_file = fs::read_to_string(code_file.unwrap()).expect("LCORE: Error Reading File");

	// This can be a concurrent task
	let lines_of_code = count_newlines(unparsed_file.as_str()) + 1;

	let program = LambdaCoreParser::parse(Rule::Program, &unparsed_file)
		.expect("LCORE: Failed To Parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `Program` rule; never failes

	//if LCORE_DEBUG { println!("{:#?}", program); }

	//let mut symbol_table: SymTab = HashMap::new();
	let mut symbol_table = Environment::new();
	symbol_table.push();

	import_builtins(&mut symbol_table);

	// Interpret the Program
	//interpret(program, 0, &mut symbol_table);

	let mut stack = VecDeque::with_capacity(lines_of_code);

	let planned = stack.capacity();
	let loc = lcore_parse(program, &mut stack);

	if LCORE_DEBUG {
		println!("---------------------------------------------");
		println!("| Code Lines | Planned Stack | Actual Stack |");
		println!("| {: <10} | {: <13} | {: <12} |", lines_of_code, planned, stack.len());
		println!("---------------------------------------------\n");
	}

	if let Err(err) = lcore_interpret(&mut stack, &mut symbol_table) {
		match err {
			LCoreError::LambdaCoreError(s) => println!("{}", s),
			LCoreError::IndexError(s) => println!("{}", s),
			LCoreError::ArgumentError(s) => println!("{}", s),
			LCoreError::NameError(s) => println!("{}", s)
		}
	}

	// Print Single symbol
	// let a = symbol_table.remove(&mut String::from("hello-world")).unwrap();
	// lcore_print_value(&mut Value::Array(vec![a]));
	*/
}
