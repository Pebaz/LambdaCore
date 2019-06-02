use std::process::Command;


fn run_file(file: String) -> String {
    let target = if cfg!(debug_assertions) {
        "target/debug/lambda_core"
    } else {
        "target/debug/lambda_core"
    };

    let output = Command::new(target)
        .arg("-f")
        .arg(file)
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_add() {
    let stdout = run_file("examples/add.lcore".to_string());
    let expect = "3\n\
                  3.2\n\
                  6\n\
                  Hello World\n\
                  15\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_break() {
    let stdout = run_file("examples/break.lcore".to_string());
    let expect = "X: 0\n\
                  Y: 0\n\
                  Y: 1\n\
                  Y: 2\n\
                  X: 1\n\
                  Y: 0\n\
                  Y: 1\n\
                  Y: 2\n\
                  X: 2\n\
                  Y: 0\n\
                  Y: 1\n\
                  Y: 2\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_comment() {
    let stdout = run_file("examples/comment.lcore".to_string());
    let expect = "Line Comment\n\
                  Block Comment\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_compare() {
    let stdout = run_file("examples/compare.lcore".to_string());
    let expect = "1      =    1       : True\n\
                  'a'    =    'a'     : True\n\
                  3.14   =    3.14    : True\n\
                  Null   =    Null    : True\n\
                  True   =    True    : True\n\
                  'a     =    'a      : True\n\
                  [1]    =    [1]     : True\n\
                  {a: 1} =    {a: 1}  : True\n\
                  \n\
                  1      !=   1      : False\n\
                  'a'    !=   'a'    : False\n\
                  3.14   !=   3.14   : False\n\
                  Null   !=   Null   : False\n\
                  True   !=   True   : False\n\
                  'a     !=   'a     : False\n\
                  [1]    !=   [1]    : False\n\
                  {a: 1} !=   {a: 1} : False\n\
                  \n\
                  True   or   True   : True\n\
                  True   and  True   : True\n\
                  Not    True        : False\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_dict() {
    let stdout = run_file("examples/dict.lcore".to_string());
    let expect1 = "{ \"first name\": \"David\", \"last name\": \"Wallace\", \"age\": 41 }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    let expect2 = "{ \"last name\": \"Wallace\", \"first name\": \"David\", \"age\": 41 }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    let expect3 = "{ \"age\": 41, \"last name\": \"Wallace\", \"first name\": \"David\" }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    let expect4 = "{ \"age\": 41, \"first name\": \"David\", \"last name\": \"Wallace\" }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    let expect5 = "{ \"first name\": \"David\", \"age\": 41, \"last name\": \"Wallace\" }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    let expect6 = "{ \"last name\": \"Wallace\", \"age\": 41, \"first name\": \"David\" }\n\
        David\n\
        41\n\
        41\n\
        David\n\
        Michael\n\
        True Value\n\
        Int Value\n\
        Float Value\n".to_string();

    assert!(
        (stdout == expect1)
            | (stdout == expect2)
            | (stdout == expect3)
            | (stdout == expect4)
            | (stdout == expect5)
            | (stdout == expect6)
    );
}

#[test]
fn test_error() {
    let stdout = run_file("examples/error.lcore".to_string());
    assert_eq!(
        stdout,
        "ArgumentError: Odd number of arguments passed to \"dict\"\n"
            .to_string()
    );
}

#[test]
fn test_eval() {
    let stdout = run_file("examples/eval.lcore".to_string());
    let expect = "World\n\
                  [\"Hello\" Null]\n\
                  2\n"
    .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_fib() {
    let stdout = run_file("examples/fib.lcore".to_string());
    assert_eq!(stdout, "(fib 40) = 63245986\n".to_string());
}

#[test]
fn test_func() {
    let stdout = run_file("examples/func.lcore".to_string());
    let expect = "Hello Pebaz!\n\
                  Hello 0!\n\
                  Hello 1!\n\
                  Hello 2!\n\
                  Hello 88!\n\
                  6\n"
    .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_get() {
    let stdout = run_file("examples/get.lcore".to_string());
    assert_eq!(stdout, "2\n".to_string());
}

#[test]
fn test_hello_world() {
    let stdout = run_file("examples/hello-world.lcore".to_string());
    assert_eq!(stdout, "Hello World!\n".to_string());
}

#[test]
fn test_if() {
    let stdout = run_file("examples/if.lcore".to_string());
    let expect = "It's True!\n\
                  One does in fact equal one!\n\
                  It's False!\n\
                  14\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_import() {
    let stdout = run_file("examples/import.lcore".to_string());
    let expect = "You are importing the `add` function!\n\
                  10\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_len() {
    let stdout = run_file("examples/len.lcore".to_string());
    assert_eq!(stdout, "3\n".to_string());
}

#[test]
fn test_loop() {
    let stdout = run_file("examples/loop.lcore".to_string());
    assert_eq!(stdout, "0\n1\n2\n".to_string());
}

#[test]
fn test_math() {
    let stdout = run_file("examples/math.lcore".to_string());
    let expect = "--------------------\n\
                  Int\n\
                  6\n\
                  -2\n\
                  8\n\
                  0\n\
                  16\n\
                  \n\
                  --------------------\n\
                  Float\n\
                  6\n\
                  -2\n\
                  8\n\
                  0.5\n\
                  16\n\
                  \n\
                  --------------------\n\
                  String\n\
                  Hello World!\n\
                  HiHiHi\n\
                  \n\
                  --------------------\n\
                  Array\n\
                  [1 2 3 4]\n\
                  [11 11]\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_order() {
    let stdout = run_file("examples/order.lcore".to_string());
    let expect = "First\n\
                  Second\n\
                  Null\n\
                  3\n\
                  [1 2 \"Three\"]\n"
        .to_string();
    assert_eq!(stdout, expect);
}

//#[test]
#[allow(dead_code)]
fn test_print() {
    let stdout = run_file("examples/order.lcore".to_string());

    // TODO(pebaz): Use a regex for this:
    let expect = "String: Lambda Core version 0.1.0\n\
                  Boolean: True\n\
                  Integer: 11\n\
                  Float: 3.14\n\
                  Null: Null\n\
                  Array: [1 2 [3 4]]\n\
                  __repr__ String: [\"This Should Be Quoted\"]\n\
                  Function: <Func at 0x1e3958a5d78>\n\
                  Return Value: 11\n\
                  hello\\nworld!\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_quote() {
    let stdout = run_file("examples/quote.lcore".to_string());
    let expect = "3\n\
                  quoted-thing: (quote [1 2 3])\n\
                  (quote [( print \"Hello World!\" )])\n"
        .to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_ret() {
    let stdout = run_file("examples/ret.lcore".to_string());
    assert_eq!(stdout, "-2\n6\n-2\n".to_string());
}

#[test]
fn test_sel() {
    let stdout = run_file("examples/sel.lcore".to_string());
    assert_eq!(stdout, "It's Three!\n55\n".to_string());
}

#[test]
fn test_stdlib() {
    let stdout = run_file("examples/stdlib.lcore".to_string());
    assert_eq!(stdout, "256\n".to_string());
}

#[test]
fn test_swap() {
    let stdout = run_file("examples/swap.lcore".to_string());
    let expect = "Swapping Nest Level: 1\n\
        Before: 3\n\
        After:  4\n\
        \n\
        Swapping Nest Level: 2\n\
        Before: { \"inner\": \"Nah.\" }\n\
        After:  { \"inner\": \"Huzzah!\" }\n\
        \n\
        Swapping Nest Level: 2\n\
        Before: [\"ZERO\"]\n\
        After:  [\"ONE\"]\n\
        \n\
        Swapping Nest Level: 3\n\
        Before: { \"inner2\": { \"inner3\": [\"SO MUCH INNER\" { \"like-so-much-inner\": \"FAILURE\" }] } }\n\
        After:  { \"inner2\": { \"inner3\": [\"SO MUCH INNER\" { \"like-so-much-inner\": \"VICTORY\" }] } }\n".to_string();
    assert_eq!(stdout, expect);
}
