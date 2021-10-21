extern crate clap;
use clap::{App, Arg};

use std::{
    collections::VecDeque,
    convert::TryInto,
    fs::File,
    io::{self, stdin, Read, Write},
    vec::IntoIter,
};
enum Token {
    Increment,
    Decrement,
    MoveUp,
    MoveDown,
    Print,
    Read,
    LoopOpen,
    LoopClose,
}
enum ParseElement {
    Increment,
    Decrement,
    MoveUp,
    MoveDown,
    Print,
    Read,
    Block(Vec<ParseElement>),
}
enum BytecodeElement {
    ChangeValue(i32),
    MovePointer(i32),
    Print,
    Read,
    PositiveJump(usize),
    NegativeJump(usize),
}

fn main() {
    let path = get_path();
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let tokens = tokenize(content);
    let parse_tree = parse_root(tokens);
    let bytecode = tree_walker(parse_tree);
    interpret(bytecode);
}

fn parse_root(tokens: Vec<Token>) -> Vec<ParseElement> {
    let mut parse_elements: Vec<ParseElement> = Vec::new();
    let mut iterator = tokens.into_iter();
    while let Some(token) = iterator.next() {
        parse_elements.push(match token {
            Token::Increment => ParseElement::Increment,
            Token::Decrement => ParseElement::Decrement,
            Token::MoveUp => ParseElement::MoveUp,
            Token::MoveDown => ParseElement::MoveDown,
            Token::Print => ParseElement::Print,
            Token::Read => ParseElement::Read,
            Token::LoopOpen => parse_block(&mut iterator),
            Token::LoopClose => panic!("unexpected ]"),
        });
    }
    parse_elements
}

fn parse_block(iterator: &mut IntoIter<Token>) -> ParseElement {
    let mut block_elements: Vec<ParseElement> = Vec::new();
    while let Some(token) = iterator.next() {
        match token {
            Token::Increment => block_elements.push(ParseElement::Increment),
            Token::Decrement => block_elements.push(ParseElement::Decrement),
            Token::MoveUp => block_elements.push(ParseElement::MoveUp),
            Token::MoveDown => block_elements.push(ParseElement::MoveDown),
            Token::Print => block_elements.push(ParseElement::Print),
            Token::Read => block_elements.push(ParseElement::Read),
            Token::LoopOpen => block_elements.push(parse_block(iterator)),
            Token::LoopClose => return ParseElement::Block(block_elements),
        };
    }
    panic!("missing ]");
}
fn tokenize(string: String) -> Vec<Token> {
    return string.chars().map(map_token).flatten().collect();
}
fn map_token(character: char) -> Option<Token> {
    match character {
        '+' => Some(Token::Increment),
        '-' => Some(Token::Decrement),
        '>' => Some(Token::MoveUp),
        '<' => Some(Token::MoveDown),
        '.' => Some(Token::Print),
        ',' => Some(Token::Read),
        '[' => Some(Token::LoopOpen),
        ']' => Some(Token::LoopClose),
        _ => None,
    }
}
fn tree_walker(tree: Vec<ParseElement>) -> Vec<BytecodeElement> {
    let mut bytecode: Vec<BytecodeElement> = Vec::new();
    for element in tree {
        match element {
            ParseElement::Increment => bytecode.push(BytecodeElement::ChangeValue(1)),
            ParseElement::Decrement => bytecode.push(BytecodeElement::ChangeValue(-1)),
            ParseElement::MoveUp => bytecode.push(BytecodeElement::MovePointer(1)),
            ParseElement::MoveDown => bytecode.push(BytecodeElement::MovePointer(-1)),
            ParseElement::Print => bytecode.push(BytecodeElement::Print),
            ParseElement::Read => bytecode.push(BytecodeElement::Read),
            ParseElement::Block(block_commands) => {
                bytecode = walk_subtree(block_commands, bytecode, 0);
            }
        }
    }
    bytecode
}
fn walk_subtree(
    block: Vec<ParseElement>,
    mut bytecode: Vec<BytecodeElement>,
    depth: usize,
) -> Vec<BytecodeElement> {
    let block_start = bytecode.len();
    for element in block {
        match element {
            ParseElement::Increment => bytecode.push(BytecodeElement::ChangeValue(1)),
            ParseElement::Decrement => bytecode.push(BytecodeElement::ChangeValue(-1)),
            ParseElement::MoveUp => bytecode.push(BytecodeElement::MovePointer(1)),
            ParseElement::MoveDown => bytecode.push(BytecodeElement::MovePointer(-1)),
            ParseElement::Print => bytecode.push(BytecodeElement::Print),
            ParseElement::Read => bytecode.push(BytecodeElement::Read),
            ParseElement::Block(block_commands) => {
                bytecode = walk_subtree(block_commands, bytecode, depth + 1);
            }
        }
    }
    bytecode.push(BytecodeElement::NegativeJump(block_start + depth));
    bytecode.insert(
        block_start,
        BytecodeElement::PositiveJump(bytecode.len() + depth),
    );
    bytecode
}
fn interpret(bytecode: Vec<BytecodeElement>) {
    let mut mem = [0; 30000];
    let mut pointer: usize = 0;
    let mut program_counter = 0;
    let mut input_buffer = VecDeque::<u8>::new();
    while let Some(instruction) = bytecode.get(program_counter) {
        match instruction {
            BytecodeElement::ChangeValue(val) => mem[pointer] = change_value(mem[pointer], *val),
            BytecodeElement::MovePointer(val) => pointer = move_pointer(pointer, *val),
            BytecodeElement::Print => print(mem[pointer]),
            BytecodeElement::Read => {
                if input_buffer.is_empty() {
                    input_buffer.extend(read());
                }
                mem[pointer] = input_buffer.pop_front().unwrap();
            }
            BytecodeElement::PositiveJump(instruction_index) => {
                program_counter = positive_jump(mem[pointer], program_counter, *instruction_index)
            }
            BytecodeElement::NegativeJump(instruction_index) => {
                program_counter = negative_jump(mem[pointer], program_counter, *instruction_index)
            }
        }
        program_counter += 1;
    }
}
fn print(value: u8) {
    print!("{}", value as char);
    let _ = io::stdout().flush();
}
fn read() -> Vec<u8> {
    loop {
        let mut input = String::new();
        let _ = stdin().read_line(&mut input);
        if input.is_ascii() {
            return input.as_bytes().to_vec();
        }
    }
}
fn positive_jump(value: u8, position: usize, destination: usize) -> usize {
    match value {
        0 => destination,
        _ => position,
    }
}
fn negative_jump(value: u8, position: usize, destination: usize) -> usize {
    match value {
        0 => position,
        _ => destination,
    }
}
fn move_pointer(current: usize, change: i32) -> usize {
    let current_i32: i32 = current.try_into().unwrap();
    let new: i32 = current_i32 + change;
    match new {
        30000 => 0,
        -1 => 29999,
        0..=29999 => new.try_into().unwrap(),
        _ => panic!("Pointer moved by more than one"),
    }
}
fn change_value(current: u8, change: i32) -> u8 {
    let current_i32: i32 = current.try_into().unwrap();
    let new: i32 = current_i32 + change;
    match new {
        256 => 0,
        -1 => 255,
        0..=255 => new.try_into().unwrap(),
        _ => panic!("Value changed by more than one"),
    }
}

fn get_path() -> String {
    let matches = App::new("Brainfuck interpreter")
        .arg(
            Arg::with_name("file")
                .help("Sets the input file to use")
                .short("f")
                .long("file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    return String::from(matches.value_of("file").unwrap());
}
