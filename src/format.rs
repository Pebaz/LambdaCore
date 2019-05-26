use crate::lcore::pest::Parser;
use crate::lcore::LambdaCoreParser;
use crate::lcore::*;
use pest::iterators::Pair;

pub fn lcore_format_code(code: String) -> String {
    let program = LambdaCoreParser::parse(Rule::Program, &code)
        .expect("LCORE: Failed To Parse")
        .next()
        .unwrap();

    println!("{:#?}", program);

    let mut buffer = String::from("");
    lcore_format_rule(program, &mut buffer);

    println!("\n\n{}\n\n", code);

    buffer
}

pub fn lcore_highlight_code(code: String) -> String {
    code
}

pub fn lcore_format_rule(node: Pair<'_, Rule>, buffer: &mut String) {
    fn push_str(s: &mut String, string: String, nest: i32, indent: i32) {
        for i in 0..indent {
            s.push_str("    ");
        }

        s.push_str(&string);
        s.push_str(" ");
    }

    match node.as_rule() {
        Rule::Program => {
			// let my stuff = node.into_inner().peekable();  // Let's you look ahead, useful for spacebars!

            for rule in node.into_inner() {
                lcore_format_rule(rule, buffer);
            }
        }

        Rule::Function => {
            buffer.push_str("(");

            for rule in node.into_inner() {
                lcore_format_rule(rule, buffer);
            }

            buffer.push_str(")");
        }

		Rule::Number => buffer.push_str(node.as_str()),
		Rule::Identifier => buffer.push_str(node.as_str()),
		Rule::Quote => buffer.push_str("'"),
		Rule::String => buffer.push_str(node.as_str()),

		/*

        Rule::Array => {
            // stack.push_back(Value::OpenBrace);

            let mut array_stack = VecDeque::new();

            for rule in node.into_inner() {
                // loc += lcore_parse(rule, stack);
                loc += lcore_parse(rule, &mut array_stack);
            }

            let mut new_array = Vec::new();
            new_array.extend(array_stack);
            stack.push_back(Value::Array(new_array));

            // stack.extend(array_stack);
            // stack.push_back(Value::CloseBrace);
        }

        

        

        Rule::BackTick => stack.push_back(Value::BackTick),
        Rule::Comma => stack.push_back(Value::Comma),
        
        Rule::Boolean => stack.push_back(Value::Boolean(
            FromStr::from_str(node.as_str().to_lowercase().as_str()).unwrap(),
        )),
        Rule::Null => stack.push_back(Value::Null),
		*/
        Rule::NewLine => buffer.push_str("\n"),
        Rule::EOI => {}
        _ => (),
    }
}
