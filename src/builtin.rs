use std::collections::{HashMap, VecDeque};

use crate::lcore::*;
use std::io::{self, Write};
use std::iter::FromIterator;
use std::process::exit;

pub fn lcore_print_value(args: &mut Value) -> Result<Value, LCoreError> {
    fn print_string(v: &String, repr: bool) {
        if repr {
            // print!("{}", v);
            // print!("\"{}\"", v);
            print!("\"");
            io::stdout().write(v.as_bytes()).ok();
            print!("\"");
        } else {
            // print!("{}", &v[1 .. v.len() - 1]);
            // print!("{}", v);
            io::stdout().write(v.as_bytes()).ok();
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
                // print!(", ");
                print!(" ");
            }
        }
        print!("]");
    }

    fn print_func(
        v: &fn(&mut Value, &mut Environment) -> Result<Value, LCoreError>,
        repr: bool,
    ) {
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
        // print!("{:?}", v);
        let length = v.len();
        let mut count = 0;

        for (key, value) in v {
            print_value(key, true);
            print!(": ");
            print_value(value, true);

            count += 1;
            if count < length {
                print!(", ");
                // print!(" ");
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
                // print!("'{}", v);
                print!("{}", v);
            }
            Value::Quote(v) => print_quote(v, true),
            Value::Dict(v) => print_dict(v, repr),
            Value::OpenFunc => print!("("),
            Value::CloseFunc => print!(")"),
            _ => {}
        }
    }

    let args = args.as_array();

    if args.len() > 1 {
        // crash(format!("Can only print 1 value at a time right now."));
        return Err(LCoreError::ArgumentError(format!(
            "Can only print 1 value at a time right now."
        )));
    }

    let value = args.iter().next().unwrap();

    print_value(value, false);

    Ok(Value::Null)
}

pub fn lcore_prin(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    lcore_print_value(args).ok();
    Ok(Value::Null)
}

pub fn lcore_print(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    lcore_print_value(args).ok();
    println!("");
    Ok(Value::Null)
}

pub fn lcore_quit(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    exit(0);
}

pub fn lcore_set(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();

    let var = args
        .next()
        .expect("Not enough arguments on call to \"set\": 0/2");

    let value = args
        .next()
        .expect("Not enough arguments on call to \"set\": 1/2");

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

        _ => (),
    }

    Ok(Value::Null)
}

pub fn lcore_loop(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    symbol_table.push();
    let mut args = args.as_array().iter();

    let quote = args
        .next()
        .expect("Not enough arguments on call to \"loop\": 0/3");
    let iters = args
        .next()
        .expect("Not enough arguments on call to \"loop\": 1/3");
    let body = args
        .next()
        .expect("Not enough arguments on call to \"loop\": 2/3");

    for i in 0..*iters.as_int() {
        let mut loop_body = match body.as_value().clone() {
            Value::Array(v) => VecDeque::from_iter(v),
            _ => unreachable!(),
        };

        if let Value::Identifier(s) = quote.as_value() {
            symbol_table.insert(s.clone().to_string(), Value::Int(i));
        }

        if let Err(err) = lcore_interpret(&mut loop_body, symbol_table) {
            match err {
                LCoreError::LambdaCoreError(..) => return Err(err),
                LCoreError::IndexError(..) => return Err(err),
                LCoreError::ArgumentError(..) => return Err(err),
                LCoreError::NameError(..) => return Err(err),
                // LCoreError::ReturnError(v) => println!("CANNOT RETURN FROM
                // FOR LOOP"),
                LCoreError::ReturnError => return Err(err),
                LCoreError::BreakError => break,
            }
        }
    }

    symbol_table.pop();
    Ok(Value::Null)
}

/// Stuff the code to run in a list value in the symbol table. Make sure to
/// store the variables to bind at call time.
pub fn lcore_defn(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    // Identifier
    // Array<Quoted(Identifier)>
    // Quoted(Array<Value>) (The code to run later)

    let mut args = args.as_array().iter();

    let name = args
        .next()
        .expect("Not enough arguments on call to \"defn\": 0/3");
    let arguments = args
        .next()
        .expect("Not enough arguments on call to \"defn\": 1/3");
    let body = args
        .next()
        .expect("Not enough arguments on call to \"defn\": 2/3");

    let def = Value::Array(vec![arguments.clone(), body.as_value().clone()]);

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

        _ => (),
    }

    Ok(Value::Null)
}

