#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

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
