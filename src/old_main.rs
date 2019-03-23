/*
https://pest.rs/
https://docs.rs/pest/2.1.0/pest/
https://docs.rs/pest_derive/2.1.0/pest_derive/
https://pest.rs/book/examples/ini.html
*/


extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fs;
use pest::Parser;


#[derive(Parser)]
#[grammar = "ini.pest"]
pub struct INIParser;


fn main() {
	let unparsed_file = fs::read_to_string("config.ini").expect("Cannot read file");

	let file = INIParser::parse(Rule::file, &unparsed_file)
		.expect("Failed to parse") // Unwrap the parse result :D
		.next().unwrap(); // Get and unwrap the `file` rule; never failes

	let mut properties: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

	let mut current_section_name = "";

	for line in file.into_inner() {
		match line.as_rule() {
			Rule::section => {
				let mut inner_rules = line.into_inner(); // { name }
				current_section_name = inner_rules.next().unwrap().as_str();
			}

			Rule::property => {
				let mut inner_rules = line.into_inner(); // { name ~ "= " ~ value }

				let name: &str = inner_rules.next().unwrap().as_str();
				let value: &str = inner_rules.next().unwrap().as_str();

				// Insert an empty inner hash map if the outer hash map hasn't
                // seen this section name before.
				let section = properties.entry(current_section_name).or_default();
				section.insert(name, value);
			}

			Rule::EOI => (),

			_ => unreachable!(), // Catch all
		}
	}

	println!("{:#?}", properties);
}