pub fn lcore_get(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();

    let obj = args
        .next()
        .expect("Not enough arguments on call to \"get\": 0/2");
    let mut key = args
        .next()
        .expect("Not enough arguments on call to \"get\": 1/2");

    if let Value::Quote(q) = key {
        key = q;
    }

    match obj {
        Value::Array(v) => {
            if let Value::Int(index) = key {
                if *index > v.len() as i64 {
                    // crash(format!(
                    // "Index out of bounds: got {} but len is {}", index,
                    // v.len()));

                    return Err(LCoreError::ArgumentError(format!(
                        "Index out of bounds: got {} but len is {}",
                        index,
                        v.len()
                    )));
                } else {
                    let len = v.len() as i64;
                    let mut idx = *index % len;
                    if idx < 0 {
                        idx += len
                    }
                    return Ok(v[idx as usize].clone());
                }
            } else {
                // crash(format!("Cannot index Array with {:?}", key));
                return Err(LCoreError::ArgumentError(format!(
                    "Cannot index Array with {:?}",
                    key
                )));
            }
        }

        Value::Dict(v) => match key {
            Value::Identifier(a) => {
                return Ok(v
                    .get(&Value::String(a.to_string()))
                    .expect(&format!("No identifier key named: \"{}\"", a))
                    .clone());
            }

            Value::String(a) => {
                return Ok(v
                    .get(key)
                    .expect(&format!("No string key named: \"{}\"", a))
                    .clone());
            }

            Value::Int(a) => {
                return Ok(v
                    .get(key)
                    .expect(&format!("No int key named: {}", a))
                    .clone());
            }

            Value::Float(a) => {
                return Ok(v
                    .get(key)
                    .expect(&format!("No float key named: {}", a))
                    .clone());
            }

            Value::Boolean(a) => {
                return Ok(v
                    .get(key)
                    .expect(&format!("No boolean key named: {}", a))
                    .clone());
            }

            _ => unreachable!(),
        },

        Value::String(v) => match key {
            Value::Int(a) => {
                println!("*******************************************");
            }

            _ => unreachable!(),
        },

        _ => (),
    }

    Ok(Value::Null)
}

pub fn lcore_dict(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let args = args.as_array();
    let mut args_iter = args.iter();

    if args.len() % 2 != 0 {
        // crash(format!("Odd number of arguments passed to \"dict\""));
        return Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Odd number of arguments passed to \"dict\""
        )));
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

    // dict.insert(Value::String(String::from("first name")), Value::Int(24));
    // dict.insert(Value::String(String::from("last name")),
    // Value::String(String::from("Wallace")));

    Ok(Value::Dict(dict))
}

pub fn lcore_import(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let filename = match args.next() {
        Some(e) => e,
        None => {
            return Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Not enough arguments on call to \"import\": 0/1"
        )))
        }
    };

    if let Value::String(file) = filename {
        symbol_table.extend(lcore_import_file(file.to_string()));
    }

    Ok(Value::Null)
}

