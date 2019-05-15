// Parsed 78k LOC in ~500ms.
//
// Goals:
// [✔] Expose function
// [✔] Call function
// [✔] Return value
// [✔] Transform recursive `interpret` into iteration
// [ ] Allow for multiple stack frames via struct
// [ ] Quoting https://www.gnu.org/software/emacs/manual/html_node/elisp/Quoting.html
//
// Problem: `stress.lcore` takes 13 seconds on performant hardware.
//
// Hypothesis: Parsing and interpreting happen within the same nested recursive
// function.
//
// To allow for an iterative approach, some limitations must be accepted:
//
// 1. A program at its highest level consists only of a set of function calls.
// 2. A function's arguments may contain values or other function calls.
// 3. The parser could construct tokens like: CallFunction, EndFunction,
// OpenArray, CloseArray and push them onto a stack.
// 4. The interpreter could have a massive while loop that could pop each token
// off the stack one at a time and construct nested values if needed (arrays,
// structs, etc.).
// 5. Once values have been saved up, a CallFunction token will be encountered.
// This is when the saved up values will be put into an array and passed to
// the function.
//
// This method will be advantageous for these reasons:
//
// 1. Right now, variables cannot be set. (e.g.: `(set name "Pebaz")` will
// crash) 2. Performance should be better since values should be popped off of
// a total stack rather than deeply nested within recursive function calls.
// 3. Prevents a stack overflow resulting from too many iterations by keeping
// all tokens on the heap in a stack container.
//
// Questions:
//
// 1. How will this work when importing code?
// 2. Is there really no way to allow identifiers to be passed as-is to
// functions that need them (like set, def, struct, etc.)?
// 3. If so, would moving to the much less elegant stack-based approach really
// be better for this particular project?
// 4. How will user-defined functions and data types work?
//

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate clap;

mod builtin;
mod lcore;

use crate::builtin::*;
use crate::lcore::pest::Parser;
use crate::lcore::*;
use clap::{App, Arg};
use colored::*;
use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs;
use std::iter::FromIterator;
use std::str::FromStr;

fn main() {
    let matches = App::new("LambdaCore")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Lisp dialect written in Rust")
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .value_name("CODE")
                .help("Run a string of code")
                .required(false),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("The script to run")
                .required(false),
        )
        .get_matches();

    // Get other CLI switches (not FILE yet)

    let code_str = matches.value_of("code");
    let code_file = matches.value_of("file");

    match (code_file, code_str) {
        (None, None) => lcore_repl(),
        (None, Some(code)) => lcore_execute_string(code.to_string()),
        (Some(file), None) => {
            let _ = lcore_import_file(file.to_string());
        }
        _ => (),
    }
}
