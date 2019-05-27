use std::process::Command;

fn run(file: String) -> String {
    let output = Command::new("target/debug/lambda_core.exe")
        .arg("-f")
        .arg(file)
        .output()
        .unwrap();

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_hello_world() {
    let stdout = run("examples/hello-world.lcore".to_string());
    assert_eq!(stdout, "Hello World!\n".to_string());
}

#[test]
fn test_len() {
    let stdout = run("examples/len.lcore".to_string());
    assert_eq!(stdout, "3\n".to_string());
}

#[test]
fn test_fib() {
    let stdout = run("examples/fib.lcore".to_string());
    assert_eq!(stdout, "(fib 40) = 63245986\n".to_string());
}

#[test]
fn test_add() {
    let stdout = run("examples/add.lcore".to_string());
    let expect = "3\n\
        3.2\n\
        6\n\
        Hello World\n\
        15\n".to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_break() {
    let stdout = run("examples/break.lcore".to_string());
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
        Y: 2\n".to_string();
    assert_eq!(stdout, expect);
}

#[test]
fn test_import() {
    let stdout = run("examples/import.lcore".to_string());
    let expect = "You are importing the `add` function!\n\
        10\n".to_string();
    assert_eq!(stdout, expect);
}