pub fn lcore_swap(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let obj_id = args.next().unwrap().as_value().as_identifier();
    let index = args.next().unwrap();
    let value = args.next().unwrap();

    // TODO(pebaz): The `index` is a quoted list of values to index by:
    // a[b][c][d][e]

    // println!("{}, {:?}, {:?}", obj_id, index, value);

    if let Some(obj) = symbol_table.get(obj_id.to_string()) {
        let mut current_obj = obj;

        let indexers = index.as_value().as_array();
        for i in 0..indexers.len() - 1 {
            // for indexer in index.as_value().as_array() {
            let indexer = &indexers[i];

            match current_obj {
                Value::Dict(ref mut v) => {
                    // current_obj = v[indexer]
                    if let Value::Identifier(s) = indexer {
                        current_obj =
                            v.get_mut(&Value::String(s.to_string())).unwrap();
                    } else {
                        current_obj = v.get_mut(&indexer).unwrap();
                    }
                }

                Value::Array(ref mut v) => {
                    // current_obj = v[indexer]

                    if let Value::Int(i) = indexer {
                        if *i > v.len() as i64 {
                            return Err(LCoreError::ArgumentError(format!(
                                "Index out of bounds: got {} but len is {}",
                                i,
                                v.len()
                            )));
                        } else {
                            let len = v.len() as i64;
                            let mut idx = i % len;
                            if idx < 0 {
                                idx += len
                            }
                            current_obj = v.get_mut(idx as usize).unwrap();
                        }
                    } else {
                        return Err(LCoreError::IndexError(
                            "IndexError: Cannot index array with non-int"
                                .to_string(),
                        ));
                    }
                }

                _ => unreachable!(),
            }
        }

        let indexer = &indexers[indexers.len() - 1];
        match current_obj {
            Value::Dict(ref mut v) => {
                if let Value::Identifier(s) = indexer {
                    *v.get_mut(&Value::String(s.to_string())).unwrap() =
                        value.clone();
                } else {
                    *v.get_mut(&indexer).unwrap() = value.clone();
                }
            }

            Value::Array(ref mut v) => {
                if let Value::Int(i) = indexer {
                    if *i > v.len() as i64 {
                        return Err(LCoreError::ArgumentError(format!(
                            "Index out of bounds: got {} but len is {}",
                            i,
                            v.len()
                        )));
                    } else {
                        let len = v.len() as i64;
                        let mut idx = i % len;
                        if idx < 0 {
                            idx += len
                        }
                        *v.get_mut(idx as usize).unwrap() = value.clone();
                    }
                } else {
                    return Err(LCoreError::IndexError(
                        "IndexError: Cannot index array with non-int"
                            .to_string(),
                    ));
                }
            }

            _ => unreachable!(),
        }

        // lcore_print_value(
        // 	&mut Value::Array(vec![
        // 		symbol_table.get(obj_id.to_string()).unwrap().clone()
        // 	])
        // );
    }

    Ok(Value::Null)
}

pub fn lcore_len(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let arg = match args.next() {
        Some(e) => e,
        None => {
            return Err(LCoreError::ArgumentError(format!(
                "ArgumentError: Not enough arguments on call to \"len\": 0/1"
            )))
        }
    };

    return match arg {
        Value::Array(v) => Ok(Value::Int(v.len() as i64)),
        Value::Dict(v) => Ok(Value::Int(v.len() as i64)),
        Value::String(v) => Ok(Value::Int(v.len() as i64)),
        Value::Quote(v) => Ok(Value::Int(1)),
        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: {:?} has no length",
            arg
        ))),
    };
}

pub fn lcore_equals(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
        (Value::Int(a), Value::Int(b)) => Ok(Value::Boolean(a == b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a == b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
        (Value::Identifier(a), Value::Identifier(b)) => {
            Ok(Value::Boolean(a == b))
        }
        (Value::Dict(a), Value::Dict(b)) => Ok(Value::Boolean(a == b)),
        (Value::Array(a), Value::Array(b)) => Ok(Value::Boolean(a == b)),
        (Value::Quote(a), Value::Quote(b)) => lcore_equals(
            &mut Value::Array(vec![*a.clone(), *b.clone()]),
            symbol_table,
        ),

        (Value::Func { f: a }, Value::Func { f: b }) => {
            // TODO(pebaz): Allow comparing builtin funcs?
            let addr_a = format!("{:p}", &*a);
            let addr_b = format!("{:p}", &*b);
            println!("HERE: {}, {}", addr_a, addr_b);
            Ok(Value::Boolean(addr_a == addr_b))
        }

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Type mismatch ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_not_equals(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Null, Value::Null) => Ok(Value::Boolean(false)),
        (Value::Int(a), Value::Int(b)) => Ok(Value::Boolean(a != b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a != b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a != b)),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a != b)),
        (Value::Identifier(a), Value::Identifier(b)) => {
            Ok(Value::Boolean(a != b))
        }
        (Value::Dict(a), Value::Dict(b)) => Ok(Value::Boolean(a != b)),
        (Value::Array(a), Value::Array(b)) => Ok(Value::Boolean(a != b)),
        (Value::Quote(a), Value::Quote(b)) => lcore_not_equals(
            &mut Value::Array(vec![*a.clone(), *b.clone()]),
            symbol_table,
        ),

        (Value::Func { f: a }, Value::Func { f: b }) => {
            // TODO(pebaz): Allow comparing builtin funcs?
            let addr_a = format!("{:p}", &*a);
            let addr_b = format!("{:p}", &*b);
            println!("HERE: {}, {}", addr_a, addr_b);
            Ok(Value::Boolean(addr_a != addr_b))
        }

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Type mismatch ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_less_than(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Boolean(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
        (Value::String(a), Value::String(b)) => {
            Ok(Value::Boolean(a.len() < b.len()))
        }
        (Value::Identifier(a), Value::Identifier(b)) => {
            Ok(Value::Boolean(a.len() < b.len()))
        }
        (Value::Dict(a), Value::Dict(b)) => {
            Ok(Value::Boolean(a.len() < b.len()))
        }
        (Value::Array(a), Value::Array(b)) => {
            Ok(Value::Boolean(a.len() < b.len()))
        }
        (Value::Quote(a), Value::Quote(b)) => lcore_less_than(
            &mut Value::Array(vec![*a.clone(), *b.clone()]),
            symbol_table,
        ),

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid Argument Types ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_logical_or(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a | b)),

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Not booleans ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_logical_and(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a & b)),

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Not booleans ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_logical_not(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();

    if let Value::Boolean(b) = a {
        return Ok(Value::Boolean(!b));
    } else {
        return Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Not a boolean ({:?})",
            a
        )));
    }
}

