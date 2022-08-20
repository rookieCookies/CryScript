#![allow(dead_code)]

pub mod exceptions;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod variables;

use std::{fmt::Display, fs::File, io::Read, rc::Rc, time::Instant};

use colored::Colorize;
use exceptions::{lexer_exceptions::InvalidAnnotation, Exception};
use include_dir::{include_dir, Dir};
use interpreter::{
    context::Context,
    instructions::{Instruction, InstructionType},
};
use parser::{data::Data, Parser};
use variables::{Variables};

use self::lexer::Lexer;

const STD_DIR: Dir = include_dir!("std_lib/");
const STD_FILES: [&str; 3] = ["std_rand", "std_math", "std_file"];

pub fn run(root_file_path: &str) -> u128 {
    let mut variables = Variables::new();
    
    let mut file_data = String::new();
    match File::open(root_file_path) {
        Ok(v) => v,
        Err(_) => Exception::new("Err: Unable to find startup fine".red().bold().to_string()).run()
    }
    .read_to_string(&mut file_data)
    .unwrap();
    // file_data.push('\0');
    let mut context = Context::new_root(
        Rc::new(FileData {
            data: file_data,
            path: root_file_path.to_string(),
        }),
        &mut variables
    );
    match Context::import_data(
        &mut context,
        Rc::new(FileData::new(
            STD_DIR
                .get_file("std.cry")
                .unwrap()
                .contents_utf8()
                .unwrap()
                .to_string(),
            "std".to_string(),
        )),
    ) {
        Ok(_) => {}
        Err(v) => v.run(),
    };
    println!(
        "{} {}{}",
        "Running".bright_green(),
        root_file_path.bright_green().bold(),
        "...".bright_green()
    );
    let time = Instant::now();
    match run_from_file(root_file_path, &mut context) {
        Ok(_) => {},
        Err(v) => v.run(),
    };
    time.elapsed().as_nanos()
    // println!("total time {}", x);
    // x
}

pub fn run_from_file(file_path: &str, context: *mut Context) -> Result<Returnable, Exception> {
    // let file_open_time = Instant::now();
    let mut file_data = String::new();
    File::open(file_path)
        .unwrap()
        .read_to_string(&mut file_data)
        .unwrap();
    file_data.push('\0');
    // println!("file open time {}", file_open_time.elapsed().as_nanos());
    run_with_data(
        Rc::new(FileData::new(
            file_data.replace('\r', ""),
            file_path.to_string(),
        )),
        context,
    )
}

pub(crate) fn run_with_data(
    file_data: Rc<FileData>,
    context: *mut Context,
) -> Result<Returnable, Exception> {
    let tokens = Lexer::lex(file_data.clone());
    let instructions = Parser::parse(file_data.clone(), tokens);
    run_with_instructions(&instructions, context, file_data)
}

pub(crate) fn run_with_instructions(
    instructions: &Vec<Instruction>,
    context: *mut Context,
    file_data: Rc<FileData>,
) -> Result<Returnable, Exception> {
    // let run_time = Instant::now();
    let start = match instructions.last() {
        Some(v) => v.start.clone(),
        None => Position::new(0),
    };
    let end = match instructions.last() {
        Some(v) => v.end.clone(),
        None => Position::new(0),
    };

    for instruction in instructions {
        if matches!(
            instruction.instruction_type,
            InstructionType::UseStatement { .. }
        ) {
            instruction.visit(context)?;
        }
    }

    for instruction in instructions {
        if matches!(
            instruction.instruction_type,
            InstructionType::ClassDeclaration { .. }
        ) {
            instruction.visit(context)?;
        }
    }

    for instruction in instructions {
        if matches!(
            instruction.instruction_type,
            InstructionType::FunctionDeclaration { .. }
        ) {
            instruction.visit(context)?;
        }
    }

    let mut return_value = Data::new(file_data, start, end, parser::data::DataType::Null);
    for instruction in instructions {
        if matches!(
            instruction.instruction_type,
            InstructionType::UseStatement { .. } | InstructionType::FunctionDeclaration { .. }
        ) {
            continue;
        }
        let value = match match instruction.visit(context) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "{}",
                    instructions
                        .iter()
                        .map(|z| format!("{} ", z.instruction_type))
                        .collect::<String>()
                );
                e.run()
            }
        } {
            Returnable::Return(val) => return Ok(Returnable::Return(val)),
            Returnable::Evaluate(val) => val,
            Returnable::Break(val) => val,
        };
        return_value = value
    }
    Ok(Returnable::Evaluate(return_value))
}
#[derive(Debug, PartialEq, Eq, PartialOrd)]
struct FileData {
    data: String,
    path: String,
}

impl FileData {
    pub fn new(data: String, path: String) -> Self {
        Self { data, path }
    }
}

#[derive(Clone, Debug)]
struct Position {
    value: usize,
}

impl Position {
    fn new(value: usize) -> Self {
        Self { value }
    }

    fn advance_by(&mut self, by: i32) -> &mut Position {
        self.value = (self.value as i32 + by).max(0) as usize;
        self
    }

    fn advance_by_owned(mut self, by: i32) -> Position {
        self.value = (self.value as i32 + by).max(0) as usize;
        self
    }
}

#[derive(PartialEq, Clone)]
pub enum Keyword {
    Var,
    If,
    Else,
    Function,
    Class,
    Use,
    Return,
    Break,
    As,
    While,
    Catch,
    Do,
    Final,
    New,
    Pass,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Keyword::Var => "let",
                Keyword::Function => "function",
                Keyword::Class => "class",
                Keyword::If => "if",
                Keyword::Else => "else",
                Keyword::Use => "use",
                Keyword::Return => "return",
                Keyword::As => "as",
                Keyword::While => "while",
                Keyword::Catch => "catch",
                Keyword::Do => "do",
                Keyword::Break => "break",
                Keyword::Final => "final",
                Keyword::New => "new",
                Keyword::Pass => "pass",
            }
        )
    }
}

#[derive(Debug)]
pub enum Returnable {
    Return(Data),
    Evaluate(Data),
    Break(Data),
}

impl Returnable {
    pub(crate) fn unwrap(self) -> Data {
        match self {
            Returnable::Return(v) => v,
            Returnable::Evaluate(v) => v,
            Returnable::Break(v) => v,
        }
    }
}

impl Display for Returnable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            match self {
                Returnable::Return(_) => "return",
                Returnable::Evaluate(_) => "evaluate",
                Returnable::Break(_) => "break",
            },
            match self {
                Returnable::Return(v) => v.data_type.to_string(),
                Returnable::Evaluate(v) => v.data_type.to_string(),
                Returnable::Break(v) => v.data_type.to_string(),
            }
        )
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum Annotation {
    DocComment(String),
}

impl Annotation {
    fn from(_v: String, (start, end, file_data): (&Position, &Position, &Rc<FileData>)) -> Self {
        InvalidAnnotation::call(start, end, file_data)
    }
}

struct DocPiece {
    file_path: String,
    start: usize,
}

pub trait AsString {
    fn as_string(&self) -> String;
}
