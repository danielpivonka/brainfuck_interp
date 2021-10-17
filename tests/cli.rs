use std::{
    fs::File,
    io::{BufReader, Error, Read},
};

use assert_cmd::Command;

fn read_file_to_string(path: &str) -> Result<String, Error> {
    let file = File::open(path)?;
    let mut result_string = String::new();

    let mut buf_reader = BufReader::new(file);
    buf_reader.read_to_string(&mut result_string)?;

    Ok(result_string)
}

#[test]
fn hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck_interpreter")?;
    cmd.env("stdout", "hello")
        .arg("--file")
        .arg("./test_files/hello-world.txt")
        .assert()
        .stdout("Hello World!\n");

    Ok(())
}

#[test]
fn zero_to_ninety_nine() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck_interpreter")?;
    let reference_output = read_file_to_string("./test_outputs/print-0-to-99-result.txt")?;

    cmd.env("stdout", "numbers")
        .arg("--file")
        .arg("./test_files/print-0-to-99.txt")
        .assert()
        .stdout(reference_output);

    Ok(())
}

#[test]
fn cat() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("brainfuck_interpreter")?;
    let reference_output = read_file_to_string("./test_outputs/lorem-ipsum.txt")?;

    cmd.env("stdout", "cat")
        .arg("--file")
        .arg("./test_files/cat.txt")
        .write_stdin(reference_output.as_str())
        .assert()
        .stdout(reference_output);

    Ok(())
}