pub fn lcore_to_str(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    Ok(Value::String(String::from("LambdaCore String!")))
}

pub fn lcore_add(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::String(a), Value::String(b)) => {
            let mut result = a.clone();
            result.push_str(b);
            Ok(Value::String(result.to_string()))
        }
        (Value::Array(a), Value::Array(b)) => {
            let mut result = a.clone();
            result.extend(b.iter().cloned());
            Ok(Value::Array(result))
        }

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid operands ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_sub(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid operands ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_mul(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::String(a), Value::Int(b)) => {
            let mut result = a.clone();
            for i in 0..*b - 1 {
                result.push_str(a);
            }
            Ok(Value::String(result.to_string()))
        }
        (Value::Array(a), Value::Int(b)) => {
            let aa = a.clone();
            let mut result = a.clone();
            for i in 0..*b - 1 {
                result.extend(aa.iter().cloned());
            }
            Ok(Value::Array(result))
        }

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid operands ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_div(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid operands ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_exponent(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let mut args = args.as_array().iter();
    let a = args.next().unwrap();
    let b = args.next().unwrap();

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            if *b >= 0 {
                Ok(Value::Int(i64::pow(*a, *b as u32)))
            } else {
                Err(LCoreError::ArgumentError(format!(
                    "ArgumentError: Negative exponent ({})",
                    b
                )))
            }
        }
        (Value::Float(a), Value::Float(b)) => {
            Ok(Value::Float(f64::powf(*a, *b)))
        }

        _ => Err(LCoreError::ArgumentError(format!(
            "ArgumentError: Invalid operands ({:?} and {:?})",
            a, b
        ))),
    }
}

pub fn lcore_if(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let args = args.as_array();
    let mut vecargs = args.iter();
    let condition = vecargs.next().unwrap();
    let block_true = vecargs.next().unwrap();

    let block_false = if args.len() > 2 {
        vecargs.next().unwrap()
    } else {
        &Value::Null
    };

    if *condition.as_bool() {
        let element = block_true.as_value();
        let result = lcore_interpret_array(element, symbol_table);
        // let mut result = result.ok().unwrap();

        let mut result = match result {
            Err(err) => return Err(err),
            Ok(yeah) => yeah,
        };

        // TODO(pebaz): Check for errors on the result

        if let Value::Array(ref mut r) = result {
            let ret = r.pop().unwrap();
            // lcore_print_value(&mut Value::Array(vec![ret]));
            // println!("  <--");
            return Ok(ret);
        }
    } else {
        if let Value::Null = block_false {
        } else {
            let element = block_false.as_value();
            let result = lcore_interpret_array(element, symbol_table);
            let mut result = result.ok().unwrap();

            if let Value::Array(ref mut r) = result {
                let ret = r.pop().unwrap();
                return Ok(ret);
            }
        }
    }

    Ok(Value::Null)
}

