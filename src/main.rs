#![allow(unused)]
use std::{env, cell::{RefMut, RefCell}, rc::Rc};

use colored::Colorize;
use environment::{register_environment_variables, ENV_FILE_NAME, ENV_NO_STD};
use lexer::Lexer;
use parser::instructions::{Instruction, InstructionKind, self, Literal};
use rand::Rng;
use utils::{read_file, FileData};

use crate::{lexer::tokens::TokenKind, environment::{ENV_DEV_DEBUG_LEXER, ENV_DEV_DEBUG_PARSER}, parser::{Parser, context::Context}, std_lib::*};

mod environment;
mod utils;
mod lexer;
mod tests;
mod exceptions;
mod parser;
mod std_lib;

fn main() {
    register_environment_variables();
    run_app()
}

fn run_app() {
    let root_context = Context::new(None, true);
    let cell = RefCell::new(root_context);
    let rc = Rc::new(cell);
    if env::var(ENV_NO_STD).unwrap() == "false" {
        Context::import_string(rc.clone(), &STD_MAIN.to_string(), &"std".to_string());
    }
    println!("{} {}", "Running".bright_green(), env::var(ENV_FILE_NAME).unwrap().clone().bright_green().bold());
    Context::import_file(rc.clone(), &env::var(ENV_FILE_NAME).unwrap());
}

pub fn run(instructions: &Vec<Instruction>, file_data: &FileData, context: Rc<RefCell<Context>>) -> Literal {
    for instruction in instructions {
        if matches!(instruction.kind, InstructionKind::Use { .. }) {
            instruction.visit(file_data, context.clone());
        }
    }
    for instruction in instructions {
        if matches!(instruction.kind, InstructionKind::DeclareFunction { .. }) {
            instruction.visit(file_data, context.clone());
        }
    }
    let mut last_statement = Literal::Null;
    for instruction in instructions {
        if !matches!(instruction.kind, InstructionKind::DeclareFunction { .. } | InstructionKind::Use { .. }) {
            let v = instruction.visit(file_data, context.clone());
            match &v {
                Literal::Return(l) => return *l.clone(),
                _ => last_statement = v,
            }
        }
    }
    last_statement
}