pub fn lcore_sel(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let args = args.as_array();
    let mut vecargs = args.iter();
    let compare = vecargs.next().unwrap();

    while let (Some(value), Some(code)) = (vecargs.next(), vecargs.next()) {
        // NOTE(pebaz): Check for `'default` block
        if let Value::Quote(q) = value {
            if let Value::Identifier(s) = value.as_value() {
                if s == "default" {
                    let result =
                        lcore_interpret_array(code.as_value(), symbol_table);
                    /*if let Err(..) = result {
                        return result;
                    }
                    break;*/
                    match result {
                        Err(..) => return result,
                        Ok(mut unwraped_result) => {
                            if let Value::Array(ref mut r) = unwraped_result {
                                let ret = r.pop().unwrap();
                                return Ok(ret);
                            }
                        }
                    }
                }
            }
        }

        let res = lcore_equals(
            &mut Value::Array(vec![compare.clone(), value.clone()]),
            symbol_table,
        );
        if let Ok(res) = res {
            if *res.as_bool() {
                let result =
                    lcore_interpret_array(code.as_value(), symbol_table);
                /*if let Err(..) = result {
                    return result;
                }
                break;*/
                
                match result {
                    Err(..) => return result,
                    Ok(mut unwraped_result) => {
                        if let Value::Array(ref mut r) = unwraped_result {
                            let ret = r.pop().unwrap();
                            return Ok(ret);
                        }
                    }
                }
            }
        }
    }

    Ok(Value::Null)
}

pub fn lcore_return(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    let args = args.as_array();
    let mut vecargs = args.iter();
    let value = vecargs.next().unwrap();
    symbol_table.push_ret(value.clone());
    // Ok(Value::Null)
    // Err(LCoreError::LambdaCoreError(format!("BREAK")))
    // LCoreError::Return(value.clone())
    LCoreError::Return()
}

pub fn lcore_break(
    args: &mut Value,
    symbol_table: &mut Environment,
) -> Result<Value, LCoreError> {
    LCoreError::Break()
}

pub fn import_builtins(symbol_table: &mut Environment) {
    symbol_table.insert("print".to_string(), Value::Func { f: lcore_print });
    symbol_table.insert("prin".to_string(), Value::Func { f: lcore_prin });
    symbol_table.insert("quit".to_string(), Value::Func { f: lcore_quit });
    symbol_table.insert("exit".to_string(), Value::Func { f: lcore_quit });
    symbol_table.insert("set".to_string(), Value::Func { f: lcore_set });
    symbol_table.insert("loop".to_string(), Value::Func { f: lcore_loop });
    symbol_table.insert("defn".to_string(), Value::Func { f: lcore_defn });
    symbol_table.insert("get".to_string(), Value::Func { f: lcore_get });
    symbol_table.insert("dict".to_string(), Value::Func { f: lcore_dict });
    symbol_table.insert("len".to_string(), Value::Func { f: lcore_len });
    symbol_table
        .insert(String::from("import"), Value::Func { f: lcore_import });
    symbol_table.insert(String::from("swap"), Value::Func { f: lcore_swap });

    symbol_table.insert("to-str".to_string(), Value::Func { f: lcore_to_str });
    symbol_table.insert("=".to_string(), Value::Func { f: lcore_equals });
    symbol_table.insert(
        "!=".to_string(),
        Value::Func {
            f: lcore_not_equals,
        },
    );
    symbol_table.insert(
        "or".to_string(),
        Value::Func {
            f: lcore_logical_or,
        },
    );
    symbol_table.insert(
        "and".to_string(),
        Value::Func {
            f: lcore_logical_and,
        },
    );
    symbol_table.insert(
        "not".to_string(),
        Value::Func {
            f: lcore_logical_not,
        },
    );
    symbol_table.insert("+".to_string(), Value::Func { f: lcore_add });
    symbol_table.insert("-".to_string(), Value::Func { f: lcore_sub });
    symbol_table.insert("*".to_string(), Value::Func { f: lcore_mul });
    symbol_table.insert("/".to_string(), Value::Func { f: lcore_div });
    symbol_table.insert("**".to_string(), Value::Func { f: lcore_exponent });
    symbol_table.insert("if".to_string(), Value::Func { f: lcore_if });
    symbol_table.insert("sel".to_string(), Value::Func { f: lcore_sel });
    symbol_table.insert("ret".to_string(), Value::Func { f: lcore_return });
    symbol_table.insert("break".to_string(), Value::Func { f: lcore_break });
    symbol_table.insert("<".to_string(), Value::Func { f: lcore_less_than });
}
